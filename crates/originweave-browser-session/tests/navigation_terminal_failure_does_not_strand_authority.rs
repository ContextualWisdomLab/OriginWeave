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
        .record_observed_navigation_terminated(
            &failed_navigation,
            NavigationTerminationOutcome::Failed,
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
    assert_eq!(
        bound.advance_context_epoch(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationFailed must not let generic epoch rotation bypass explicit presentation re-establishment"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_failure,
        "generic rotation after navigationFailed must fail before adapter I/O"
    );

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
        bound.record_observed_navigation_terminated(
            &failed_navigation,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "one terminal outcome consumes the exact pending witness; replay cannot rewrite a newer generation"
    );
    assert_eq!(adapter_calls.get(), calls_before_stale_failure_replay);
    assert_eq!(
        bound.execute_authorized_context_operation(
            &after_failure,
            "still-usable-after-stale-failure-replay"
        ),
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
        .record_observed_navigation_terminated(
            &aborted_navigation,
            NavigationTerminationOutcome::Aborted,
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
    assert_eq!(
        bound.advance_context_epoch(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationAborted must not let generic epoch rotation bypass explicit presentation re-establishment"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_abort,
        "generic rotation after navigationAborted must fail before adapter I/O"
    );

    let after_abort = bound.reestablish_presentation_authority(context).expect(
        "a terminal abort must close pending state without permanently denying the owned context",
    );
    assert_eq!(
        after_abort.context_epoch().value(),
        after_failure.context_epoch().value() + 1
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_abort, "usable-after-abort"),
        Ok(context)
    );
}

#[test]
fn navigation_after_negative_terminal_before_reestablishment_waits_for_latest_terminal() {
    let context = BrowsingContextId::new(1012).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = TerminalNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("terminal-navigation-user-context-1012")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1012).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_navigation = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation enters pending state");
    bound
        .record_observed_navigation_terminated(&first_pending, NavigationTerminationOutcome::Failed)
        .expect("first navigation failure closes only its pending transition");
    assert_eq!(adapter_calls.get(), calls_before_navigation);
    assert_eq!(
        bound.advance_context_epoch(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "a negative terminal must leave generic epoch rotation closed until explicit re-establishment"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "rejected generic rotation after the first negative terminal must remain zero-I/O"
    );

    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect(
            "a browser may start another navigation after failure before presentation authority is re-established",
        );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "admitting the later navigation must remain a zero-I/O Browser Session transition"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the earlier navigation failure must not authorize the document while a later navigation is pending"
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation);

    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed failure witness must stay dead after a later navigation starts"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "the prior terminal witness must not terminate the later pending navigation through a different outcome"
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation);
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting stale terminal evidence must leave the latest navigation pending"
    );

    bound
        .record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Aborted,
        )
        .expect(
            "the latest navigation's own terminal outcome closes the current pending transition",
        );
    assert_eq!(adapter_calls.get(), calls_before_navigation);
    assert_eq!(
        bound.advance_context_epoch(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the latest negative terminal must still require explicit re-establishment instead of generic epoch rotation"
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation);

    let reestablished = bound.reestablish_presentation_authority(context).expect(
        "the owner may re-establish only after the latest navigation reaches a terminal outcome",
    );
    assert_eq!(
        reestablished.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "negative terminals and later navigation starts while invalidated must not spend presentation epochs"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &reestablished,
            "usable-after-latest-negative-terminal"
        ),
        Ok(context)
    );
}
