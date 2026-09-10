use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateError,
    DisposableContextDestroyError, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct ReusingPort {
    handle: DisposableContextHandle,
    destroy_calls: usize,
}

impl ReusingPort {
    fn new(context: u64, isolation: &str) -> Result<Self, &'static str> {
        let isolation = DisposableIsolationId::parse(isolation)
            .map_err(|_| "static fixture isolation id must be valid")?;
        let browsing_context = BrowsingContextId::new(context)
            .map_err(|_| "static fixture browsing context id must be valid")?;
        Ok(Self {
            handle: DisposableContextHandle::new(isolation, browsing_context),
            destroy_calls: 0,
        })
    }
}

impl DisposableContextPort for ReusingPort {
    fn create_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        Ok(self.handle.clone())
    }

    fn destroy_disposable_context(
        &mut self,
        _browser_session: BrowserSessionId,
        _context: &DisposableContextHandle,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_calls += 1;
        Ok(())
    }
}

/// A retained authority from a completed aggregate must not become valid again after identifier reuse.
#[test]
fn stale_authority_cannot_cross_sequential_session_incarnations() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(701)
        .map_err(|_| "static fixture browser session id must be valid")?;

    let mut port_a = ReusingPort::new(7010, "user-context-reused")?;
    let mut session_a = BrowserSession::start(session_id);
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
    let mut session_b = BrowserSession::start(session_id);
    let authority_b = session_b
        .create_disposable_context(&mut port_b)
        .map_err(|_| "second disposable context creation must succeed")?;

    assert_eq!(
        session_b.destroy_disposable_context(&authority_a, &mut port_b),
        Err(BrowserSessionError::AuthorityMismatch)
    );
    assert_eq!(port_b.destroy_calls, 0);

    session_b
        .destroy_disposable_context(&authority_b, &mut port_b)
        .map_err(|_| "current incarnation authority must remain valid")?;
    assert_eq!(port_b.destroy_calls, 1);
    Ok(())
}
