use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableContextPortId, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct RecordingPort {
    port_id: DisposableContextPortId,
    context: BrowsingContextId,
    isolation: &'static str,
    create_calls: usize,
    destroy_calls: usize,
}

impl RecordingPort {
    fn new(port_id: u64, context: u64, isolation: &'static str) -> Self {
        Self {
            port_id: DisposableContextPortId::new(port_id).expect("valid port id"),
            context: BrowsingContextId::new(context).expect("valid browsing context"),
            isolation,
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
        self.create_calls += 1;
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse(self.isolation).expect("valid isolation id"),
            self.context,
        ))
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        assert_eq!(request.port_id(), self.port_id);
        self.destroy_calls += 1;
        Ok(())
    }
}

#[test]
fn distinct_port_with_same_claimed_id_cannot_create() {
    let mut session = BrowserSession::start(BrowserSessionId::new(17).expect("valid session id"))
        .expect("incarnation capacity");
    let mut approved_port = RecordingPort::new(101, 41, "approved-isolation");
    session
        .create_disposable_context(&mut approved_port)
        .expect("bind approved port");

    let mut spoofing_port = RecordingPort::new(101, 42, "spoofed-isolation");
    assert_eq!(
        session.create_disposable_context(&mut spoofing_port),
        Err(BrowserSessionError::LifecyclePortMismatch)
    );
    assert_eq!(
        spoofing_port.create_calls, 0,
        "distinct adapter with the same self-reported id reached create I/O"
    );
}

#[test]
fn distinct_port_with_same_claimed_id_cannot_destroy() {
    let mut session = BrowserSession::start(BrowserSessionId::new(18).expect("valid session id"))
        .expect("incarnation capacity");
    let mut approved_port = RecordingPort::new(101, 51, "approved-isolation");
    let authority = session
        .create_disposable_context(&mut approved_port)
        .expect("bind approved port");

    let mut spoofing_port = RecordingPort::new(101, 52, "spoofed-isolation");
    assert_eq!(
        session.destroy_disposable_context(&authority, &mut spoofing_port),
        Err(BrowserSessionError::LifecyclePortMismatch)
    );
    assert_eq!(
        spoofing_port.destroy_calls, 0,
        "distinct adapter with the same self-reported id reached destroy I/O"
    );
}
