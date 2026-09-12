use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct OverlappingNavigationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for OverlappingNavigationProbePort {
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

impl AuthorizedContextOperationPort for OverlappingNavigationProbePort {
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

#[test]
fn newer_navigation_start_supersedes_prior_pending_witness_until_its_own_terminal_event() {
    let context = BrowsingContextId::new(1041).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = OverlappingNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("overlapping-navigation-user-context-1041")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1041).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first qualified navigation start issues a pending witness");
    assert_eq!(adapter_calls.get(), calls_after_create);

    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("a distinct later navigation start may supersede an earlier still-pending navigation");
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "a late negative terminal event for a superseded navigation must not close the newer pending generation"
    );
    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a late positive terminal event for a superseded navigation must not settle the newer pending generation"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "superseded terminal evidence must not reopen authority while the newer navigation is pending"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation state transitions and superseded-witness rejection must remain zero-I/O"
    );

    bound
        .record_observed_navigation_settled(&second_pending)
        .expect("only the current pending navigation may settle the presentation generation");
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a superseded witness remains dead after the newer navigation settles"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("current terminal observation permits explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "overlapping navigation starts must not themselves mint extra presentation epochs"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-before-overlap"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the pre-navigation authority stays stale across both overlapping navigations"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "fresh-after-current-terminal"),
        Ok(context)
    );
}
