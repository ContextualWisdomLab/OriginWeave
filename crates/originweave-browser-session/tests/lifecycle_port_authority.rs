use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableContextPortId, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct RecordingPort {
    port_id: DisposableContextPortId,
    create_calls: usize,
    destroy_calls: usize,
}

impl RecordingPort {
    fn new(port_id: u64) -> Self {
        Self {
            port_id: DisposableContextPortId::new(port_id).expect("valid port id"),
            create_calls: 0,
            destroy_calls: 0,
        }
    }
}

impl DisposableContextPort for RecordingPort {
    fn port_id(&self) -> DisposableContextPortId {
        self.port_id
    }

    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        assert_eq!(request.port_id(), self.port_id);
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
        assert_eq!(request.port_id(), self.port_id);
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
fn aggregate_issued_request_binds_lifecycle_io_to_one_port() {
    let mut session = BrowserSession::start(BrowserSessionId::new(7).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound_port = RecordingPort::new(101);
    let authority = session
        .create_disposable_context(&mut bound_port)
        .expect("Browser Session-issued create request");
    assert_eq!(bound_port.create_calls, 1);

    let mut other_port = RecordingPort::new(102);
    assert_eq!(
        session.create_disposable_context(&mut other_port),
        Err(BrowserSessionError::LifecyclePortMismatch)
    );
    assert_eq!(other_port.create_calls, 0, "wrong port reached create I/O");
    assert_eq!(
        session.destroy_disposable_context(&authority, &mut other_port),
        Err(BrowserSessionError::LifecyclePortMismatch)
    );
    assert_eq!(
        other_port.destroy_calls, 0,
        "wrong port reached destroy I/O"
    );

    session
        .destroy_disposable_context(&authority, &mut bound_port)
        .expect("Browser Session-issued destroy request");
    assert_eq!(bound_port.destroy_calls, 1);
}
