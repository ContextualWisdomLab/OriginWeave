use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateDisposition, DisposableContextCreateError,
    DisposableContextCreateRecoveryEvidence, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
struct CreateAttemptLedger {
    create_attempts: Vec<u64>,
    completion_attempts: Vec<(u64, DisposableContextCreateDisposition)>,
}

#[derive(Debug)]
struct SameHandleRejectedCompletionPort {
    handle: DisposableContextHandle,
    ledger: Rc<RefCell<CreateAttemptLedger>>,
}

impl SameHandleRejectedCompletionPort {
    fn new(
        handle: DisposableContextHandle,
        ledger: Rc<RefCell<CreateAttemptLedger>>,
    ) -> Self {
        Self { handle, ledger }
    }
}

impl DisposableContextPort for SameHandleRejectedCompletionPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.ledger
            .borrow_mut()
            .create_attempts
            .push(request.attempt_epoch().value());
        Ok(self.handle.clone())
    }

    fn complete_disposable_context_creation(
        &mut self,
        completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.ledger.borrow_mut().completion_attempts.push((
            completion.attempt_epoch().value(),
            completion.disposition(),
        ));
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
    let ledger = Rc::new(RefCell::new(CreateAttemptLedger::default()));
    let port = SameHandleRejectedCompletionPort::new(handle.clone(), Rc::clone(&ledger));
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

    let observed = ledger.borrow();
    assert_eq!(observed.create_attempts, vec![1, 2]);
    assert_eq!(
        observed.completion_attempts,
        vec![
            (1, DisposableContextCreateDisposition::Accepted),
            (2, DisposableContextCreateDisposition::Rejected),
        ],
        "the adapter must observe the same aggregate-issued attempt identity that recovery evidence retains"
    );
    drop(observed);

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
            handle.clone()
        )),
        "attempt 1 ownership and attempt 2 candidate are distinct lifecycle facts even when their remote handle values are equal"
    );

    let create_evidence = bound.browser_session().create_attempt_recovery_evidence();
    assert_eq!(create_evidence.len(), 2);
    match &create_evidence[0] {
        DisposableContextCreateRecoveryEvidence::DuplicateCandidate {
            attempt_epoch,
            context,
        } => {
            assert_eq!(attempt_epoch.value(), 2);
            assert_eq!(context, &handle);
        }
        other => panic!("unexpected duplicate recovery evidence: {other:?}"),
    }
    match &create_evidence[1] {
        DisposableContextCreateRecoveryEvidence::CompletionUnsettled {
            attempt_epoch,
            disposition,
            context,
        } => {
            assert_eq!(attempt_epoch.value(), 2);
            assert_eq!(*disposition, DisposableContextCreateDisposition::Rejected);
            assert_eq!(context, &handle);
        }
        other => panic!("unexpected completion recovery evidence: {other:?}"),
    }
}
