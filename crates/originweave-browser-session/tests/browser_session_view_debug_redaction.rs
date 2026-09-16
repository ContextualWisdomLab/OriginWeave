use originweave_browser_session::{
    BrowserSession, DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DebugExposurePort;

impl DisposableContextPort for DebugExposurePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("browser-session-debug-secret-isolation")
                .expect("lossless isolation id"),
            BrowsingContextId::new(777).expect("valid browsing context"),
        ))
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

#[test]
fn read_only_browser_session_view_debug_does_not_expose_remote_identity() {
    let session = BrowserSession::start(BrowserSessionId::new(777).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(DebugExposurePort);
    bound
        .create_disposable_context()
        .expect("accepted disposable context");

    let bound_debug = format!("{bound:?}");
    assert!(
        !bound_debug.contains("browser-session-debug-secret-isolation"),
        "the bound wrapper already promises redacted diagnostics"
    );

    let read_only_debug = format!("{:?}", bound.browser_session());
    assert!(
        !read_only_debug.contains("browser-session-debug-secret-isolation"),
        "the public read-only Browser Session view must not bypass BoundBrowserSession debug redaction and disclose exact remote lifecycle identity"
    );
}
