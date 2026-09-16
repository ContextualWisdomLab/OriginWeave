use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
    NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct RecreatedTerminalProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for RecreatedTerminalProbePort {
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

fn assert_destroy_recreate_does_not_inherit_terminal_eligibility(
    session_value: u64,
    context_value: u64,
    terminal: Option<NavigationTerminationOutcome>,
) {
    let context = BrowsingContextId::new(context_value).expect("valid context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = RecreatedTerminalProbePort {
        handles: VecDeque::from([
            handle(context, "terminal-aba-old-ownership"),
            handle(context, "terminal-aba-new-ownership"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_value).expect("valid browser session id"),
    )
    .expect("incarnation capacity")
    .bind_lifecycle_port(port);

    let old_authority = bound
        .create_disposable_context()
        .expect("first ownership generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            context,
            old_authority.context_epoch(),
        )
        .expect("first ownership generation enters navigation-pending state");
    match terminal {
        Some(outcome) => bound
            .record_observed_navigation_terminated(&old_pending, outcome)
            .expect("matching negative terminal creates one re-establishment eligibility"),
        None => bound
            .record_observed_navigation_settled(&old_pending)
            .expect("matching positive terminal creates one re-establishment eligibility"),
    }

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(context)
        .expect("proven destruction consumes the terminal ownership generation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction reaches the lifecycle adapter exactly once"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);

    let new_authority = bound
        .create_disposable_context()
        .expect("the same raw browsing-context id may be accepted as a fresh ownership generation");
    assert_eq!(
        new_authority.context(),
        old_authority.context(),
        "the hostile case intentionally reuses the same raw browsing-context id"
    );
    assert!(
        new_authority.context_epoch().value() > old_authority.context_epoch().value(),
        "fresh ownership must receive a newer aggregate-issued context epoch"
    );

    let calls_after_recreate = adapter_calls.get();
    let recovery_evidence_after_recreate = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "terminal eligibility from a destroyed generation must not attach to a recreated raw context id"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_recreate,
        "rejected stale terminal eligibility must fail before adapter I/O"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_recreate.as_slice(),
        "rejected ABA-style re-establishment must not manufacture recovery evidence"
    );

    let current = bound
        .presentation_authority(context)
        .expect("the recreated generation keeps its fresh presentation authority");
    assert_eq!(current.incarnation(), new_authority.incarnation());
    assert_eq!(current.context_epoch(), new_authority.context_epoch());
    assert_eq!(current.context(), new_authority.context());

    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            context,
            new_authority.context_epoch(),
        )
        .expect("the recreated generation may start its own navigation");
    bound
        .record_observed_navigation_settled(&new_pending)
        .expect("only the recreated generation's own witness may settle its navigation");
    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the recreated generation may re-establish after its own terminal outcome");
    assert_eq!(
        reestablished.context_epoch().value(),
        new_authority.context_epoch().value() + 1,
        "the rejected stale terminal opportunity must not consume an aggregate epoch"
    );
    assert_eq!(adapter_calls.get(), calls_after_recreate);
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_recreate.as_slice(),
        "fresh-generation settlement and re-establishment must not rewrite recovery evidence"
    );
}

#[test]
fn recreated_raw_context_id_does_not_inherit_positive_terminal_reestablishment_eligibility() {
    assert_destroy_recreate_does_not_inherit_terminal_eligibility(972, 972, None);
}

#[test]
fn recreated_raw_context_id_does_not_inherit_aborted_terminal_reestablishment_eligibility() {
    assert_destroy_recreate_does_not_inherit_terminal_eligibility(
        973,
        973,
        Some(NavigationTerminationOutcome::Aborted),
    );
}

#[test]
fn recreated_raw_context_id_does_not_inherit_failed_terminal_reestablishment_eligibility() {
    assert_destroy_recreate_does_not_inherit_terminal_eligibility(
        974,
        974,
        Some(NavigationTerminationOutcome::Failed),
    );
}
