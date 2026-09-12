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

struct CrossSessionNavigationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CrossSessionNavigationProbePort {
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

impl AuthorizedContextOperationPort for CrossSessionNavigationProbePort {
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
    session: BrowserSessionId,
    context: BrowsingContextId,
    isolation: &str,
    adapter_calls: &Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<CrossSessionNavigationProbePort> {
    let port = CrossSessionNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(isolation).expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(adapter_calls),
    };
    BrowserSession::start(session)
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn navigation_generation_from_prior_session_incarnation_cannot_revoke_current_session() {
    let reused_session = BrowserSessionId::new(971).expect("valid session id");
    let reused_context = BrowsingContextId::new(971).expect("valid browsing context");
    let first_adapter_calls = Rc::new(Cell::new(0));
    let second_adapter_calls = Rc::new(Cell::new(0));

    let mut first = bound_session(
        reused_session,
        reused_context,
        "cross-session-navigation-user-context-first",
        &first_adapter_calls,
    );
    let mut second = bound_session(
        reused_session,
        reused_context,
        "cross-session-navigation-user-context-second",
        &second_adapter_calls,
    );

    let first_authority = first
        .create_disposable_context()
        .expect("first session accepts its disposable context");
    let second_authority = second
        .create_disposable_context()
        .expect("second session accepts its disposable context");

    assert_eq!(
        first_authority.context_epoch(),
        second_authority.context_epoch(),
        "independent Browser Session aggregates may legitimately allocate the same local epoch value"
    );
    assert_ne!(
        first_authority.incarnation(),
        second_authority.incarnation(),
        "process-local Browser Session incarnations must disambiguate reused transport/context identities"
    );

    let calls_before_stale_session_observation = second_adapter_calls.get();
    assert!(
        matches!(
            second.record_observed_navigation(
                first_authority.incarnation(),
                reused_context,
                first_authority.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        ),
        "a buffered navigation correlated to a prior aggregate incarnation must not revoke the current aggregate even when raw session, context, and epoch values alias"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_stale_session_observation,
        "cross-session generation confusion must fail before adapter I/O"
    );
    assert_eq!(
        second.execute_authorized_context_operation(&second_authority, "second-still-current"),
        Ok(reused_context),
        "rejecting prior-incarnation provenance must leave current presentation authority usable"
    );

    let calls_before_current_navigation = second_adapter_calls.get();
    let _second_settlement_authority = second
        .record_observed_navigation(
            second_authority.incarnation(),
            reused_context,
            second_authority.context_epoch(),
        )
        .expect("the exact current aggregate generation may invalidate its own authority");
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_current_navigation,
        "valid navigation invalidation remains a zero-I/O domain transition"
    );
    assert_eq!(
        second.execute_authorized_context_operation(&second_authority, "second-now-stale"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the exact current-session observation invalidates only the matching aggregate generation"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_current_navigation,
        "stale current-session authority must be rejected before adapter I/O"
    );

    assert_eq!(
        first.execute_authorized_context_operation(&first_authority, "first-remains-current"),
        Ok(reused_context),
        "navigation observed for the second aggregate must not mutate the independent first aggregate"
    );
}
