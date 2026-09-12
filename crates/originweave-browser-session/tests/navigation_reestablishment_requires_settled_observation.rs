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

struct NavigationSettlementProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for NavigationSettlementProbePort {
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

impl AuthorizedContextOperationPort for NavigationSettlementProbePort {
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
fn navigation_start_cannot_reissue_presentation_authority_before_settled_browser_observation() {
    let context = BrowsingContextId::new(981).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = NavigationSettlementProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("navigation-settlement-user-context-981")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(981).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_navigation = adapter_calls.get();

    let settlement_authority = bound
        .record_observed_navigation(
            pre_navigation.incarnation(),
            context,
            pre_navigation.context_epoch(),
        )
        .expect(
            "navigation start invalidates presentation authority and issues settlement authority",
        );
    assert_eq!(adapter_calls.get(), calls_before_navigation);
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-while-navigation-pending"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        ))
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation);

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "a navigation-start observation alone must not authorize presentation re-establishment while the browser transition is still unsettled"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "pending-navigation re-establishment must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_settled(&settlement_authority)
        .expect(
            "adapter-qualified commit or fragment completion presents the exact aggregate-issued settlement authority",
        );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "settling browser observation is a zero-I/O Browser Session transition"
    );

    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the bound owner may re-establish only after the matching navigation is settled");
    assert_eq!(
        reestablished.context_epoch().value(),
        pre_navigation.context_epoch().value() + 1,
        "settlement itself must not consume an authority epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "current-after-settlement"),
        Ok(context)
    );

    let calls_before_stale_settlement_replay = adapter_calls.get();
    assert_eq!(
        bound.record_observed_navigation_settled(&settlement_authority),
        Err(BrowserSessionError::AuthorityMismatch),
        "a delayed settlement witness from the prior generation must not alter the newly established generation"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_stale_settlement_replay,
        "stale settlement replay must fail before adapter I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &reestablished,
            "still-current-after-stale-settlement",
        ),
        Ok(context),
        "rejecting stale settlement must leave current authority usable"
    );

    let calls_before_later_navigation = adapter_calls.get();
    let later_settlement_authority = bound
        .record_observed_navigation(
            reestablished.incarnation(),
            context,
            reestablished.context_epoch(),
        )
        .expect("later navigation start invalidates the current presentation generation");
    assert_eq!(adapter_calls.get(), calls_before_later_navigation);

    assert_eq!(
        bound.record_observed_navigation_settled(&settlement_authority),
        Err(BrowserSessionError::AuthorityMismatch),
        "the prior generation's consumed settlement witness must stay dead while a later navigation is pending"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &settlement_authority,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "the prior generation's witness must not terminate a later pending navigation through the negative path"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_later_navigation,
        "stale positive and negative terminal evidence must fail before adapter I/O"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "every later navigation must remain closed until its own matching browser settlement"
    );
    assert_eq!(adapter_calls.get(), calls_before_later_navigation);

    bound
        .record_observed_navigation_settled(&later_settlement_authority)
        .expect("later matching navigation settlement presents its aggregate-issued witness");
    let second_reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the later settled document can receive a fresh authority");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        reestablished.context_epoch().value() + 1
    );
}
