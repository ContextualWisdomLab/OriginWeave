use std::cell::Cell;

use originweave_browser_session::{
    BrowserSession, DisposableContextCreateError, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableContextPortId, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct SideEffectingIdentityPort {
    identity_callbacks: Cell<usize>,
    create_calls: usize,
}

impl SideEffectingIdentityPort {
    fn new() -> Self {
        Self {
            identity_callbacks: Cell::new(0),
            create_calls: 0,
        }
    }
}

impl DisposableContextPort for SideEffectingIdentityPort {
    fn port_id(&self) -> DisposableContextPortId {
        self.identity_callbacks
            .set(self.identity_callbacks.get().saturating_add(1));
        DisposableContextPortId::new(401).expect("valid port id")
    }

    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls += 1;
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("preflight-user-context").expect("valid isolation id"),
            BrowsingContextId::new(401).expect("valid browsing context"),
        ))
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

#[test]
fn lifecycle_authority_does_not_depend_on_side_effecting_identity_preflight() {
    let mut session = BrowserSession::start(BrowserSessionId::new(401).expect("valid session id"))
        .expect("incarnation capacity");
    let mut port = SideEffectingIdentityPort::new();

    session
        .create_disposable_context(&mut port)
        .expect("authorized create");

    assert_eq!(
        port.identity_callbacks.get(),
        0,
        "Browser Session invoked an arbitrary adapter callback before lifecycle authority was established"
    );
    assert_eq!(port.create_calls, 1);
}
