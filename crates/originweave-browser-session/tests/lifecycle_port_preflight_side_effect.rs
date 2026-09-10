use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct SideEffectingIdentityPort {
    identity_callbacks: Rc<Cell<usize>>,
    create_calls: Rc<Cell<usize>>,
}

impl SideEffectingIdentityPort {
    fn new(identity_callbacks: Rc<Cell<usize>>, create_calls: Rc<Cell<usize>>) -> Self {
        Self {
            identity_callbacks,
            create_calls,
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
        self.create_calls
            .set(self.create_calls.get().saturating_add(1));
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("preflight-user-context").expect("valid isolation id"),
            BrowsingContextId::new(401).expect("valid browsing context"),
        ))
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        Ok(())
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
    let identity_callbacks = Rc::new(Cell::new(0));
    let create_calls = Rc::new(Cell::new(0));
    let port = SideEffectingIdentityPort::new(
        Rc::clone(&identity_callbacks),
        Rc::clone(&create_calls),
    );

    // Prove the fixture observes a shared-reference callback without retaining adapter access after bind.
    port.identity_probe();
    assert_eq!(identity_callbacks.get(), 1);
    identity_callbacks.set(0);

    let mut bound = session.bind_lifecycle_port(port);
    assert_eq!(
        identity_callbacks.get(),
        0,
        "binding invoked adapter code before aggregate-issued lifecycle authority existed"
    );
    bound
        .create_disposable_context()
        .expect("authorized create");
    assert_eq!(identity_callbacks.get(), 0);
    assert_eq!(create_calls.get(), 1);
}
