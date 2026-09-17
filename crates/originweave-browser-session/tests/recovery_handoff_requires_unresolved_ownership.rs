use originweave_browser_session::{
    BrowserSession, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CleanPort {
    handle: DisposableContextHandle,
}

impl DisposableContextPort for CleanPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        Ok(self.handle.clone())
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

fn clean_port(context: u64, isolation: &str) -> Result<CleanPort, &'static str> {
    let isolation = DisposableIsolationId::parse(isolation)
        .map_err(|_| "static fixture isolation id must be valid")?;
    let browsing_context = BrowsingContextId::new(context)
        .map_err(|_| "static fixture browsing context id must be valid")?;
    Ok(CleanPort {
        handle: DisposableContextHandle::new(isolation, browsing_context),
    })
}

#[test]
fn transport_loss_without_remote_ownership_cannot_enter_recovery_custody(
) -> Result<(), &'static str> {
    let session = BrowserSession::start(
        BrowserSessionId::new(7_120).map_err(|_| "static session id must be valid")?,
    )
    .map_err(|_| "browser session incarnation must be available")?;
    let mut bound = session.bind_lifecycle_port(clean_port(71_200, "unused-recovery-port")?);

    assert!(bound.record_transport_loss());
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost
    );
    assert!(bound.browser_session().recovery_evidence().is_empty());
    assert!(
        bound.into_recovery().is_err(),
        "transport loss without unresolved remote ownership must not mint recovery adapter authority"
    );
    Ok(())
}

#[test]
fn transport_loss_after_proven_destruction_cannot_reopen_recovery_custody(
) -> Result<(), &'static str> {
    let session = BrowserSession::start(
        BrowserSessionId::new(7_121).map_err(|_| "static session id must be valid")?,
    )
    .map_err(|_| "browser session incarnation must be available")?;
    let mut bound = session.bind_lifecycle_port(clean_port(71_210, "destroyed-recovery-port")?);

    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture context creation must succeed")?;
    bound
        .destroy_disposable_context(&authority)
        .map_err(|_| "fixture destruction must be proven")?;
    assert!(bound.browser_session().recovery_evidence().is_empty());

    assert!(bound.record_transport_loss());
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost
    );
    assert!(bound.browser_session().recovery_evidence().is_empty());
    assert!(
        bound.into_recovery().is_err(),
        "proven destruction must not be followed by a recovery-only adapter capability with no unresolved evidence"
    );
    Ok(())
}
