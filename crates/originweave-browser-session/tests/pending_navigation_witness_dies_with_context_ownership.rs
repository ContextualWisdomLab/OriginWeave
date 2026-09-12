use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DestroyedPendingNavigationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DestroyedPendingNavigationProbePort {
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
) -> originweave_browser_session::BoundBrowserSession<DestroyedPendingNavigationProbePort> {
    let port = DestroyedPendingNavigationProbePort {
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
fn positive_terminal_witness_cannot_outlive_proven_context_destruction() {
    let context = BrowsingContextId::new(1031).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1031,
        context,
        "pending-navigation-destroyed-context-1031",
        &adapter_calls,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("active owned navigation issues one pending witness");

    bound
        .destroy_owned_disposable_context(context)
        .expect("proven lifecycle destruction consumes context ownership while navigation remains pending");
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::ContextNotOwned),
        "a witness minted before proven destruction must not settle historical context ownership"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "terminal replay must not recreate presentation authority after lifecycle ownership was consumed"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "a stale terminal witness must not manufacture recovery evidence for a proven-destroyed context"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy settlement and re-establishment must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves the active aggregate eligible for normal end");
}

#[test]
fn negative_terminal_witness_cannot_outlive_proven_context_destruction() {
    let context = BrowsingContextId::new(1032).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1032,
        context,
        "pending-navigation-destroyed-context-1032",
        &adapter_calls,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let pending = bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("active owned navigation issues one pending witness");

    bound
        .destroy_owned_disposable_context(context)
        .expect("proven lifecycle destruction consumes context ownership while navigation remains pending");
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_terminated(
            &pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::ContextNotOwned),
        "a late failure for a proven-destroyed context must not consume historical pending state"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "negative terminal replay must not recreate presentation authority after lifecycle ownership was consumed"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "a stale negative terminal witness must not rewrite lifecycle evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy termination and re-establishment must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves the active aggregate eligible for normal end");
}
