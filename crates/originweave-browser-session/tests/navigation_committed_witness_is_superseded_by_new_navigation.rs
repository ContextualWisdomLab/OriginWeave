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

struct CommittedSupersessionProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CommittedSupersessionProbePort {
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

impl AuthorizedContextOperationPort for CommittedSupersessionProbePort {
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
fn newer_navigation_kills_an_older_witness_even_after_commit_progress() {
    let context = BrowsingContextId::new(1331).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CommittedSupersessionProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("committed-supersession-user-context-1331")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1331).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation start issues a pending witness");
    bound
        .record_observed_navigation_committed(&first_pending)
        .expect("the first navigation may record non-terminal commit progress");

    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("a newer same-context navigation supersedes the committed-but-unsettled witness");
    let state_after_supersession = bound.browser_session().state();
    let recovery_after_supersession = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "commit progress from the superseded witness must not survive into the newer navigation"
    );
    assert_eq!(bound.browser_session().state(), state_after_supersession);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice()
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "a stale duplicate commit must not manufacture re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-old-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "a stale duplicate commit must not reactivate retained authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "late positive settlement from the superseded committed witness must not close the newer navigation"
    );
    assert_eq!(bound.browser_session().state(), state_after_supersession);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice()
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale positive evidence must not manufacture re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-old-positive"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale positive evidence must not reactivate retained authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "late negative settlement from the superseded committed witness must not close the newer navigation"
    );
    assert_eq!(bound.browser_session().state(), state_after_supersession);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice()
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale negative evidence must not manufacture re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-old-negative"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale negative evidence must not reactivate retained authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "late download-start evidence from the superseded committed witness must not close the newer navigation"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_supersession,
        "each stale observation from the committed predecessor must be rejected before aggregate lifecycle mutation"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice(),
        "each stale observation from the committed predecessor must not manufacture recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale download evidence must not manufacture re-establishment eligibility while the newer navigation is pending"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-old-download"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale download evidence must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "commit progress, supersession, each stale-evidence rejection, and authority rejection must remain zero-I/O"
    );

    bound
        .record_observed_navigation_settled(&second_pending)
        .expect("only the current pending witness may close the navigation");
    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("the current navigation permits exactly one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "superseding a committed predecessor and rejecting all of its late evidence must not spend an extra presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the current closure-derived re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-committed-supersession"),
        Ok(context),
        "the authority minted from the current navigation must remain executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}
