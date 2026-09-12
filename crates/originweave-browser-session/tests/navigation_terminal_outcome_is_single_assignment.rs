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

struct ConflictingTerminalOutcomeProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ConflictingTerminalOutcomeProbePort {
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

impl AuthorizedContextOperationPort for ConflictingTerminalOutcomeProbePort {
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
fn one_pending_navigation_accepts_exactly_one_terminal_outcome_across_positive_and_negative_paths() {
    let context = BrowsingContextId::new(1012).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = ConflictingTerminalOutcomeProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("single-terminal-outcome-user-context-1012")
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
    let calls_after_create = adapter_calls.get();

    let positively_terminal = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues one pending-navigation witness");
    bound
        .record_observed_navigation_settled(&positively_terminal)
        .expect("the exact witness may be consumed once by a positive terminal observation");
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_terminated(
            &positively_terminal,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "a witness already consumed by commit/fragment settlement must not be reusable by a conflicting failed outcome"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "conflicting terminal replay must fail before adapter I/O"
    );

    let after_positive = bound
        .reestablish_presentation_authority(context)
        .expect("positive terminal observation permits explicit fresh authority");
    assert_eq!(
        after_positive.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_positive, "usable-after-positive-terminal"),
        Ok(context)
    );

    let calls_before_negative = adapter_calls.get();
    let negatively_terminal = bound
        .record_observed_navigation(
            after_positive.incarnation(),
            context,
            after_positive.context_epoch(),
        )
        .expect("later navigation enters a distinct pending generation");
    assert_eq!(adapter_calls.get(), calls_before_negative);
    bound
        .record_observed_navigation_terminated(
            &negatively_terminal,
            NavigationTerminationOutcome::Aborted,
        )
        .expect("the exact witness may be consumed once by a negative terminal observation");
    assert_eq!(adapter_calls.get(), calls_before_negative);

    assert_eq!(
        bound.record_observed_navigation_settled(&negatively_terminal),
        Err(BrowserSessionError::AuthorityMismatch),
        "a witness already consumed by abort/failure must not be reusable by a conflicting commit/fragment outcome"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_negative,
        "conflicting positive replay must fail before adapter I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_positive, "still-stale-after-negative-terminal"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "a negative terminal outcome must not resurrect the pre-navigation authority"
    );
    assert_eq!(adapter_calls.get(), calls_before_negative);

    let after_negative = bound
        .reestablish_presentation_authority(context)
        .expect("negative terminal observation closes pending state without stranding the owned context");
    assert_eq!(
        after_negative.context_epoch().value(),
        after_positive.context_epoch().value() + 1
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&after_negative, "usable-after-negative-terminal"),
        Ok(context)
    );
}
