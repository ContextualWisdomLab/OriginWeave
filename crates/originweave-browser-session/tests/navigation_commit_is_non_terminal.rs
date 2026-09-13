use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CommitProgressProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CommitProgressProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handle
            .take()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }
}

impl AuthorizedContextOperationPort for CommitProgressProbePort {
    type Operation = &'static str;
    type Output = BrowsingContextId;
    type Error = ();

    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(request.context().browsing_context())
    }
}

fn bound_session(
    session_id: u64,
    context_id: u64,
    adapter_calls: Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<CommitProgressProbePort> {
    let context = BrowsingContextId::new(context_id).expect("valid browsing context");
    let port = CommitProgressProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(format!("commit-progress-user-context-{context_id}"))
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls,
    };

    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn commit_progress_does_not_restore_authority_and_later_failure_can_still_terminate() {
    let context = BrowsingContextId::new(1101).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1101, 1101, Rc::clone(&adapter_calls));

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");

    bound
        .record_observed_navigation_committed(&pending)
        .expect("commit is progress for the current navigation, not its terminal settlement");
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationCommitted alone must not restore mutation authority while later abort/failure remains possible"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "pre-navigation authority remains revoked after commit progress"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "commit progress and stale-authority rejection must not perform browser I/O"
    );

    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed)
        .expect("a matching failure after commit must remain an admissible terminal outcome");
    assert_eq!(adapter_calls.get(), calls_after_create);

    let after_failure = bound
        .reestablish_presentation_authority(context)
        .expect("explicit re-establishment is allowed only after the later terminal outcome");
    assert_eq!(
        after_failure.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "commit progress must not consume a presentation epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_failure, "usable-after-failure-terminal"),
        Ok(context)
    );
}

#[test]
fn commit_progress_preserves_witness_until_positive_completion() {
    let context = BrowsingContextId::new(1102).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1102, 1102, Rc::clone(&adapter_calls));

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");

    bound
        .record_observed_navigation_committed(&pending)
        .expect("commit records non-terminal progress without consuming the witness");
    bound
        .record_observed_navigation_settled(&pending)
        .expect("the same witness remains available for later complete positive settlement");
    assert_eq!(adapter_calls.get(), calls_after_create);

    let after_completion = bound
        .reestablish_presentation_authority(context)
        .expect("complete positive settlement permits explicit fresh authority");
    assert_eq!(
        after_completion.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "only explicit re-establishment after completion advances the presentation epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_completion, "usable-after-completion"),
        Ok(context)
    );
}
