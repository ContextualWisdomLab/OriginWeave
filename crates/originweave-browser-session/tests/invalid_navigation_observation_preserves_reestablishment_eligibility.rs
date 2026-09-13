use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
    NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct InvalidNavigationProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for InvalidNavigationProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handles
            .pop_front()
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

fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

fn bound_session(
    session_id: u64,
    first_context: BrowsingContextId,
    second_context: BrowsingContextId,
    adapter_calls: &Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<InvalidNavigationProbePort> {
    let port = InvalidNavigationProbePort {
        handles: VecDeque::from([
            handle(first_context, "invalid-navigation-first-context"),
            handle(second_context, "invalid-navigation-second-context"),
        ]),
        adapter_calls: Rc::clone(adapter_calls),
    };
    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn invalid_later_navigation_does_not_consume_positive_terminal_reestablishment_eligibility() {
    let first_context = BrowsingContextId::new(1041).expect("valid first context");
    let second_context = BrowsingContextId::new(1042).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1041, first_context, second_context, &adapter_calls);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_settled(&first_pending)
        .expect("matching positive terminal creates one re-establishment eligibility");
    let calls_before_invalid_observation = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            second_authority.context_epoch(),
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "a stale or cross-context epoch must be rejected before it can replace terminal-derived re-establishment eligibility"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_invalid_observation,
        "invalid later navigation provenance must fail before adapter I/O"
    );

    let sibling_current = bound
        .presentation_authority(second_context)
        .expect("invalid first-context observation must not disturb the sibling authority");
    assert_eq!(sibling_current.incarnation(), second_authority.incarnation());
    assert_eq!(
        sibling_current.context_epoch(),
        second_authority.context_epoch()
    );

    let reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("rejected invalid navigation must leave the prior valid terminal eligibility intact");
    assert_eq!(
        reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "the invalid observation must not consume an aggregate epoch or force a newer navigation generation"
    );
    assert_eq!(adapter_calls.get(), calls_before_invalid_observation);
}

#[test]
fn invalid_later_navigation_does_not_consume_negative_terminal_reestablishment_eligibility() {
    let first_context = BrowsingContextId::new(1043).expect("valid first context");
    let second_context = BrowsingContextId::new(1044).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(1043, first_context, second_context, &adapter_calls);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        )
        .expect("matching negative terminal creates one re-establishment eligibility");
    let calls_before_invalid_observation = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            second_authority.context_epoch(),
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "invalid later navigation provenance must not erase eligibility created by a negative terminal outcome"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_invalid_observation,
        "invalid post-abort navigation provenance must fail before adapter I/O"
    );

    let reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("rejected invalid navigation must preserve the negative-terminal re-establishment opportunity");
    assert_eq!(
        reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "the rejected observation must not spend an aggregate epoch"
    );
    assert_eq!(adapter_calls.get(), calls_before_invalid_observation);
}
