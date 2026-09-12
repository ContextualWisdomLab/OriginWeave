use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DestroyedContextProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DestroyedContextProbePort {
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

#[test]
fn late_navigation_after_proven_destroy_cannot_resurrect_consumed_context_ownership() {
    let context = BrowsingContextId::new(991).expect("valid browsing context");
    let foreign = BrowsingContextId::new(992).expect("valid foreign browsing context");
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("destroyed-navigation-user-context-991")
            .expect("valid isolation id"),
        context,
    );
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DestroyedContextProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(991).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        )
        .expect("navigation invalidates presentation authority before lifecycle cleanup");
    bound
        .destroy_owned_disposable_context(context)
        .expect("exact lifecycle owner proves remote cleanup");

    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "proven cleanup consumes context ownership without implicitly ending the reusable browser session"
    );
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation(
            authority.incarnation(),
            context,
            authority.context_epoch(),
        ),
        Err(BrowserSessionError::ContextNotOwned),
        "a buffered navigation for a proven-destroyed context must not revive historical ownership"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "rejecting a late event for consumed ownership must not move the aggregate into recovery"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "a late event for a proven-destroyed context must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "historical context lookup must reject before adapter I/O"
    );

    assert_eq!(
        bound.record_observed_navigation(
            authority.incarnation(),
            foreign,
            authority.context_epoch(),
        ),
        Err(BrowserSessionError::ContextNotOwned),
        "an unrelated raw selector remains unowned while the aggregate is active"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);

    bound
        .end()
        .expect("the reusable session can end normally after consumed ownership stays closed");
}
