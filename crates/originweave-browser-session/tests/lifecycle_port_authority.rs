use originweave_browser_session::{
    BrowserSession, DisposableContextCreateError, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct RecordingPort {
    create_calls: usize,
    destroy_calls: usize,
}

impl RecordingPort {
    fn new() -> Self {
        Self {
            create_calls: 0,
            destroy_calls: 0,
        }
    }
}

impl DisposableContextPort for RecordingPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        assert_eq!(request.browser_session(), BrowserSessionId::new(7).unwrap());
        self.create_calls += 1;
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("aggregate-issued-request").expect("valid isolation id"),
            BrowsingContextId::new(41).expect("valid browsing context"),
        ))
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
        self.destroy_calls += 1;
        Ok(())
    }
}

#[test]
fn aggregate_issued_request_is_reachable_only_through_owned_port_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(7).expect("valid session id"))
        .expect("incarnation capacity");
    let unbound_other_port = RecordingPort::new();
    let mut bound = session.bind_lifecycle_port(RecordingPort::new());

    let authority = bound
        .create_disposable_context()
        .expect("Browser Session-issued create request");
    assert_eq!(bound.lifecycle_port().create_calls, 1);
    assert_eq!(unbound_other_port.create_calls, 0);

    bound
        .destroy_disposable_context(&authority)
        .expect("Browser Session-issued destroy request");
    assert_eq!(bound.lifecycle_port().destroy_calls, 1);
    assert_eq!(unbound_other_port.destroy_calls, 0);
}
