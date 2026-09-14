use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DuplicateCommitProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DuplicateCommitProbePort {
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

impl AuthorizedContextOperationPort for DuplicateCommitProbePort {
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

#[test]
fn duplicate_commit_replay_is_rejected_without_consuming_the_pending_witness() {
    let context = BrowsingContextId::new(1321).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DuplicateCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("duplicate-commit-user-context-1321")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1321).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues one pending witness");

    bound
        .record_observed_navigation_committed(&pending)
        .expect("the first matching commit records non-terminal progress");
    let state_after_first_commit = bound.browser_session().state();
    let recovery_after_first_commit = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a duplicate navigationCommitted delivery for the same witness must fail closed rather than being accepted as fresh progress"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_first_commit,
        "duplicate commit rejection must not mutate aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_first_commit.as_slice(),
        "duplicate commit rejection must not mutate recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit replay must not manufacture a terminal-derived re-establishment opportunity"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-duplicate-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "duplicate commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "commit progress and duplicate replay rejection must remain zero-I/O"
    );

    bound
        .record_observed_navigation_settled(&pending)
        .expect("duplicate commit rejection must leave the original pending witness available for one qualified closure");
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("the later qualified closure permits exactly one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "duplicate commit replay must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the closure-derived re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-duplicate-commit"),
        Ok(context),
        "the authority minted after the qualified closure must remain executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}
