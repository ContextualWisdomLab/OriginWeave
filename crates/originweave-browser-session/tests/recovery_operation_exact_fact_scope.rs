use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, RecoveryContextOperationError, RecoveryContextOperationPort,
    RecoveryContextOperationRequest, RecoverySettlementPort, RecoverySettlementRequest,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryOperation {
    InspectSelectedFact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureError {
    Rejected,
}

struct ExactFactRecoveryPort {
    create_results: VecDeque<DisposableContextHandle>,
    operation_calls: Rc<Cell<usize>>,
    observed_recovery: Rc<RefCell<Vec<Option<BrowserSessionRecoveryEvidence>>>>,
}

impl DisposableContextPort for ExactFactRecoveryPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_results
            .pop_front()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
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
        Err(DisposableContextDestroyError::DestroyFailed)
    }
}

impl RecoveryContextOperationPort for ExactFactRecoveryPort {
    type Operation = RecoveryOperation;
    type Output = ();
    type Error = FixtureError;

    fn execute_recovery_context_operation(
        &mut self,
        request: &RecoveryContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.operation_calls.set(self.operation_calls.get() + 1);
        self.observed_recovery
            .borrow_mut()
            .push(request.recovery_evidence().cloned());
        if request.operation() != &RecoveryOperation::InspectSelectedFact {
            return Err(FixtureError::Rejected);
        }
        Ok(())
    }
}

impl RecoverySettlementPort for ExactFactRecoveryPort {
    type Proof = ();
    type Error = FixtureError;

    fn verify_recovery_settlement(
        &mut self,
        _request: &RecoverySettlementRequest<Self::Proof>,
    ) -> Result<(), Self::Error> {
        Ok(())
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

fn handle(isolation_id: &str, context_id: u64) -> Result<DisposableContextHandle, &'static str> {
    Ok(DisposableContextHandle::new(
        isolation(isolation_id)?,
        context(context_id)?,
    ))
}

#[test]
fn recovery_operation_is_scoped_to_one_current_exact_fact() -> Result<(), &'static str> {
    let first = handle("operation-fact-a", 91_001)?;
    let second = handle("operation-fact-b", 91_002)?;
    let operation_calls = Rc::new(Cell::new(0));
    let observed_recovery = Rc::new(RefCell::new(Vec::new()));
    let port = ExactFactRecoveryPort {
        create_results: VecDeque::from([first.clone(), second.clone()]),
        operation_calls: Rc::clone(&operation_calls),
        observed_recovery: Rc::clone(&observed_recovery),
    };
    let mut bound = BrowserSession::start(session(9_101)?)
        .map_err(|_| "browser session incarnation must be available")?
        .bind_lifecycle_port(port);
    let first_authority = bound
        .create_disposable_context()
        .map_err(|_| "first create must succeed")?;
    let _second_authority = bound
        .create_disposable_context()
        .map_err(|_| "second create must succeed")?;
    assert_eq!(
        bound.destroy_disposable_context(&first_authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );

    let mut recovery = bound
        .into_recovery()
        .map_err(|_| "destroy uncertainty must enter recovery custody")?;
    assert_eq!(recovery.recovery_evidence().len(), 2);
    let first_fact = recovery.recovery_fact(0).ok_or("first fact must exist")?;
    let stale_sibling = recovery.recovery_fact(1).ok_or("sibling fact must exist")?;

    recovery
        .execute_recovery_context_operation(first_fact, RecoveryOperation::InspectSelectedFact)
        .map_err(|_| "current exact fact must authorize its bounded recovery operation")?;
    assert_eq!(operation_calls.get(), 1);
    assert_eq!(observed_recovery.borrow().len(), 1);
    assert_eq!(
        observed_recovery.borrow()[0].as_ref(),
        recovery.recovery_evidence().first(),
        "adapter request must expose only the selected recovery fact, not sibling uncertainty"
    );

    recovery
        .settle_recovery_fact(first_fact, ())
        .map_err(|_| "first exact fact must settle")?;
    assert_eq!(
        recovery.execute_recovery_context_operation(
            stale_sibling,
            RecoveryOperation::InspectSelectedFact,
        ),
        Err(RecoveryContextOperationError::StaleFact),
        "a fact issued before ledger mutation must fail before adapter I/O"
    );
    assert_eq!(operation_calls.get(), 1);

    let current_sibling = recovery
        .recovery_fact(0)
        .ok_or("remaining sibling must be re-issued at the current revision")?;
    recovery
        .execute_recovery_context_operation(
            current_sibling,
            RecoveryOperation::InspectSelectedFact,
        )
        .map_err(|_| "re-issued current sibling must reach the retained adapter")?;
    assert_eq!(operation_calls.get(), 2);
    assert_eq!(observed_recovery.borrow().len(), 2);
    assert_eq!(
        observed_recovery.borrow()[1].as_ref(),
        recovery.recovery_evidence().first(),
    );
    Ok(())
}

#[test]
fn foreign_recovery_fact_is_rejected_before_operation_io() -> Result<(), &'static str> {
    fn recovering_session(
        session_id: u64,
        context_id: u64,
        isolation_id: &str,
        operation_calls: Rc<Cell<usize>>,
    ) -> Result<originweave_browser_session::BoundBrowserSessionRecovery<ExactFactRecoveryPort>, &'static str>
    {
        let owned = handle(isolation_id, context_id)?;
        let port = ExactFactRecoveryPort {
            create_results: VecDeque::from([owned]),
            operation_calls,
            observed_recovery: Rc::new(RefCell::new(Vec::new())),
        };
        let mut bound = BrowserSession::start(session(session_id)?)
            .map_err(|_| "browser session incarnation must be available")?
            .bind_lifecycle_port(port);
        let authority = bound
            .create_disposable_context()
            .map_err(|_| "fixture create must succeed")?;
        assert_eq!(
            bound.destroy_disposable_context(&authority),
            Err(BrowserSessionError::ContextDestructionFailed)
        );
        bound
            .into_recovery()
            .map_err(|_| "fixture must enter recovery custody")
    }

    let first_calls = Rc::new(Cell::new(0));
    let second_calls = Rc::new(Cell::new(0));
    let first = recovering_session(9_201, 92_001, "operation-foreign-a", first_calls)?;
    let mut second = recovering_session(
        9_202,
        92_002,
        "operation-foreign-b",
        Rc::clone(&second_calls),
    )?;
    let foreign_fact = first.recovery_fact(0).ok_or("foreign fact must exist")?;

    assert_eq!(
        second.execute_recovery_context_operation(
            foreign_fact,
            RecoveryOperation::InspectSelectedFact,
        ),
        Err(RecoveryContextOperationError::AuthorityMismatch)
    );
    assert_eq!(second_calls.get(), 0);
    Ok(())
}
