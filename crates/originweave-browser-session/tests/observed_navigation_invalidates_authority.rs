use std::cell::Cell;
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

struct NavigationAwarePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for NavigationAwarePort {
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
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }
}

impl AuthorizedContextOperationPort for NavigationAwarePort {
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
) -> originweave_browser_session::BoundBrowserSession<NavigationAwarePort> {
    let port = NavigationAwarePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(isolation).expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(adapter_calls),
    };
    BrowserSession::start(BrowserSessionId::new(session).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn browser_observed_navigation_invalidates_pre_navigation_authority_before_adapter_io() {
    let context = BrowsingContextId::new(901).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(901, context, "navigation-user-context-901", &adapter_calls);

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "pre-navigation"),
        Ok(context)
    );

    let calls_before_navigation = adapter_calls.get();
    bound
        .record_observed_navigation(context)
        .expect("owned context navigation invalidates the prior authority epoch");
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "observed navigation invalidation must not perform adapter I/O"
    );

    bound
        .record_observed_navigation(context)
        .expect("duplicate observation while authority is already invalidated is idempotent");
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "duplicate navigation observation must remain zero-I/O"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-after-navigation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "pre-navigation authority must be rejected before adapter I/O"
    );

    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the exact bound owner explicitly re-establishes authority after invalidation");
    assert_eq!(
        reestablished.context_epoch().value(),
        pre_navigation.context_epoch().value() + 1,
        "duplicate delivery while invalidated must not consume additional epochs"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "post-navigation"),
        Ok(context)
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation + 1);

    let calls_before_second_navigation = adapter_calls.get();
    bound
        .record_observed_navigation(context)
        .expect("a later navigation after explicit re-establishment invalidates the new authority");
    assert_eq!(
        adapter_calls.get(),
        calls_before_second_navigation,
        "a later navigation invalidation must also remain zero-I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "stale-after-second-navigation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_second_navigation,
        "the re-established authority must become stale before adapter I/O on a later navigation"
    );

    let second_reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the exact bound owner can establish a fresh authority for the later document");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        reestablished.context_epoch().value() + 1,
        "a later distinct navigation must consume exactly one new epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-document"),
        Ok(context)
    );
    assert_eq!(adapter_calls.get(), calls_before_second_navigation + 1);
}

#[test]
fn foreign_navigation_observation_is_rejected_without_invalidating_owned_authority_or_adapter_io() {
    let owned = BrowsingContextId::new(911).expect("valid owned context");
    let foreign = BrowsingContextId::new(912).expect("valid foreign context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(911, owned, "navigation-user-context-911", &adapter_calls);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_foreign_observation = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation(foreign),
        Err(BrowserSessionError::ContextNotOwned)
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_foreign_observation,
        "foreign navigation observation must be rejected without adapter I/O"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&authority, "still-current"),
        Ok(owned),
        "foreign observation must not invalidate an unrelated owned context"
    );
    assert_eq!(adapter_calls.get(), calls_before_foreign_observation + 1);
}
