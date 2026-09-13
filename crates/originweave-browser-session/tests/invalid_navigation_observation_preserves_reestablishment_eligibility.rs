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

    assert!(
        matches!(
            bound.record_observed_navigation(
                first_authority.incarnation(),
                first_context,
                second_authority.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        ),
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

    assert!(
        matches!(
            bound.record_observed_navigation(
                first_authority.incarnation(),
                first_context,
                second_authority.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        ),
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

#[test]
fn prior_incarnation_navigation_does_not_consume_current_terminal_reestablishment_eligibility() {
    let reused_context = BrowsingContextId::new(1045).expect("valid reused context");
    let sibling_context = BrowsingContextId::new(1046).expect("valid sibling context");
    let prior_adapter_calls = Rc::new(Cell::new(0));
    let current_adapter_calls = Rc::new(Cell::new(0));
    let mut prior = bound_session(
        1045,
        reused_context,
        sibling_context,
        &prior_adapter_calls,
    );
    let mut current = bound_session(
        1045,
        reused_context,
        sibling_context,
        &current_adapter_calls,
    );

    let prior_authority = prior
        .create_disposable_context()
        .expect("prior incarnation accepts its disposable context");
    let current_authority = current
        .create_disposable_context()
        .expect("current incarnation accepts its disposable context");
    let current_pending = current
        .record_observed_navigation(
            current_authority.incarnation(),
            reused_context,
            current_authority.context_epoch(),
        )
        .expect("current incarnation navigation invalidates presentation authority");
    current
        .record_observed_navigation_settled(&current_pending)
        .expect("current terminal creates one re-establishment eligibility");
    let calls_before_stale_incarnation = current_adapter_calls.get();

    assert_ne!(
        prior_authority.incarnation(),
        current_authority.incarnation(),
        "test requires distinct Browser Session incarnations for reused raw identities"
    );
    assert!(
        matches!(
            current.record_observed_navigation(
                prior_authority.incarnation(),
                reused_context,
                current_authority.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        ),
        "a prior-incarnation navigation must be rejected before replacing current terminal-derived eligibility"
    );
    assert_eq!(
        current_adapter_calls.get(),
        calls_before_stale_incarnation,
        "cross-incarnation rejection must happen before adapter I/O"
    );

    let reestablished = current
        .reestablish_presentation_authority(reused_context)
        .expect("rejected prior-incarnation evidence must leave current terminal eligibility intact");
    assert_eq!(
        reestablished.context_epoch().value(),
        current_authority.context_epoch().value() + 1,
        "rejected prior-incarnation evidence must not spend an aggregate epoch"
    );
    assert_eq!(current_adapter_calls.get(), calls_before_stale_incarnation);
}
