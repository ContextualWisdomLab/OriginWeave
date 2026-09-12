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

struct TransportLossProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for TransportLossProbePort {
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

impl AuthorizedContextOperationPort for TransportLossProbePort {
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
fn buffered_navigation_after_transport_loss_cannot_mutate_recovery_state_or_revive_authority() {
    let context = BrowsingContextId::new(961).expect("valid browsing context");
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("transport-loss-navigation-user-context-961")
            .expect("valid isolation id"),
        context,
    );
    let adapter_calls = Rc::new(Cell::new(0));
    let port = TransportLossProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(961).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let pre_loss_authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_loss_authority, "before-loss"),
        Ok(context)
    );

    assert!(bound.record_transport_loss());
    let calls_after_transport_loss = adapter_calls.get();
    let recovery_evidence_after_transport_loss = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation(context),
        Err(BrowserSessionError::SessionNotActive),
        "a buffered navigation event from a dead transport must not be accepted as current browser state"
    );
    assert_eq!(
        bound.record_observed_navigation(context),
        Err(BrowserSessionError::SessionNotActive),
        "replayed buffered navigation after transport loss must remain fail-closed"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_loss_authority, "after-loss"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::SessionNotActive
        )),
        "late navigation observation must never revive pre-loss presentation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_transport_loss,
        "late navigation and retained authority must both fail before adapter I/O after transport loss"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_transport_loss.as_slice(),
        "late navigation must not rewrite or duplicate exact transport-loss recovery evidence"
    );
}
