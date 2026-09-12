use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct EndedSessionProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for EndedSessionProbePort {
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
fn navigation_after_normal_end_is_rejected_before_context_ownership_lookup() {
    let context = BrowsingContextId::new(981).expect("valid browsing context");
    let foreign = BrowsingContextId::new(982).expect("valid foreign browsing context");
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("ended-navigation-user-context-981")
            .expect("valid isolation id"),
        context,
    );
    let adapter_calls = Rc::new(Cell::new(0));
    let port = EndedSessionProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(981).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .destroy_disposable_context(&authority)
        .expect("proven cleanup permits normal session end");
    bound.end().expect("session ends after proven destruction");
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Ended);

    let calls_after_end = adapter_calls.get();
    let recovery_evidence_after_end = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation(context),
        Err(BrowserSessionError::SessionNotActive),
        "a late navigation from an ended session must not expose historical ownership"
    );
    assert_eq!(
        bound.record_observed_navigation(foreign),
        Err(BrowserSessionError::SessionNotActive),
        "aggregate inactivity must be rejected before a foreign raw selector can become an ownership oracle"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Ended,
        "late navigation must not move a normally ended aggregate back into an active or recovery state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_end.as_slice(),
        "late navigation after normal end must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_end,
        "late owned and foreign navigation selectors must fail before adapter I/O after normal end"
    );
}
