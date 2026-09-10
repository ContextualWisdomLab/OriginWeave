use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct ReusingPort {
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
        Ok(Self {
            handle: DisposableContextHandle::new(isolation, browsing_context),
            create_incarnations: Vec::new(),
            destroy_incarnations: Vec::new(),
        })
    }
}

impl DisposableContextPort for ReusingPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_incarnations.push(request.incarnation());
        Ok(self.handle.clone())
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_incarnations.push(request.incarnation());
        Ok(())
    }
}

/// A retained authority from a completed aggregate must not become valid again after identifier reuse.
#[test]
fn stale_authority_cannot_cross_sequential_session_incarnations() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(701)
        .map_err(|_| "static fixture browser session id must be valid")?;

    let session_a = BrowserSession::start(session_id)
        .map_err(|_| "first browser session incarnation must be available")?;
    let mut bound_a = session_a.bind_lifecycle_port(ReusingPort::new(
        7010,
        "user-context-reused",
    )?);
    let authority_a = bound_a
        .create_disposable_context()
        .map_err(|_| "first disposable context creation must succeed")?;
    bound_a
        .destroy_disposable_context(&authority_a)
        .map_err(|_| "first disposable context destruction must succeed")?;
    bound_a
        .end()
        .map_err(|_| "first browser session must end normally")?;

    let session_b = BrowserSession::start(session_id)
        .map_err(|_| "second browser session incarnation must be available")?;
    let mut bound_b = session_b.bind_lifecycle_port(ReusingPort::new(
        7010,
        "user-context-reused",
    )?);
    let authority_b = bound_b
        .create_disposable_context()
        .map_err(|_| "second disposable context creation must succeed")?;

    assert_ne!(
        bound_a.browser_session().incarnation(),
        bound_b.browser_session().incarnation()
    );
    assert_eq!(
        bound_a.lifecycle_port().create_incarnations,
        vec![bound_a.browser_session().incarnation()]
    );
    assert_eq!(
        bound_b.lifecycle_port().create_incarnations,
        vec![bound_b.browser_session().incarnation()]
    );
    assert_eq!(
        bound_b.destroy_disposable_context(&authority_a),
        Err(BrowserSessionError::AuthorityMismatch)
    );
    assert!(bound_b.lifecycle_port().destroy_incarnations.is_empty());

    bound_b
        .destroy_disposable_context(&authority_b)
        .map_err(|_| "current incarnation authority must remain valid")?;
    assert_eq!(
        bound_b.lifecycle_port().destroy_incarnations,
        vec![bound_b.browser_session().incarnation()]
    );
    Ok(())
}
