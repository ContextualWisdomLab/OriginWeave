use originweave_browser_session::{
    BrowserSession, BrowserSessionIncarnation, DisposableContextCreateError,
    DisposableContextDestroyError, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct RecordingPort {
    create_calls: usize,
    destroy_calls: usize,
}

impl DisposableContextPort for RecordingPort {
    fn create_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
        _incarnation: BrowserSessionIncarnation,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls += 1;
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("raw-port-side-door").expect("valid isolation id"),
            BrowsingContextId::new(41).expect("valid browsing context"),
        ))
    }

    fn destroy_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
        _incarnation: BrowserSessionIncarnation,
        _context: &DisposableContextHandle,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_calls += 1;
        Ok(())
    }
}

#[test]
fn raw_session_identity_cannot_directly_authorize_create_or_destroy() {
    let session = BrowserSession::start(BrowserSessionId::new(7).expect("valid session id"))
        .expect("incarnation capacity");
    let mut port = RecordingPort {
        create_calls: 0,
        destroy_calls: 0,
    };

    let direct_create = DisposableContextPort::create_disposable_context(
        &mut port,
        session.id(),
        session.incarnation(),
    );
    assert!(
        direct_create.is_err(),
        "raw session/incarnation values must not be sufficient lifecycle authority"
    );
    assert_eq!(port.create_calls, 0, "unauthorized create reached adapter I/O");

    let forged_handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("raw-port-side-door").expect("valid isolation id"),
        BrowsingContextId::new(41).expect("valid browsing context"),
    );
    let direct_destroy = DisposableContextPort::destroy_disposable_context(
        &mut port,
        session.id(),
        session.incarnation(),
        &forged_handle,
    );
    assert!(
        direct_destroy.is_err(),
        "raw lifecycle tuple must not be sufficient destruction authority"
    );
    assert_eq!(port.destroy_calls, 0, "unauthorized destroy reached adapter I/O");
}
