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
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
}

impl RecordingPort {
    fn new(create_calls: Rc<Cell<usize>>, destroy_calls: Rc<Cell<usize>>) -> Self {
        Self {
            create_calls,
            destroy_calls,
        }
    }
}

impl DisposableContextPort for RecordingPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        assert_eq!(request.browser_session(), BrowserSessionId::new(7).unwrap());
        self.create_calls.set(self.create_calls.get() + 1);
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("aggregate-issued-request").expect("valid isolation id"),
            BrowsingContextId::new(41).expect("valid browsing context"),
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
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        assert_eq!(request.browser_session(), BrowserSessionId::new(7).unwrap());
        assert_eq!(
            request.context().browsing_context(),
            BrowsingContextId::new(41).unwrap()
        );
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Ok(())
    }
}

#[test]
fn aggregate_issued_request_is_reachable_only_through_owned_port_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(7).expect("valid session id"))
        .expect("incarnation capacity");
    let approved_create_calls = Rc::new(Cell::new(0));
    let approved_destroy_calls = Rc::new(Cell::new(0));
    let other_create_calls = Rc::new(Cell::new(0));
    let other_destroy_calls = Rc::new(Cell::new(0));
    let _unbound_other_port = RecordingPort::new(
        Rc::clone(&other_create_calls),
        Rc::clone(&other_destroy_calls),
    );
    let mut bound = session.bind_lifecycle_port(RecordingPort::new(
        Rc::clone(&approved_create_calls),
        Rc::clone(&approved_destroy_calls),
    ));

    let authority = bound
        .create_disposable_context()
        .expect("Browser Session-issued create request");
    assert_eq!(approved_create_calls.get(), 1);
    assert_eq!(other_create_calls.get(), 0);

    bound
        .destroy_disposable_context(&authority)
        .expect("Browser Session-issued destroy request");
    assert_eq!(approved_destroy_calls.get(), 1);
    assert_eq!(other_destroy_calls.get(), 0);
}
