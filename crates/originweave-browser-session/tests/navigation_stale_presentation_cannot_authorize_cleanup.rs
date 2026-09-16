use std::cell::{Cell, RefCell};
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CleanupSeparationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
    destroyed_handles: Rc<RefCell<Vec<DisposableContextHandle>>>,
}

impl DisposableContextPort for CleanupSeparationProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handle
            .take()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.destroyed_handles
            .borrow_mut()
            .push(request.context().clone());
        Ok(())
    }
}

#[test]
fn invalidated_presentation_capability_cannot_be_reused_as_lifecycle_cleanup_authority() {
    let context = BrowsingContextId::new(1161).expect("valid browsing context");
    let isolation = DisposableIsolationId::parse("navigation-cleanup-separation-user-context-1161")
        .expect("valid isolation id");
    let exact_handle = DisposableContextHandle::new(isolation, context);
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let port = CleanupSeparationProbePort {
        handle: Some(exact_handle.clone()),
        adapter_calls: Rc::clone(&adapter_calls),
        destroyed_handles: Rc::clone(&destroyed_handles),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1161).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();

    bound
        .record_observed_navigation(
            pre_navigation.incarnation(),
            context,
            pre_navigation.context_epoch(),
        )
        .expect("observed navigation revokes the presentation capability");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation authority state is aggregate-local and must not perform browser I/O"
    );

    assert_eq!(
        bound.destroy_disposable_context(&pre_navigation),
        Err(BrowserSessionError::AuthorityMismatch),
        "a capability revoked for presentation mutation must not remain a second lifecycle-cleanup authorization path"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "rejecting the stale presentation capability must happen before browser cleanup I/O"
    );
    assert!(
        destroyed_handles.borrow().is_empty(),
        "stale presentation authority must not reach browser.removeUserContext-equivalent cleanup"
    );

    bound
        .destroy_owned_disposable_context(context)
        .expect(
            "the exact bound lifecycle owner still owns cleanup without reopening presentation authority",
        );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[exact_handle],
        "lifecycle cleanup must use the exact browser-issued handle retained by the bound owner"
    );
    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextNotOwned),
        "proven cleanup consumes lifecycle ownership exactly once"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create + 1,
        "duplicate cleanup rejection must remain zero-I/O"
    );

    bound
        .end()
        .expect("proven lifecycle cleanup leaves the reusable Browser Session endable");
}
