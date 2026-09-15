use std::cell::{Cell, RefCell};
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, RecoveryContextOperationError, RecoveryContextOperationPort,
    RecoveryContextOperationRequest,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryOperation {
    ReconcileExactEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryOperationFailure {
    BackendUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecoveryObservation {
    browser_session: BrowserSessionId,
    incarnation: u64,
    state: BrowserSessionState,
    recovery_evidence: Vec<BrowserSessionRecoveryEvidence>,
    create_evidence_count: usize,
    operation: RecoveryOperation,
}

struct RecoveryPort {
    handle: DisposableContextHandle,
    fail_destroy: bool,
    fail_recovery: Rc<Cell<bool>>,
    recovery_calls: Rc<Cell<usize>>,
    observations: Rc<RefCell<Vec<RecoveryObservation>>>,
}

impl DisposableContextPort for RecoveryPort {
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
        if self.fail_destroy {
            Err(DisposableContextDestroyError::DestroyFailed)
        } else {
            Ok(())
        }
    }
}

impl RecoveryContextOperationPort for RecoveryPort {
    type Operation = RecoveryOperation;
    type Output = ();
    type Error = RecoveryOperationFailure;

    fn execute_recovery_context_operation(
        &mut self,
        request: &RecoveryContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.recovery_calls.set(self.recovery_calls.get() + 1);
        self.observations.borrow_mut().push(RecoveryObservation {
            browser_session: request.browser_session(),
            incarnation: request.incarnation().value(),
            state: request.state(),
            recovery_evidence: request.recovery_evidence().to_vec(),
            create_evidence_count: request.create_attempt_recovery_evidence().len(),
            operation: *request.operation(),
        });
        if self.fail_recovery.get() {
            Err(RecoveryOperationFailure::BackendUnavailable)
        } else {
            Ok(())
        }
    }
}

fn isolation(value: &str) -> Result<DisposableIsolationId, &'static str> {
    DisposableIsolationId::parse(value).map_err(|_| "fixture isolation must be representable")
}

fn context(value: u64) -> Result<BrowsingContextId, &'static str> {
    BrowsingContextId::new(value).map_err(|_| "fixture browsing context must be valid")
}

fn session(value: u64) -> Result<BrowserSessionId, &'static str> {
    BrowserSessionId::new(value).map_err(|_| "fixture session must be valid")
}

#[test]
fn recovery_custody_routes_only_purpose_bounded_io_to_the_exact_consumed_adapter(
) -> Result<(), &'static str> {
    let fail_recovery = Rc::new(Cell::new(true));
    let recovery_calls = Rc::new(Cell::new(0));
    let observations = Rc::new(RefCell::new(Vec::new()));
    let expected_session = session(7_901)?;
    let expected_handle = DisposableContextHandle::new(
        isolation("same-adapter-recovery-user-context")?,
        context(79_010)?,
    );
    let port = RecoveryPort {
        handle: expected_handle.clone(),
        fail_destroy: true,
        fail_recovery: Rc::clone(&fail_recovery),
        recovery_calls: Rc::clone(&recovery_calls),
        observations: Rc::clone(&observations),
    };
    let mut bound = BrowserSession::start(expected_session)
        .map_err(|_| "browser session incarnation must be available")?
        .bind_lifecycle_port(port);
    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture create must succeed")?;
    let expected_incarnation = bound.browser_session().incarnation();
    assert_eq!(
        bound.destroy_disposable_context(&authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    let expected_recovery_evidence = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        expected_recovery_evidence,
        vec![BrowserSessionRecoveryEvidence::UnprovenDestruction {
            context: expected_handle,
            context_epoch: authority.context_epoch(),
        }]
    );

    let mut recovery = bound
        .into_recovery()
        .map_err(|_| "RecoveryRequired must enter recovery custody")?;

    assert_eq!(
        recovery.execute_recovery_context_operation(RecoveryOperation::ReconcileExactEvidence),
        Err(RecoveryContextOperationError::Adapter(
            RecoveryOperationFailure::BackendUnavailable
        ))
    );
    assert_eq!(recovery_calls.get(), 1);
    assert_eq!(recovery.state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(recovery.recovery_evidence(), expected_recovery_evidence);

    let first = observations.borrow();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].browser_session, expected_session);
    assert_eq!(first[0].incarnation, expected_incarnation.value());
    assert_eq!(first[0].state, BrowserSessionState::RecoveryRequired);
    assert_eq!(first[0].recovery_evidence, expected_recovery_evidence);
    assert_eq!(first[0].create_evidence_count, 0);
    assert_eq!(
        first[0].operation,
        RecoveryOperation::ReconcileExactEvidence
    );
    drop(first);

    fail_recovery.set(false);
    recovery
        .execute_recovery_context_operation(RecoveryOperation::ReconcileExactEvidence)
        .map_err(|_| "purpose-bounded recovery operation must reach the retained adapter")?;
    assert_eq!(recovery_calls.get(), 2);
    assert_eq!(
        recovery.state(),
        BrowserSessionState::RecoveryRequired,
        "generic recovery adapter success is not itself destruction or reconciliation proof"
    );
    assert_eq!(
        recovery.recovery_evidence(),
        expected_recovery_evidence,
        "recovery operation dispatch must not erase unresolved ownership evidence"
    );
    Ok(())
}
