use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct RecoveryProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for RecoveryProbePort {
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
        Err(DisposableContextDestroyError::DestroyFailed)
    }
}

#[test]
fn navigation_replay_after_recovery_required_cannot_become_invalidated_state_idempotency() {
    let context = BrowsingContextId::new(971).expect("valid browsing context");
    let foreign = BrowsingContextId::new(972).expect("valid foreign browsing context");
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("recovery-navigation-user-context-971")
            .expect("valid isolation id"),
        context,
    );
    let adapter_calls = Rc::new(Cell::new(0));
    let port = RecoveryProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(971).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .record_observed_navigation(context, pre_navigation.context_epoch())
        .expect("active owned navigation invalidates presentation authority");

    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextDestructionFailed),
        "unproven cleanup must enter recovery"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );

    let calls_after_failure = adapter_calls.get();
    let recovery_evidence_after_failure = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation(foreign, pre_navigation.context_epoch()),
        Err(BrowserSessionError::SessionNotActive),
        "aggregate recovery trust must be rejected before a raw context selector can reveal ownership"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "foreign navigation probing must not rewrite RecoveryRequired"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "foreign navigation probing must leave the original unproven-destruction evidence unchanged"
    );
    assert_eq!(
        bound.record_observed_navigation(context, pre_navigation.context_epoch()),
        Err(BrowserSessionError::SessionNotActive),
        "aggregate recovery state must take precedence over duplicate-while-invalidated idempotency"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "rejected navigation must not rewrite the aggregate out of RecoveryRequired"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "rejected navigation must leave the original unproven-destruction evidence unchanged"
    );

    assert_eq!(
        bound.record_observed_navigation(context, pre_navigation.context_epoch()),
        Err(BrowserSessionError::SessionNotActive),
        "replayed navigation cannot be accepted after ownership recovery begins"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "replayed navigation must not rewrite RecoveryRequired into another inactive state"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_failure,
        "foreign probing and navigation received after RecoveryRequired must fail before adapter I/O"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "late navigation must not rewrite or duplicate unproven-destruction recovery evidence"
    );
}
