use std::cell::{Cell, RefCell};
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CleanupProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
    destroyed_handles: Rc<RefCell<Vec<DisposableContextHandle>>>,
}

impl DisposableContextPort for CleanupProbePort {
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

impl AuthorizedContextOperationPort for CleanupProbePort {
    type Operation = &'static str;
    type Output = BrowsingContextId;
    type Error = ();

    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(request.context().browsing_context())
    }
}

fn bound_session(
    session: u64,
    context: BrowsingContextId,
    isolation: &str,
    adapter_calls: &Rc<Cell<usize>>,
    destroyed_handles: &Rc<RefCell<Vec<DisposableContextHandle>>>,
) -> originweave_browser_session::BoundBrowserSession<CleanupProbePort> {
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    );
    let port = CleanupProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(adapter_calls),
        destroyed_handles: Rc::clone(destroyed_handles),
    };
    BrowserSession::start(BrowserSessionId::new(session).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

fn expected_handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

#[test]
fn navigation_invalidation_does_not_strand_owned_disposable_cleanup() {
    let context = BrowsingContextId::new(921).expect("valid browsing context");
    let isolation = "navigation-cleanup-user-context-921";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        921,
        context,
        isolation,
        &adapter_calls,
        &destroyed_handles,
    );

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "pre-navigation"),
        Ok(context)
    );

    bound
        .record_observed_navigation(context)
        .expect("observed navigation invalidates presentation authority");
    let calls_after_navigation = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-presentation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_navigation,
        "stale presentation authority must fail before adapter I/O"
    );

    bound
        .destroy_owned_disposable_context(context)
        .expect("the exact bound lifecycle owner must destroy its isolation without presentation re-establishment");
    assert_eq!(
        adapter_calls.get(),
        calls_after_navigation + 1,
        "lifecycle destruction must call the bound port exactly once"
    );
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(context, isolation)],
        "destruction must use the exact browser-issued handle retained at creation"
    );

    let calls_after_destroy = adapter_calls.get();
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "proven destruction must consume lifecycle ownership and cannot reopen presentation authority"
    );
    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextNotOwned),
        "proven destruction must not permit a second remote cleanup attempt from the same raw selector"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy re-establishment and duplicate cleanup must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves no owned disposable context behind");
}

#[test]
fn later_navigation_after_reestablishment_still_allows_lifecycle_cleanup_without_reissuing_presentation_authority() {
    let context = BrowsingContextId::new(931).expect("valid browsing context");
    let isolation = "navigation-cleanup-user-context-931";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        931,
        context,
        isolation,
        &adapter_calls,
        &destroyed_handles,
    );

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .record_observed_navigation(context)
        .expect("first navigation invalidates initial presentation authority");
    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("owner explicitly establishes authority for the next document");
    assert_eq!(
        reestablished.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    bound
        .record_observed_navigation(context)
        .expect("later navigation invalidates the re-established presentation authority");

    let calls_before_destroy = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "stale-second-document"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(adapter_calls.get(), calls_before_destroy);

    bound
        .destroy_owned_disposable_context(context)
        .expect("lifecycle ownership survives presentation invalidation across later navigation");
    assert_eq!(adapter_calls.get(), calls_before_destroy + 1);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(context, isolation)]
    );
    bound.end().expect("destroyed session can end normally");
}

#[test]
fn raw_foreign_context_cannot_select_cleanup_outside_bound_lifecycle_ownership_after_navigation_invalidation() {
    let owned = BrowsingContextId::new(941).expect("valid owned context");
    let foreign = BrowsingContextId::new(942).expect("valid foreign context");
    let isolation = "navigation-cleanup-user-context-941";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        941,
        owned,
        isolation,
        &adapter_calls,
        &destroyed_handles,
    );

    bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_navigation = adapter_calls.get();
    bound
        .record_observed_navigation(owned)
        .expect("owned navigation invalidates presentation authority before cleanup selection");
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "navigation invalidation must not perform adapter I/O"
    );

    let calls_before_foreign_destroy = adapter_calls.get();
    assert_eq!(
        bound.destroy_owned_disposable_context(foreign),
        Err(BrowserSessionError::ContextNotOwned),
        "navigation invalidation must not let a raw foreign browser identifier manufacture cleanup authority"
    );
    assert_eq!(adapter_calls.get(), calls_before_foreign_destroy);
    assert!(destroyed_handles.borrow().is_empty());

    bound
        .destroy_owned_disposable_context(owned)
        .expect("the same bound owner can destroy its exact retained handle while presentation remains invalidated");
    assert_eq!(adapter_calls.get(), calls_before_foreign_destroy + 1);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(owned, isolation)]
    );
}
