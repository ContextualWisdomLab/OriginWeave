use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
};
use originweave_core::BrowserSessionId;

struct SideEffectingDebugPort {
    debug_callbacks: Rc<Cell<usize>>,
}

impl fmt::Debug for SideEffectingDebugPort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_callbacks
            .set(self.debug_callbacks.get().saturating_add(1));
        formatter.write_str("adapter-secret-sentinel")
    }
}

impl DisposableContextPort for SideEffectingDebugPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        Err(DisposableContextCreateError::CreateFailedClean)
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
fn bound_session_debug_never_executes_or_exposes_adapter_debug() {
    let debug_callbacks = Rc::new(Cell::new(0));
    let port = SideEffectingDebugPort {
        debug_callbacks: Rc::clone(&debug_callbacks),
    };
    let session = BrowserSession::start(BrowserSessionId::new(502).expect("valid session id"))
        .expect("incarnation capacity");
    let bound = session.bind_lifecycle_port(port);

    let rendered = format!("{bound:?}");

    assert_eq!(
        debug_callbacks.get(),
        0,
        "formatting a bound session must not execute adapter-owned Debug code"
    );
    assert!(
        !rendered.contains("adapter-secret-sentinel"),
        "bound-session diagnostics must not expose adapter-internal state"
    );
}
