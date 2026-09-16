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
struct RecordingPort {
    context: BrowsingContextId,
    isolation: &'static str,
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
}

impl RecordingPort {
    fn new(
        context: u64,
        isolation: &'static str,
        create_calls: Rc<Cell<usize>>,
        destroy_calls: Rc<Cell<usize>>,
    ) -> Self {
        Self {
            context: BrowsingContextId::new(context).expect("valid browsing context"),
            isolation,
            create_calls,
            destroy_calls,
        }
    }
}

impl DisposableContextPort for RecordingPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls.set(self.create_calls.get() + 1);
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse(self.isolation).expect("valid isolation id"),
            self.context,
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
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Ok(())
    }
}

#[test]
fn distinct_adapter_cannot_be_substituted_for_create_after_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(17).expect("valid session id"))
        .expect("incarnation capacity");
    let approved_create_calls = Rc::new(Cell::new(0));
    let approved_destroy_calls = Rc::new(Cell::new(0));
    let spoof_create_calls = Rc::new(Cell::new(0));
    let spoof_destroy_calls = Rc::new(Cell::new(0));
    let approved_port = RecordingPort::new(
        41,
        "approved-isolation",
        Rc::clone(&approved_create_calls),
        Rc::clone(&approved_destroy_calls),
    );
    let _spoofing_port = RecordingPort::new(
        42,
        "spoofed-isolation",
        Rc::clone(&spoof_create_calls),
        Rc::clone(&spoof_destroy_calls),
    );
    let mut bound = session.bind_lifecycle_port(approved_port);

    bound
        .create_disposable_context()
        .expect("bound adapter creates context");
    assert_eq!(approved_create_calls.get(), 1);
    assert_eq!(spoof_create_calls.get(), 0);
}

#[test]
fn distinct_adapter_cannot_be_substituted_for_destroy_after_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(18).expect("valid session id"))
        .expect("incarnation capacity");
    let approved_create_calls = Rc::new(Cell::new(0));
    let approved_destroy_calls = Rc::new(Cell::new(0));
    let spoof_create_calls = Rc::new(Cell::new(0));
    let spoof_destroy_calls = Rc::new(Cell::new(0));
    let approved_port = RecordingPort::new(
        51,
        "approved-isolation",
        Rc::clone(&approved_create_calls),
        Rc::clone(&approved_destroy_calls),
    );
    let _spoofing_port = RecordingPort::new(
        52,
        "spoofed-isolation",
        Rc::clone(&spoof_create_calls),
        Rc::clone(&spoof_destroy_calls),
    );
    let mut bound = session.bind_lifecycle_port(approved_port);
    let authority = bound
        .create_disposable_context()
        .expect("bound adapter creates context");

    bound
        .destroy_disposable_context(&authority)
        .expect("bound adapter destroys context");
    assert_eq!(approved_destroy_calls.get(), 1);
    assert_eq!(spoof_destroy_calls.get(), 0);
}
