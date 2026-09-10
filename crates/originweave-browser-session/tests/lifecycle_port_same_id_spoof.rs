use originweave_browser_session::{
    BrowserSession, DisposableContextCreateError, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct RecordingPort {
    context: BrowsingContextId,
    isolation: &'static str,
    create_calls: usize,
    destroy_calls: usize,
}

impl RecordingPort {
    fn new(context: u64, isolation: &'static str) -> Self {
        Self {
            context: BrowsingContextId::new(context).expect("valid browsing context"),
            isolation,
            create_calls: 0,
            destroy_calls: 0,
        }
    }
}

impl DisposableContextPort for RecordingPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls += 1;
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse(self.isolation).expect("valid isolation id"),
            self.context,
        ))
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_calls += 1;
        Ok(())
    }
}

#[test]
fn distinct_adapter_cannot_be_substituted_for_create_after_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(17).expect("valid session id"))
        .expect("incarnation capacity");
    let approved_port = RecordingPort::new(41, "approved-isolation");
    let spoofing_port = RecordingPort::new(42, "spoofed-isolation");
    let mut bound = session.bind_lifecycle_port(approved_port);

    bound
        .create_disposable_context()
        .expect("bound adapter creates context");
    assert_eq!(bound.lifecycle_port().create_calls, 1);
    assert_eq!(spoofing_port.create_calls, 0);
}

#[test]
fn distinct_adapter_cannot_be_substituted_for_destroy_after_binding() {
    let session = BrowserSession::start(BrowserSessionId::new(18).expect("valid session id"))
        .expect("incarnation capacity");
    let approved_port = RecordingPort::new(51, "approved-isolation");
    let spoofing_port = RecordingPort::new(52, "spoofed-isolation");
    let mut bound = session.bind_lifecycle_port(approved_port);
    let authority = bound
        .create_disposable_context()
        .expect("bound adapter creates context");

    bound
        .destroy_disposable_context(&authority)
        .expect("bound adapter destroys context");
    assert_eq!(bound.lifecycle_port().destroy_calls, 1);
    assert_eq!(spoofing_port.destroy_calls, 0);
}
