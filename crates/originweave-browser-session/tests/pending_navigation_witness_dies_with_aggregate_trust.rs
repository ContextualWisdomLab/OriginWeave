use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct AggregateTrustLossProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
    fail_destroy: bool,
}

impl DisposableContextPort for AggregateTrustLossProbePort {
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

#[test]
fn pending_navigation_witness_cannot_settle_after_transport_loss() {
    let context = BrowsingContextId::new(1021).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = AggregateTrustLossProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("pending-navigation-transport-loss-1021")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
        fail_destroy: false,
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1021).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("active owned navigation issues one pending witness");

    assert!(bound.record_transport_loss());
    let calls_after_loss = adapter_calls.get();
    let recovery_evidence_after_loss = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::SessionNotActive),
        "aggregate transport trust must gate before a previously issued settlement witness is considered"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::SessionNotActive),
        "a negative terminal replay from the dead transport must not consume or revive pending navigation state"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::TransportLost);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_loss.as_slice()
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_loss,
        "terminal replay after transport loss must fail before adapter I/O"
    );
}

#[test]
fn pending_navigation_witness_cannot_mutate_recovery_required() {
    let context = BrowsingContextId::new(1022).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = AggregateTrustLossProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("pending-navigation-recovery-1022")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
        fail_destroy: true,
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1022).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("active owned navigation issues one pending witness");

    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextDestructionFailed),
        "unproven lifecycle cleanup must enter recovery even while presentation is pending"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::RecoveryRequired);
    let calls_after_failure = adapter_calls.get();
    let recovery_evidence_after_failure = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::SessionNotActive),
        "recovery state must take precedence over an otherwise exact pending witness"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::SessionNotActive),
        "late failure evidence must not rewrite lifecycle recovery state"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "terminal replay must preserve the exact unproven-destruction evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_failure,
        "terminal replay during recovery must fail before adapter I/O"
    );
}

#[test]
fn pending_navigation_witness_cannot_outlive_normal_session_end() {
    let context = BrowsingContextId::new(1023).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = AggregateTrustLossProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("pending-navigation-ended-1023")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
        fail_destroy: false,
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1023).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("active owned navigation issues one pending witness");

    bound
        .destroy_owned_disposable_context(context)
        .expect("proven lifecycle cleanup is independent from presentation settlement");
    bound.end().expect("clean session end after proven lifecycle cleanup");
    let calls_after_end = adapter_calls.get();

    assert_eq!(bound.browser_session().state(), BrowserSessionState::Ended);
    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::SessionNotActive),
        "an opaque witness issued by an ended aggregate must not remain usable"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::SessionNotActive),
        "an ended aggregate must reject all terminal replay before witness validation"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Ended);
    assert_eq!(
        adapter_calls.get(),
        calls_after_end,
        "terminal replay after end must fail before adapter I/O"
    );
}
