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

struct LateCommitAfterTerminalProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for LateCommitAfterTerminalProbePort {
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

impl AuthorizedContextOperationPort for LateCommitAfterTerminalProbePort {
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
    context: BrowsingContextId,
    adapter_calls: &Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<LateCommitAfterTerminalProbePort> {
    let port = LateCommitAfterTerminalProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(format!(
                "terminal-late-commit-user-context-{session_id}"
            ))
            .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(adapter_calls),
    };

    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn positive_terminal_consumption_rejects_late_commit_before_and_after_reestablishment() {
    let context = BrowsingContextId::new(1261).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1261, context, &adapter_calls);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");
    bound
        .record_observed_navigation_settled(&pending)
        .expect("matching complete-positive evidence consumes the pending witness");

    let state_after_terminal = bound.browser_session().state();
    let recovery_after_terminal = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationCommitted replay must not reuse a witness consumed by positive settlement"
    );
    assert_eq!(bound.browser_session().state(), state_after_terminal);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_terminal.as_slice()
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-positive-late-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "late commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("late commit replay must preserve the one positive-terminal opportunity");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "late commit replay before re-establishment must not spend an epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "positive-terminal re-establishment remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let state_after_reestablishment = bound.browser_session().state();
    let recovery_after_reestablishment = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the positive-terminal witness stays dead after fresh authority is minted"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_reestablishment
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_reestablishment.as_slice()
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "post-reestablishment late commit must not manufacture hidden second eligibility"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-positive-late-commit"),
        Ok(context),
        "the already-issued current authority must remain executable"
    );

    let calls_before_next_navigation = adapter_calls.get();
    let next_pending = bound
        .record_observed_navigation(fresh.incarnation(), context, fresh.context_epoch())
        .expect("a later navigation may start normally");
    bound
        .record_observed_navigation_settled(&next_pending)
        .expect("the later navigation may settle normally");
    let next = bound
        .reestablish_presentation_authority(context)
        .expect("the later settlement permits one fresh authority");
    assert_eq!(
        next.context_epoch().value(),
        fresh.context_epoch().value() + 1,
        "stale commit replay must not perturb the aggregate epoch sequence"
    );
    assert_eq!(adapter_calls.get(), calls_before_next_navigation);
    assert_eq!(
        bound.execute_authorized_context_operation(&next, "usable-after-next-positive-navigation"),
        Ok(context),
        "the next epoch must represent executable authority rather than a hollow token"
    );
    assert_eq!(adapter_calls.get(), calls_before_next_navigation + 1);
}

#[test]
fn negative_terminal_consumption_rejects_late_commit_before_and_after_reestablishment() {
    let context = BrowsingContextId::new(1262).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1262, context, &adapter_calls);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");
    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed)
        .expect("matching negative terminal evidence consumes the pending witness");

    let state_after_terminal = bound.browser_session().state();
    let recovery_after_terminal = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationCommitted replay must not reuse a witness consumed by negative termination"
    );
    assert_eq!(bound.browser_session().state(), state_after_terminal);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_terminal.as_slice()
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-negative-late-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "late commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("late commit replay must preserve the one negative-terminal opportunity");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "late commit replay before re-establishment must not spend an epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "negative-terminal re-establishment remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let state_after_reestablishment = bound.browser_session().state();
    let recovery_after_reestablishment = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the negative-terminal witness stays dead after fresh authority is minted"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_reestablishment
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_reestablishment.as_slice()
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "post-reestablishment late commit must not manufacture hidden second eligibility"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-negative-late-commit"),
        Ok(context),
        "the already-issued current authority must remain executable"
    );

    let calls_before_next_navigation = adapter_calls.get();
    let next_pending = bound
        .record_observed_navigation(fresh.incarnation(), context, fresh.context_epoch())
        .expect("a later navigation may start normally");
    bound
        .record_observed_navigation_terminated(&next_pending, NavigationTerminationOutcome::Aborted)
        .expect("the later navigation may terminate normally");
    let next = bound
        .reestablish_presentation_authority(context)
        .expect("the later terminal outcome permits one fresh authority");
    assert_eq!(
        next.context_epoch().value(),
        fresh.context_epoch().value() + 1,
        "stale commit replay must not perturb the aggregate epoch sequence"
    );
    assert_eq!(adapter_calls.get(), calls_before_next_navigation);
    assert_eq!(
        bound.execute_authorized_context_operation(&next, "usable-after-next-negative-navigation"),
        Ok(context),
        "the next epoch must represent executable authority rather than a hollow token"
    );
    assert_eq!(adapter_calls.get(), calls_before_next_navigation + 1);
}
