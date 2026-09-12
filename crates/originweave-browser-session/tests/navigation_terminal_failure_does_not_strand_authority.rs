use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationSettlementOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct TerminalNavigationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for TerminalNavigationProbePort {
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

impl AuthorizedContextOperationPort for TerminalNavigationProbePort {
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
fn failed_or_aborted_navigation_closes_pending_state_without_silently_minting_authority() {
    let context = BrowsingContextId::new(1011).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = TerminalNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("terminal-navigation-user-context-1011")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1011).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_failure = adapter_calls.get();
    let failed_navigation = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start invalidates the current presentation generation");
    assert_eq!(adapter_calls.get(), calls_before_failure);
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-during-failed-navigation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        ))
    );
    assert_eq!(adapter_calls.get(), calls_before_failure);

    bound
        .record_observed_navigation_settled(
            &failed_navigation,
            NavigationSettlementOutcome::Failed,
        )
        .expect("matching navigationFailed terminates the pending navigation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_failure,
        "terminal failure observation must remain a zero-I/O Browser Session transition"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "still-stale-after-failure"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "navigationFailed must not silently revive pre-navigation presentation authority"
    );
    assert_eq!(adapter_calls.get(), calls_before_failure);

    let after_failure = bound
        .reestablish_presentation_authority(context)
        .expect("a terminal failure must not strand a reusable owned context after explicit re-establishment");
    assert_eq!(
        after_failure.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_failure, "usable-after-failure"),
        Ok(context)
    );

    let calls_before_stale_failure_replay = adapter_calls.get();
    assert_eq!(
        bound.record_observed_navigation_settled(
            &failed_navigation,
            NavigationSettlementOutcome::Committed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "one terminal outcome consumes the exact pending witness; a conflicting replay cannot rewrite it"
    );
    assert_eq!(adapter_calls.get(), calls_before_stale_failure_replay);
    assert_eq!(
        bound.execute_authorized_context_operation(&after_failure, "still-usable-after-stale-failure-replay"),
        Ok(context)
    );

    let calls_before_abort = adapter_calls.get();
    let aborted_navigation = bound
        .record_observed_navigation(
            after_failure.incarnation(),
            context,
            after_failure.context_epoch(),
        )
        .expect("later navigation enters pending state");
    assert_eq!(adapter_calls.get(), calls_before_abort);
    bound
        .record_observed_navigation_settled(
            &aborted_navigation,
            NavigationSettlementOutcome::Aborted,
        )
        .expect("matching navigationAborted terminates the later pending navigation");
    assert_eq!(adapter_calls.get(), calls_before_abort);
    assert_eq!(
        bound.execute_authorized_context_operation(&after_failure, "stale-after-abort"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "navigationAborted must not silently restore the pre-navigation authority"
    );
    assert_eq!(adapter_calls.get(), calls_before_abort);

    let after_abort = bound
        .reestablish_presentation_authority(context)
        .expect("a terminal abort must close pending state without permanently denying the owned context");
    assert_eq!(
        after_abort.context_epoch().value(),
        after_failure.context_epoch().value() + 1
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_abort, "usable-after-abort"),
        Ok(context)
    );
}
