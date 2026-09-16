use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
    NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ReestablishmentOwnershipProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ReestablishmentOwnershipProbePort {
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

fn bound_session(
    session: u64,
    context: BrowsingContextId,
    isolation: &str,
    adapter_calls: &Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<ReestablishmentOwnershipProbePort> {
    let port = ReestablishmentOwnershipProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(isolation).expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(adapter_calls),
    };
    BrowserSession::start(BrowserSessionId::new(session).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn positive_terminal_reestablishment_eligibility_dies_with_proven_context_destruction() {
    let context = BrowsingContextId::new(1033).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1033,
        context,
        "terminal-reestablishment-destroyed-context-1033",
        &adapter_calls,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_settled(&pending)
        .expect("matching positive terminal creates one re-establishment eligibility");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(context)
        .expect("proven destruction consumes lifecycle ownership before re-establishment");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction reaches the exact bound lifecycle port once"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);

    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "a positive terminal must not leave reusable re-establishment eligibility after exact ownership is destroyed"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "rejected resurrection must not manufacture recovery evidence for a proven-destroyed context"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy re-establishment must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves the active aggregate eligible for normal end");
}

#[test]
fn negative_terminal_reestablishment_eligibility_dies_with_proven_context_destruction() {
    let context = BrowsingContextId::new(1034).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1034,
        context,
        "terminal-reestablishment-destroyed-context-1034",
        &adapter_calls,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(authority.incarnation(), context, authority.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed)
        .expect("matching negative terminal creates one re-establishment eligibility");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(context)
        .expect("proven destruction consumes lifecycle ownership before re-establishment");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction reaches the exact bound lifecycle port once"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);

    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "a negative terminal must not leave reusable re-establishment eligibility after exact ownership is destroyed"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "rejected post-failure resurrection must not rewrite lifecycle evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy re-establishment must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves the active aggregate eligible for normal end");
}
