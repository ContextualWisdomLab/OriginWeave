use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
    NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ReestablishmentTrustProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
    fail_destroy: bool,
}

impl DisposableContextPort for ReestablishmentTrustProbePort {
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
        if self.fail_destroy {
            Err(DisposableContextDestroyError::DestroyFailed)
        } else {
            Ok(())
        }
    }
}

fn bound_session(
    session: u64,
    context: BrowsingContextId,
    isolation: &str,
    adapter_calls: &Rc<Cell<usize>>,
    fail_destroy: bool,
) -> originweave_browser_session::BoundBrowserSession<ReestablishmentTrustProbePort> {
    let port = ReestablishmentTrustProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(isolation).expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(adapter_calls),
        fail_destroy,
    };
    BrowserSession::start(BrowserSessionId::new(session).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn positive_terminal_reestablishment_eligibility_dies_with_transport_trust() {
    let context = BrowsingContextId::new(1035).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1035,
        context,
        "terminal-reestablishment-transport-loss-1035",
        &adapter_calls,
        false,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_settled(&pending)
        .expect("matching positive terminal creates one re-establishment eligibility");

    assert!(
        bound.record_transport_loss(),
        "transport loss must revoke aggregate trust before re-establishment"
    );
    let calls_after_loss = adapter_calls.get();
    let recovery_evidence_after_loss = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::SessionNotActive),
        "positive terminal eligibility must not outlive aggregate transport trust"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost,
        "re-establishment must not rewrite transport-loss state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_loss.as_slice(),
        "rejected re-establishment must preserve transport-loss recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_loss,
        "post-loss re-establishment must fail before adapter I/O"
    );
}

#[test]
fn negative_terminal_reestablishment_eligibility_dies_when_recovery_is_required() {
    let context = BrowsingContextId::new(1036).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1036,
        context,
        "terminal-reestablishment-recovery-required-1036",
        &adapter_calls,
        true,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed)
        .expect("matching negative terminal creates one re-establishment eligibility");

    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextDestructionFailed),
        "unproven destruction must move the aggregate into recovery before re-establishment"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );
    let calls_after_failure = adapter_calls.get();
    let recovery_evidence_after_failure = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::SessionNotActive),
        "negative terminal eligibility must not override RecoveryRequired"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "re-establishment must not rewrite unresolved lifecycle recovery state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "rejected re-establishment must preserve exact unproven-destruction evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_failure,
        "re-establishment during recovery must fail before adapter I/O"
    );
}

#[test]
fn aborted_terminal_reestablishment_eligibility_dies_with_transport_trust() {
    let context = BrowsingContextId::new(1037).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1037,
        context,
        "terminal-reestablishment-aborted-transport-loss-1037",
        &adapter_calls,
        false,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Aborted)
        .expect("matching aborted terminal creates one re-establishment eligibility");

    assert!(
        bound.record_transport_loss(),
        "transport loss must revoke aggregate trust after an aborted navigation"
    );
    let calls_after_loss = adapter_calls.get();
    let recovery_evidence_after_loss = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::SessionNotActive),
        "aborted terminal eligibility must not outlive aggregate transport trust"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost,
        "re-establishment must not rewrite transport loss after an aborted navigation"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_loss.as_slice(),
        "rejected post-abort re-establishment must preserve transport-loss evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_loss,
        "post-abort re-establishment after transport loss must fail before adapter I/O"
    );
}

#[test]
fn aborted_terminal_reestablishment_eligibility_dies_when_recovery_is_required() {
    let context = BrowsingContextId::new(1038).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1038,
        context,
        "terminal-reestablishment-aborted-recovery-required-1038",
        &adapter_calls,
        true,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Aborted)
        .expect("matching aborted terminal creates one re-establishment eligibility");

    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextDestructionFailed),
        "unproven destruction must enter recovery after an aborted navigation"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );
    let calls_after_failure = adapter_calls.get();
    let recovery_evidence_after_failure = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::SessionNotActive),
        "aborted terminal eligibility must not override RecoveryRequired"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "re-establishment must not rewrite recovery state after an aborted navigation"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "rejected post-abort re-establishment must preserve unproven-destruction evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_failure,
        "post-abort re-establishment during recovery must fail before adapter I/O"
    );
}
