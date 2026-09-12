use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationPort, AuthorizedContextOperationRequest, BrowserSession,
    BrowserSessionError, DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct NavigationSettlementAuthorityProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for NavigationSettlementAuthorityProbePort {
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

impl AuthorizedContextOperationPort for NavigationSettlementAuthorityProbePort {
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
) -> originweave_browser_session::BoundBrowserSession<NavigationSettlementAuthorityProbePort> {
    let port = NavigationSettlementAuthorityProbePort {
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
fn raw_navigation_provenance_cannot_be_replayed_as_settlement_authority() {
    let reused_session = BrowserSessionId::new(1001).expect("valid session id");
    let reused_context = BrowsingContextId::new(1001).expect("valid browsing context");
    let first_adapter_calls = Rc::new(Cell::new(0));
    let second_adapter_calls = Rc::new(Cell::new(0));

    let mut first = bound_session(
        reused_session,
        reused_context,
        "navigation-settlement-authority-first",
        &first_adapter_calls,
    );
    let mut second = bound_session(
        reused_session,
        reused_context,
        "navigation-settlement-authority-second",
        &second_adapter_calls,
    );

    let first_presentation = first
        .create_disposable_context()
        .expect("first aggregate accepts its context");
    let second_presentation = second
        .create_disposable_context()
        .expect("second aggregate accepts its context");

    assert_eq!(
        first_presentation.context_epoch(),
        second_presentation.context_epoch(),
        "independent aggregates may allocate the same local epoch value"
    );
    assert_ne!(
        first_presentation.incarnation(),
        second_presentation.incarnation(),
        "aggregate incarnation disambiguates reused browser identifiers"
    );

    let first_settlement_authority = first
        .record_observed_navigation(
            first_presentation.incarnation(),
            reused_context,
            first_presentation.context_epoch(),
        )
        .expect("admitted navigation start issues an aggregate-bound settlement authority");
    let second_settlement_authority = second
        .record_observed_navigation(
            second_presentation.incarnation(),
            reused_context,
            second_presentation.context_epoch(),
        )
        .expect("second aggregate issues its own settlement authority");

    let calls_before_cross_aggregate_settlement = second_adapter_calls.get();
    assert_eq!(
        second.record_observed_navigation_settled(&first_settlement_authority),
        Err(BrowserSessionError::AuthorityMismatch),
        "raw session/context/epoch aliasing must not let another aggregate's navigation witness unlock settlement"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_cross_aggregate_settlement,
        "foreign settlement authority must fail before adapter I/O"
    );
    assert_eq!(
        second.reestablish_presentation_authority(reused_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting a foreign settlement authority must leave the current navigation pending"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_cross_aggregate_settlement
    );

    second
        .record_observed_navigation_settled(&second_settlement_authority)
        .expect(
            "only the Browser Session-issued authority for this pending navigation may settle it",
        );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_cross_aggregate_settlement,
        "settlement remains a zero-I/O domain transition"
    );

    let second_reestablished = second
        .reestablish_presentation_authority(reused_context)
        .expect("exact settlement unlocks explicit presentation-authority re-establishment");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        second_presentation.context_epoch().value() + 1
    );

    assert_eq!(
        first.record_observed_navigation_settled(&first_settlement_authority),
        Ok(()),
        "the first aggregate retains authority over its own pending navigation"
    );
}
