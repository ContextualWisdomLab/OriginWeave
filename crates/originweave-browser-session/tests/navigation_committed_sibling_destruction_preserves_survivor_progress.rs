use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CommittedSiblingDestructionProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CommittedSiblingDestructionProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handles
            .pop_front()
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

impl AuthorizedContextOperationPort for CommittedSiblingDestructionProbePort {
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

fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

fn assert_surviving_context_stays_committed_and_locked(
    bound: &mut originweave_browser_session::BoundBrowserSession<CommittedSiblingDestructionProbePort>,
    context: BrowsingContextId,
    retained_authority: &originweave_browser_session::PresentationMutationAuthority,
    adapter_calls: &Rc<Cell<usize>>,
    expected_calls: usize,
    operation: &'static str,
) {
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroying a sibling must not manufacture re-establishment eligibility for a committed survivor"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(retained_authority, operation),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "destroying a sibling must not reactivate the survivor's retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        expected_calls,
        "survivor authority probes must fail before adapter I/O"
    );
}

#[test]
fn destroying_one_committed_context_preserves_sibling_duplicate_commit_rejection() {
    let first_context = BrowsingContextId::new(1361).expect("valid first context");
    let second_context = BrowsingContextId::new(1362).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CommittedSiblingDestructionProbePort {
        handles: VecDeque::from([
            handle(first_context, "committed-destroy-first-user-context-1361"),
            handle(second_context, "committed-destroy-second-user-context-1362"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1361).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first context enters navigation-pending state");
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("second context independently enters navigation-pending state");
    let state_after_starts = bound.browser_session().state();
    let recovery_after_starts = bound.browser_session().recovery_evidence().to_vec();
    let calls_after_starts = adapter_calls.get();

    bound
        .record_observed_navigation_committed(&first_pending)
        .expect("first context records non-terminal commit progress");
    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("second context independently records non-terminal commit progress");
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_starts);
    assert_surviving_context_stays_committed_and_locked(
        &mut bound,
        second_context,
        &second_authority,
        &adapter_calls,
        calls_after_starts,
        "second-stale-before-sibling-destruction",
    );

    bound
        .destroy_owned_disposable_context(first_context)
        .expect("proven destruction consumes only the first context ownership generation");
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "destroying one committed context must preserve aggregate trust"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice(),
        "proven destruction must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_starts + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    let calls_after_destroy = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::ContextNotOwned),
        "destroyed context commit evidence is permanently stale"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_surviving_context_stays_committed_and_locked(
        &mut bound,
        second_context,
        &second_authority,
        &adapter_calls,
        calls_after_destroy,
        "second-stale-after-destroyed-sibling-replay",
    );

    assert_eq!(
        bound.record_observed_navigation_committed(&second_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroying the sibling must not clear the survivor's already-consumed commit slot"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "duplicate survivor commit rejection must remain non-mutating after sibling destruction"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_surviving_context_stays_committed_and_locked(
        &mut bound,
        second_context,
        &second_authority,
        &adapter_calls,
        calls_after_destroy,
        "second-stale-after-own-duplicate-commit",
    );

    bound
        .record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Failed,
        )
        .expect("the surviving committed navigation may still close through its own terminal outcome");
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-after-terminal-closure",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "terminal closure creates eligibility but never reactivates retained authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);

    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("the surviving context receives one explicit fresh authority");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "destroying the sibling must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the survivor's re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-current"),
        Ok(second_context),
        "fresh authority after sibling destruction must be executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy + 1);
}
