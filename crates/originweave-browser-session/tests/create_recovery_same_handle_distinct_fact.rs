use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateDisposition, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct SameHandleRejectedCompletionPort {
    handle: DisposableContextHandle,
    create_attempts: Vec<u64>,
    completion_attempts: Vec<(u64, DisposableContextCreateDisposition)>,
}

impl SameHandleRejectedCompletionPort {
    fn new(handle: DisposableContextHandle) -> Self {
        Self {
            handle,
            create_attempts: Vec::new(),
            completion_attempts: Vec::new(),
        }
    }
}

impl DisposableContextPort for SameHandleRejectedCompletionPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_attempts.push(request.attempt_epoch().value());
        Ok(self.handle.clone())
    }

    fn complete_disposable_context_creation(
        &mut self,
        completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.completion_attempts
            .push((completion.attempt_epoch().value(), completion.disposition()));
        if completion.disposition() == DisposableContextCreateDisposition::Rejected {
            Err(DisposableContextCreateCompletionError::CompletionFailed)
        } else {
            Ok(())
        }
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

#[test]
fn same_valued_rejected_create_keeps_prior_ownership_as_a_distinct_recovery_fact() {
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse("same-valued-create-recovery")
            .expect("valid browser-issued isolation identity"),
        BrowsingContextId::new(94_001).expect("valid browsing-context identity"),
    );
    let session = BrowserSession::start(
        BrowserSessionId::new(94_001).expect("valid browser-session identity"),
    )
    .expect("browser-session incarnation capacity");
    let port = SameHandleRejectedCompletionPort::new(handle.clone());
    let mut bound = session.bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("attempt 1 becomes the accepted owned context");
    assert_eq!(first_authority.context_epoch().value(), 1);

    assert_eq!(
        bound.create_disposable_context(),
        Err(BrowserSessionError::ContextCreationUncertain),
        "attempt 2 returns the same remote handle but its rejected completion is unproven"
    );

    assert_eq!(bound.browser_session().recovery_evidence().len(), 3);
    assert!(bound
        .browser_session()
        .recovery_evidence()
        .contains(&BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
            handle.clone()
        )));
    assert!(bound
        .browser_session()
        .recovery_evidence()
        .contains(&BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(
            handle.clone()
        )));
    assert!(bound
        .browser_session()
        .recovery_evidence()
        .contains(&BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(
            handle
        )),
        "attempt 1 ownership and attempt 2 candidate are distinct lifecycle facts even when their remote handle values are equal"
    );
}
