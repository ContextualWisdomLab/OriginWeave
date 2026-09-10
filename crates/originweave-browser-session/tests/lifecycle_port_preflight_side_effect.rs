use std::cell::Cell;

use originweave_browser_session::{
    BrowserSession, DisposableContextCreateError, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableIsolationId,
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

    fn identity_probe(&self) {
        self.identity_callbacks
            .set(self.identity_callbacks.get().saturating_add(1));
    }
}

impl DisposableContextPort for SideEffectingIdentityPort {
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
fn lifecycle_binding_invokes_no_adapter_callback_before_authorized_create() {
    let session = BrowserSession::start(BrowserSessionId::new(401).expect("valid session id"))
        .expect("incarnation capacity");
    let port = SideEffectingIdentityPort::new();
    let mut bound = session.bind_lifecycle_port(port);

    assert_eq!(
        bound.lifecycle_port().identity_callbacks.get(),
        0,
        "binding invoked adapter code before aggregate-issued lifecycle authority existed"
    );
    bound
        .create_disposable_context()
        .expect("authorized create");
    assert_eq!(bound.lifecycle_port().identity_callbacks.get(), 0);
    assert_eq!(bound.lifecycle_port().create_calls, 1);

    // Prove the fixture would detect an identity callback if production code invoked one.
    bound.lifecycle_port().identity_probe();
    assert_eq!(bound.lifecycle_port().identity_callbacks.get(), 1);
}
