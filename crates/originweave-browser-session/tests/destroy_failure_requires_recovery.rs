use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableContextPortId, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct FailingDestroyPort {
    port_id: DisposableContextPortId,
    next_handle: DisposableContextHandle,
    create_calls: usize,
    destroy_calls: usize,
}

impl FailingDestroyPort {
    fn new(context: u64, isolation: &str) -> Result<Self, &'static str> {
        let isolation = DisposableIsolationId::parse(isolation)
            .map_err(|_| "static fixture isolation id must be valid")?;
        let browsing_context = BrowsingContextId::new(context)
            .map_err(|_| "static fixture browsing context id must be valid")?;
        let port_id = DisposableContextPortId::new(context)
            .ok_or("static fixture lifecycle port id must be non-zero")?;
        Ok(Self {
            port_id,
            next_handle: DisposableContextHandle::new(isolation, browsing_context),
            create_calls: 0,
            destroy_calls: 0,
        })
    }
}

impl DisposableContextPort for FailingDestroyPort {
    fn port_id(&self) -> DisposableContextPortId {
        self.port_id
    }

    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        assert_eq!(request.port_id(), self.port_id);
        self.create_calls += 1;
        Ok(self.next_handle.clone())
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        assert_eq!(request.port_id(), self.port_id);
        self.destroy_calls += 1;
        Err(DisposableContextDestroyError::DestroyFailed)
    }
}

/// An unproven destroy must retain exact recovery evidence and reject later normal authority.
#[test]
fn destroy_failure_requires_recovery_before_any_new_authority() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(501)
        .map_err(|_| "static fixture browser session id must be valid")?;
    let context_id = BrowsingContextId::new(5010)
        .map_err(|_| "static fixture browsing context id must be valid")?;
    let expected_isolation = DisposableIsolationId::parse("user-context-501")
        .map_err(|_| "static fixture recovery isolation id must be valid")?;
    let expected_handle = DisposableContextHandle::new(expected_isolation, context_id);
    let mut session = BrowserSession::start(session_id)
        .map_err(|_| "browser session incarnation must be available")?;
    let mut failing_port = FailingDestroyPort::new(5010, "user-context-501")?;

    let authority = session
        .create_disposable_context(&mut failing_port)
        .map_err(|_| "fixture disposable context creation must succeed")?;
    assert_eq!(
        session.destroy_disposable_context(&authority, &mut failing_port),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(failing_port.destroy_calls, 1);
    assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(
        session.recovery_evidence(),
        &[BrowserSessionRecoveryEvidence::UnprovenDestruction(
            expected_handle
        )]
    );
    assert!(!session.transport_is_lost());

    assert!(session.record_transport_loss());
    assert!(session.transport_is_lost());
    assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);
    assert!(!session.record_transport_loss());

    let mut later_port = FailingDestroyPort::new(5011, "user-context-501-later")?;
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
    Ok(())
}
