use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableContextPortId, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct ReusingPort {
    port_id: DisposableContextPortId,
    handle: DisposableContextHandle,
    create_incarnations: Vec<BrowserSessionIncarnation>,
    destroy_incarnations: Vec<BrowserSessionIncarnation>,
}

impl ReusingPort {
    fn new(context: u64, isolation: &str) -> Result<Self, &'static str> {
        let isolation = DisposableIsolationId::parse(isolation)
            .map_err(|_| "static fixture isolation id must be valid")?;
        let browsing_context = BrowsingContextId::new(context)
            .map_err(|_| "static fixture browsing context id must be valid")?;
        let port_id = DisposableContextPortId::new(context)
            .ok_or("static fixture lifecycle port id must be non-zero")?;
        Ok(Self {
            port_id,
            handle: DisposableContextHandle::new(isolation, browsing_context),
            create_incarnations: Vec::new(),
            destroy_incarnations: Vec::new(),
        })
    }
}

impl DisposableContextPort for ReusingPort {
    fn port_id(&self) -> DisposableContextPortId {
        self.port_id
    }

    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        assert_eq!(request.port_id(), self.port_id);
        self.create_incarnations.push(request.incarnation());
        Ok(self.handle.clone())
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        assert_eq!(request.port_id(), self.port_id);
        self.destroy_incarnations.push(request.incarnation());
        Ok(())
    }
}

/// A retained authority from a completed aggregate must not become valid again after identifier reuse.
#[test]
fn stale_authority_cannot_cross_sequential_session_incarnations() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(701)
        .map_err(|_| "static fixture browser session id must be valid")?;

    let mut port_a = ReusingPort::new(7010, "user-context-reused")?;
    let mut session_a = BrowserSession::start(session_id)
        .map_err(|_| "first browser session incarnation must be available")?;
    let authority_a = session_a
        .create_disposable_context(&mut port_a)
        .map_err(|_| "first disposable context creation must succeed")?;
    session_a
        .destroy_disposable_context(&authority_a, &mut port_a)
        .map_err(|_| "first disposable context destruction must succeed")?;
    session_a
        .end()
        .map_err(|_| "first browser session must end normally")?;

    let mut port_b = ReusingPort::new(7010, "user-context-reused")?;
    let mut session_b = BrowserSession::start(session_id)
        .map_err(|_| "second browser session incarnation must be available")?;
    let authority_b = session_b
        .create_disposable_context(&mut port_b)
        .map_err(|_| "second disposable context creation must succeed")?;

    assert_ne!(session_a.incarnation(), session_b.incarnation());
    assert_eq!(port_a.create_incarnations, vec![session_a.incarnation()]);
    assert_eq!(port_b.create_incarnations, vec![session_b.incarnation()]);
    assert_eq!(
        session_b.destroy_disposable_context(&authority_a, &mut port_b),
        Err(BrowserSessionError::AuthorityMismatch)
    );
    assert!(port_b.destroy_incarnations.is_empty());

    session_b
        .destroy_disposable_context(&authority_b, &mut port_b)
        .map_err(|_| "current incarnation authority must remain valid")?;
    assert_eq!(port_b.destroy_incarnations, vec![session_b.incarnation()]);
    Ok(())
}
