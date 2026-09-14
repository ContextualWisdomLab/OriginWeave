use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ReusedHandlePort {
    handle: DisposableContextHandle,
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ReusedHandlePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls.set(self.create_calls.get() + 1);
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
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Ok(())
    }
}

#[test]
fn proven_destroy_releases_hot_ownership_without_resurrecting_stale_authority(
) -> Result<(), &'static str> {
    let browsing_context = BrowsingContextId::new(8_100)
        .map_err(|_| "static browsing context id must be valid")?;
    let isolation = DisposableIsolationId::parse("bounded-hot-ownership")
        .map_err(|_| "static isolation id must be valid")?;
    let handle = DisposableContextHandle::new(isolation, browsing_context);
    let create_calls = Rc::new(Cell::new(0));
    let destroy_calls = Rc::new(Cell::new(0));
    let port = ReusedHandlePort {
        handle,
        create_calls: Rc::clone(&create_calls),
        destroy_calls: Rc::clone(&destroy_calls),
    };
    let session = BrowserSession::start(
        BrowserSessionId::new(810).map_err(|_| "static browser session id must be valid")?,
    )
    .map_err(|_| "browser session incarnation must be available")?;
    let mut bound = session.bind_lifecycle_port(port);

    let first = bound
        .create_disposable_context()
        .map_err(|_| "first ownership generation must be accepted")?;
    assert_eq!(first.context_epoch().value(), 1);
    bound
        .destroy_disposable_context(&first)
        .map_err(|_| "first ownership generation must be proven destroyed")?;
    assert_eq!(create_calls.get(), 1);
    assert_eq!(destroy_calls.get(), 1);
    assert_eq!(
        bound.destroy_disposable_context(&first),
        Err(BrowserSessionError::ContextNotOwned),
        "a proven-destroyed authority must fail before another destroy call"
    );
    assert_eq!(destroy_calls.get(), 1);

    let second = bound
        .create_disposable_context()
        .map_err(|_| "proven destruction must release the reusable remote identity from hot ownership")?;
    assert_eq!(second.browsing_context(), first.browsing_context());
    assert_eq!(second.isolation(), first.isolation());
    assert_eq!(second.context_epoch().value(), first.context_epoch().value() + 1);
    assert_eq!(
        bound.destroy_disposable_context(&first),
        Err(BrowserSessionError::AuthorityMismatch),
        "same-valued remote identity reuse must not resurrect the predecessor epoch"
    );
    assert_eq!(
        destroy_calls.get(),
        1,
        "stale authority must fail before lifecycle adapter I/O"
    );
    bound
        .destroy_disposable_context(&second)
        .map_err(|_| "current ownership generation must still authorize exact destruction")?;

    let mut previous_epoch = second.context_epoch().value();
    for _ in 0..256 {
        let current = bound
            .create_disposable_context()
            .map_err(|_| "proven-destroyed identity reuse must remain bounded and admissible")?;
        assert_eq!(current.context_epoch().value(), previous_epoch + 1);
        previous_epoch = current.context_epoch().value();
        bound
            .destroy_disposable_context(&current)
            .map_err(|_| "each current ownership generation must be proven destroyed")?;
    }

    assert_eq!(create_calls.get(), 258);
    assert_eq!(destroy_calls.get(), 258);
    bound
        .end()
        .map_err(|_| "no live or uncertain ownership may remain after proven destruction")?;
    Ok(())
}
