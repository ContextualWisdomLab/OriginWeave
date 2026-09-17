use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
    RecoveryContextOperationError, RecoveryContextOperationPort, RecoveryContextOperationRequest,
    RecoverySettlementPort, RecoverySettlementRequest,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct RecoveryPort {
    context: DisposableContextHandle,
    recovery_operation_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for RecoveryPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        Ok(self.context.clone())
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

impl RecoveryContextOperationPort for RecoveryPort {
    type Operation = ();
    type Output = ();
    type Error = ();

    fn execute_recovery_context_operation(
        &mut self,
        _request: &RecoveryContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.recovery_operation_calls
            .set(self.recovery_operation_calls.get() + 1);
        Ok(())
    }
}

impl RecoverySettlementPort for RecoveryPort {
    type Proof = ();
    type Error = ();

    fn verify_recovery_settlement(
        &mut self,
        _request: &RecoverySettlementRequest<Self::Proof>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn session(value: u64) -> Result<BrowserSessionId, &'static str> {
    BrowserSessionId::new(value).map_err(|_| "fixture browser session must be valid")
}

fn context(value: u64) -> Result<BrowsingContextId, &'static str> {
    BrowsingContextId::new(value).map_err(|_| "fixture browsing context must be valid")
}

#[test]
fn terminal_recovery_settlement_revokes_generic_recovery_io() -> Result<(), &'static str> {
    let recovery_operation_calls = Rc::new(Cell::new(0));
    let port = RecoveryPort {
        context: DisposableContextHandle::new(
            DisposableIsolationId::parse("terminal-recovery-context")
                .map_err(|_| "fixture isolation must be representable")?,
            context(91_001)?,
        ),
        recovery_operation_calls: Rc::clone(&recovery_operation_calls),
    };
    let mut bound = BrowserSession::start(session(9_101)?)
        .map_err(|_| "browser session incarnation must be available")?
        .bind_lifecycle_port(port);
    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture create must succeed")?;
    assert_eq!(
        bound.destroy_disposable_context(&authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );

    let mut recovery = bound
        .into_recovery()
        .map_err(|_| "unproven destruction must enter recovery custody")?;
    let fact = recovery
        .recovery_fact(0)
        .ok_or("unproven destruction recovery fact must exist")?;
    recovery
        .execute_recovery_context_operation(fact, ())
        .map_err(|_| "recovery operation must be available while uncertainty remains")?;
    assert_eq!(recovery_operation_calls.get(), 1);

    recovery
        .settle_recovery_fact(fact, ())
        .map_err(|_| "independently verified recovery fact must settle")?;
    assert_eq!(recovery.state(), BrowserSessionState::Ended);

    assert_eq!(
        recovery.execute_recovery_context_operation(fact, ()),
        Err(RecoveryContextOperationError::RecoveryClosed),
        "terminal recovery custody must not retain a generic adapter-I/O capability"
    );
    assert_eq!(
        recovery_operation_calls.get(),
        1,
        "terminal-state rejection must happen before retained-adapter I/O"
    );
    Ok(())
}
