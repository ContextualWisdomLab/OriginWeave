use std::cell::RefCell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct ReusingPort {
    handle: DisposableContextHandle,
    create_incarnations: Rc<RefCell<Vec<BrowserSessionIncarnation>>>,
    destroy_incarnations: Rc<RefCell<Vec<BrowserSessionIncarnation>>>,
}

impl ReusingPort {
    fn new(
        context: u64,
        isolation: &str,
        create_incarnations: Rc<RefCell<Vec<BrowserSessionIncarnation>>>,
        destroy_incarnations: Rc<RefCell<Vec<BrowserSessionIncarnation>>>,
    ) -> Result<Self, &'static str> {
        let isolation = DisposableIsolationId::parse(isolation)
            .map_err(|_| "static fixture isolation id must be valid")?;
        let browsing_context = BrowsingContextId::new(context)
            .map_err(|_| "static fixture browsing context id must be valid")?;
        Ok(Self {
            handle: DisposableContextHandle::new(isolation, browsing_context),
            create_incarnations,
            destroy_incarnations,
        })
    }
}

impl DisposableContextPort for ReusingPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_incarnations
            .borrow_mut()
            .push(request.incarnation());
        Ok(self.handle.clone())
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_incarnations
            .borrow_mut()
            .push(request.incarnation());
        Ok(())
    }
}

/// A retained authority from a completed aggregate must not become valid again after identifier reuse.
#[test]
fn stale_authority_cannot_cross_sequential_session_incarnations() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(701)
        .map_err(|_| "static fixture browser session id must be valid")?;

    let create_a = Rc::new(RefCell::new(Vec::new()));
    let destroy_a = Rc::new(RefCell::new(Vec::new()));
    let session_a = BrowserSession::start(session_id)
        .map_err(|_| "first browser session incarnation must be available")?;
    let mut bound_a = session_a.bind_lifecycle_port(ReusingPort::new(
        7010,
        "user-context-reused",
        Rc::clone(&create_a),
        Rc::clone(&destroy_a),
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

    let create_b = Rc::new(RefCell::new(Vec::new()));
    let destroy_b = Rc::new(RefCell::new(Vec::new()));
    let session_b = BrowserSession::start(session_id)
        .map_err(|_| "second browser session incarnation must be available")?;
    let mut bound_b = session_b.bind_lifecycle_port(ReusingPort::new(
        7010,
        "user-context-reused",
        Rc::clone(&create_b),
        Rc::clone(&destroy_b),
    )?);
    let authority_b = bound_b
        .create_disposable_context()
        .map_err(|_| "second disposable context creation must succeed")?;

    assert_ne!(
        bound_a.browser_session().incarnation(),
        bound_b.browser_session().incarnation()
    );
    assert_eq!(
        create_a.borrow().as_slice(),
        &[bound_a.browser_session().incarnation()]
    );
    assert_eq!(
        create_b.borrow().as_slice(),
        &[bound_b.browser_session().incarnation()]
    );
    assert_eq!(
        bound_b.destroy_disposable_context(&authority_a),
        Err(BrowserSessionError::AuthorityMismatch)
    );
    assert!(destroy_b.borrow().is_empty());

    bound_b
        .destroy_disposable_context(&authority_b)
        .map_err(|_| "current incarnation authority must remain valid")?;
    assert_eq!(
        destroy_b.borrow().as_slice(),
        &[bound_b.browser_session().incarnation()]
    );
    Ok(())
}
