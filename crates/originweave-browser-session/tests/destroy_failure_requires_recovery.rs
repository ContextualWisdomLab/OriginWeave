use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextHandle,
    DisposableContextPort, DisposableContextPortError, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct FailingDestroyPort {
    next_handle: DisposableContextHandle,
    create_calls: usize,
    destroy_calls: usize,
}

impl FailingDestroyPort {
    fn new(context: u64, isolation: &str) -> Self {
        Self {
            next_handle: DisposableContextHandle::new(
                DisposableIsolationId::parse(isolation).expect("valid isolation id"),
                BrowsingContextId::new(context).expect("valid context id"),
            ),
            create_calls: 0,
            destroy_calls: 0,
        }
    }
}

impl DisposableContextPort for FailingDestroyPort {
    fn create_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
    ) -> Result<DisposableContextHandle, DisposableContextPortError> {
        self.create_calls += 1;
        Ok(self.next_handle.clone())
    }

    fn destroy_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
        _context: &DisposableContextHandle,
    ) -> Result<(), DisposableContextPortError> {
        self.destroy_calls += 1;
        Err(DisposableContextPortError::DestroyFailed)
    }
}

/// An unproven destroy must quarantine the whole aggregate before any later browser I/O.
#[test]
fn destroy_failure_requires_recovery_before_any_new_authority() {
    let session_id = BrowserSessionId::new(501).expect("valid session id");
    let context_id = BrowsingContextId::new(5010).expect("valid context id");
    let mut session = BrowserSession::start(session_id);
    let mut failing_port = FailingDestroyPort::new(5010, "user-context-501");

    let authority = session
        .create_disposable_context(&mut failing_port)
        .expect("owned disposable context");
    assert_eq!(
        session.destroy_disposable_context(&authority, &mut failing_port),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(failing_port.destroy_calls, 1);
    assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);

    let mut later_port = FailingDestroyPort::new(5011, "user-context-501-later");
    assert_eq!(
        session.create_disposable_context(&mut later_port),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(later_port.create_calls, 0);
    assert_eq!(
        session.presentation_authority(context_id),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(
        session.advance_context_epoch(context_id),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
}
