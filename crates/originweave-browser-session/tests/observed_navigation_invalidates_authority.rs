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
    operation_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for NavigationAwarePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.handle
            .take()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
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
        self.operation_calls.set(self.operation_calls.get() + 1);
        Ok(request.context().browsing_context())
    }
}

#[test]
fn browser_observed_navigation_invalidates_pre_navigation_authority_before_adapter_io() {
    let context = BrowsingContextId::new(901).expect("valid browsing context");
    let operation_calls = Rc::new(Cell::new(0));
    let port = NavigationAwarePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("navigation-user-context-901")
                .expect("valid isolation id"),
            context,
        )),
        operation_calls: Rc::clone(&operation_calls),
    };
    let session = BrowserSession::start(BrowserSessionId::new(901).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port);

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "pre-navigation"),
        Ok(context)
    );
    assert_eq!(operation_calls.get(), 1);

    bound
        .record_observed_navigation(context)
        .expect("owned context navigation invalidates the prior authority epoch");

    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-after-navigation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(
        operation_calls.get(),
        1,
        "pre-navigation authority must be rejected before adapter I/O"
    );

    let reestablished = bound
        .presentation_authority(context)
        .expect("owner explicitly re-establishes authority after navigation invalidation");
    assert_ne!(
        reestablished.context_epoch(),
        pre_navigation.context_epoch(),
        "observed navigation must rotate the context epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "post-navigation"),
        Ok(context)
    );
    assert_eq!(operation_calls.get(), 2);
}
