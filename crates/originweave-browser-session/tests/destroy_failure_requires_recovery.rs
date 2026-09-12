use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct FailingDestroyPort {
    next_handle: DisposableContextHandle,
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
}

impl FailingDestroyPort {
    fn new(
        context: u64,
        isolation: &str,
        create_calls: Rc<Cell<usize>>,
        destroy_calls: Rc<Cell<usize>>,
    ) -> Result<Self, &'static str> {
        let isolation = DisposableIsolationId::parse(isolation)
            .map_err(|_| "static fixture isolation id must be valid")?;
        let browsing_context = BrowsingContextId::new(context)
            .map_err(|_| "static fixture browsing context id must be valid")?;
        Ok(Self {
            next_handle: DisposableContextHandle::new(isolation, browsing_context),
            create_calls,
            destroy_calls,
        })
    }
}

impl DisposableContextPort for FailingDestroyPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls.set(self.create_calls.get() + 1);
        Ok(self.next_handle.clone())
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
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Err(DisposableContextDestroyError::DestroyFailed)
    }
}

/// An unproven destroy must retain exact recovery evidence and reject later normal authority.
#[test]
fn destroy_failure_requires_recovery_before_any_new_authority() -> Result<(), &'static str> {
    let session_id = BrowserSessionId::new(501)
        .map_err(|_| "static fixture browser session id must be valid")?;
    let context_id = BrowsingContextId::new(5010)
        .map_err(|_| "static fixture browsing context id must be valid")?;
    let expected_isolation = DisposableIsolationId::parse("user-context-501")
        .map_err(|_| "static fixture recovery isolation id must be valid")?;
    let expected_handle = DisposableContextHandle::new(expected_isolation, context_id);
    let session = BrowserSession::start(session_id)
        .map_err(|_| "browser session incarnation must be available")?;
    let create_calls = Rc::new(Cell::new(0));
    let destroy_calls = Rc::new(Cell::new(0));
    let failing_port = FailingDestroyPort::new(
        5010,
        "user-context-501",
        Rc::clone(&create_calls),
        Rc::clone(&destroy_calls),
    )?;
    let mut bound = session.bind_lifecycle_port(failing_port);

    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture disposable context creation must succeed")?;
    assert_eq!(
        bound.destroy_disposable_context(&authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(destroy_calls.get(), 1);
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        &[BrowserSessionRecoveryEvidence::UnprovenDestruction(
            expected_handle
        )]
    );
    assert!(!bound.browser_session().transport_is_lost());

    assert!(bound.record_transport_loss());
    assert!(bound.browser_session().transport_is_lost());
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );
    assert!(!bound.record_transport_loss());
    assert_eq!(
        bound.create_disposable_context(),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(create_calls.get(), 1);
    assert_eq!(
        bound.presentation_authority(context_id),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(
        bound.advance_context_epoch(context_id),
        Err(BrowserSessionError::SessionNotActive)
    );
    assert_eq!(bound.end(), Err(BrowserSessionError::SessionNotActive));
    Ok(())
}
