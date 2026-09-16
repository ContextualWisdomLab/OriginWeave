use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRecoveryEvidence,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId, RecoverySettlementError,
    RecoverySettlementPort, RecoverySettlementRequest,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObservedAbsentProof {
    browsing_context: BrowsingContextId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettlementVerificationError {
    WrongFact,
}

struct SettlementPort {
    create_results: VecDeque<Result<DisposableContextHandle, DisposableContextCreateError>>,
    settlement_calls: Rc<Cell<usize>>,
    settled_recovery: Rc<RefCell<Vec<BrowserSessionRecoveryEvidence>>>,
    settled_create_attempts: Rc<RefCell<Vec<DisposableContextCreateRecoveryEvidence>>>,
}

impl DisposableContextPort for SettlementPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_results
            .pop_front()
            .unwrap_or(Err(DisposableContextCreateError::CreateFailedClean))
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

impl RecoverySettlementPort for SettlementPort {
    type Proof = ObservedAbsentProof;
    type Error = SettlementVerificationError;

    fn verify_recovery_settlement(
        &mut self,
        request: &RecoverySettlementRequest<Self::Proof>,
    ) -> Result<(), Self::Error> {
        self.settlement_calls.set(self.settlement_calls.get() + 1);
        if let Some(evidence) = request.recovery_evidence() {
            let expected_context = match evidence {
                BrowserSessionRecoveryEvidence::UnprovenDestruction { context, .. }
                | BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(context)
                | BrowserSessionRecoveryEvidence::TransportLossOwnedHandle(context)
                | BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(context)
                | BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(context) => {
                    Some(context.browsing_context())
                }
                BrowserSessionRecoveryEvidence::PartialCreationIsolation(_) => None,
            };
            if expected_context.is_some_and(|context| context != request.proof().browsing_context) {
                return Err(SettlementVerificationError::WrongFact);
            }
            self.settled_recovery.borrow_mut().push(evidence.clone());
            return Ok(());
        }
        if let Some(evidence) = request.create_attempt_recovery_evidence() {
            let expected_context = match evidence {
                DisposableContextCreateRecoveryEvidence::DuplicateCandidate { context, .. }
                | DisposableContextCreateRecoveryEvidence::CompletionUnsettled { context, .. } => {
                    Some(context.browsing_context())
                }
                DisposableContextCreateRecoveryEvidence::FailedUncertain { .. } => None,
            };
            if expected_context.is_some_and(|context| context != request.proof().browsing_context) {
                return Err(SettlementVerificationError::WrongFact);
            }
            self.settled_create_attempts.borrow_mut().push(evidence.clone());
            return Ok(());
        }
        Err(SettlementVerificationError::WrongFact)
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
fn exact_fact_settlement_is_single_use_local_and_does_not_erase_sibling_uncertainty(
) -> Result<(), &'static str> {
    let first_handle = handle("recovery-settlement-a", 81_001)?;
    let second_handle = handle("recovery-settlement-b", 81_002)?;
    let settlement_calls = Rc::new(Cell::new(0));
    let settled_recovery = Rc::new(RefCell::new(Vec::new()));
    let settled_create_attempts = Rc::new(RefCell::new(Vec::new()));
    let port = SettlementPort {
        create_results: VecDeque::from([
            Ok(first_handle.clone()),
            Ok(second_handle.clone()),
        ]),
        settlement_calls: Rc::clone(&settlement_calls),
        settled_recovery: Rc::clone(&settled_recovery),
        settled_create_attempts: Rc::clone(&settled_create_attempts),
    };
    let mut bound = BrowserSession::start(session(8_101)?)
        .map_err(|_| "browser session incarnation must be available")?
        .bind_lifecycle_port(port);
    let first_authority = bound
        .create_disposable_context()
        .map_err(|_| "first create must succeed")?;
    let second_authority = bound
        .create_disposable_context()
        .map_err(|_| "second create must succeed")?;
    assert_eq!(
        bound.destroy_disposable_context(&first_authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(bound.browser_session().recovery_evidence().len(), 2);

    let mut recovery = bound
        .into_recovery()
        .map_err(|_| "RecoveryRequired must enter recovery custody")?;
    let first_fact = recovery
        .recovery_fact(0)
        .ok_or("first recovery fact must be addressable")?;
    let stale_replay = recovery
        .recovery_fact(0)
        .ok_or("same current fact may be inspected twice before settlement")?;
    let sibling_fact = recovery
        .recovery_fact(1)
        .ok_or("sibling recovery fact must be addressable")?;

    recovery
        .settle_recovery_fact(
            first_fact,
            ObservedAbsentProof {
                browsing_context: first_handle.browsing_context(),
            },
        )
        .map_err(|_| "independently verified exact first fact must settle")?;
    assert_eq!(settlement_calls.get(), 1);
    assert_eq!(recovery.state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(recovery.recovery_evidence().len(), 1);
    assert!(recovery.recovery_evidence().iter().any(|evidence| {
        matches!(
            evidence,
            BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(context)
                if context == &second_handle
        )
    }));

    assert_eq!(
        recovery.settle_recovery_fact(
            stale_replay,
            ObservedAbsentProof {
                browsing_context: first_handle.browsing_context(),
            },
        ),
        Err(RecoverySettlementError::StaleFact)
    );
    assert_eq!(
        settlement_calls.get(),
        1,
        "stale replay must fail before adapter proof verification"
    );

    assert_eq!(
        recovery.settle_recovery_fact(
            sibling_fact,
            ObservedAbsentProof {
                browsing_context: first_handle.browsing_context(),
            },
        ),
        Err(RecoverySettlementError::StaleFact),
        "settling one fact invalidates previously issued sibling handles; callers must reread current custody"
    );
    assert_eq!(settlement_calls.get(), 1);

    let current_sibling = recovery
        .recovery_fact(0)
        .ok_or("remaining sibling fact must be re-addressable after revision change")?;
    assert_eq!(
        recovery.settle_recovery_fact(
            current_sibling,
            ObservedAbsentProof {
                browsing_context: first_handle.browsing_context(),
            },
        ),
        Err(RecoverySettlementError::Adapter(
            SettlementVerificationError::WrongFact
        ))
    );
    assert_eq!(settlement_calls.get(), 2);
    assert_eq!(recovery.state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(recovery.recovery_evidence().len(), 1);

    let current_sibling = recovery
        .recovery_fact(0)
        .ok_or("failed proof must leave the exact sibling fact current")?;
    recovery
        .settle_recovery_fact(
            current_sibling,
            ObservedAbsentProof {
                browsing_context: second_handle.browsing_context(),
            },
        )
        .map_err(|_| "qualified sibling absence must settle")?;
    assert_eq!(settlement_calls.get(), 3);
    assert!(recovery.recovery_evidence().is_empty());
    assert!(recovery.create_attempt_recovery_evidence().is_empty());
    assert_eq!(
        recovery.state(),
        BrowserSessionState::Ended,
        "settlement may reach a terminal closed state but must never restore ordinary browser authority"
    );
    assert_eq!(settled_recovery.borrow().len(), 2);
    assert!(settled_create_attempts.borrow().is_empty());
    let _ = second_authority;
    Ok(())
}

#[test]
fn foreign_fact_is_rejected_before_the_other_session_adapter_observes_proof(
) -> Result<(), &'static str> {
    fn recovering_session(
        session_id: u64,
        context_id: u64,
        isolation_id: &str,
        settlement_calls: Rc<Cell<usize>>,
    ) -> Result<originweave_browser_session::BoundBrowserSessionRecovery<SettlementPort>, &'static str>
    {
        let owned = handle(isolation_id, context_id)?;
        let port = SettlementPort {
            create_results: VecDeque::from([Ok(owned)]),
            settlement_calls,
            settled_recovery: Rc::new(RefCell::new(Vec::new())),
            settled_create_attempts: Rc::new(RefCell::new(Vec::new())),
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
    let first = recovering_session(8_201, 82_001, "foreign-fact-a", first_calls)?;
    let mut second = recovering_session(
        8_202,
        82_002,
        "foreign-fact-b",
        Rc::clone(&second_calls),
    )?;
    let foreign_fact = first
        .recovery_fact(0)
        .ok_or("foreign recovery fact must exist")?;

    assert_eq!(
        second.settle_recovery_fact(
            foreign_fact,
            ObservedAbsentProof {
                browsing_context: context(82_001)?,
            },
        ),
        Err(RecoverySettlementError::AuthorityMismatch)
    );
    assert_eq!(
        second_calls.get(),
        0,
        "foreign session/incarnation fact must fail before adapter proof verification"
    );
    assert_eq!(second.recovery_evidence().len(), 1);
    Ok(())
}

#[test]
fn dual_recovery_ledgers_require_independent_exact_fact_retirement(
) -> Result<(), &'static str> {
    let uncertain_handle = handle("uncertain-create-fact", 83_001)?;
    let settlement_calls = Rc::new(Cell::new(0));
    let settled_recovery = Rc::new(RefCell::new(Vec::new()));
    let settled_create_attempts = Rc::new(RefCell::new(Vec::new()));
    let port = SettlementPort {
        create_results: VecDeque::from([Err(
            DisposableContextCreateError::CreateFailedUncertain(Some(
                uncertain_handle.isolation().clone(),
            )),
        )]),
        settlement_calls: Rc::clone(&settlement_calls),
        settled_recovery: Rc::clone(&settled_recovery),
        settled_create_attempts: Rc::clone(&settled_create_attempts),
    };
    let mut bound = BrowserSession::start(session(8_301)?)
        .map_err(|_| "browser session incarnation must be available")?
        .bind_lifecycle_port(port);
    assert_eq!(
        bound.create_disposable_context(),
        Err(BrowserSessionError::ContextCreationUncertain)
    );
    let mut recovery = bound
        .into_recovery()
        .map_err(|_| "uncertain create must enter recovery custody")?;
    assert_eq!(recovery.recovery_evidence().len(), 1);
    assert_eq!(recovery.create_attempt_recovery_evidence().len(), 1);

    let identity_fact = recovery
        .recovery_fact(0)
        .ok_or("identity recovery fact must exist")?;
    recovery
        .settle_recovery_fact(
            identity_fact,
            ObservedAbsentProof {
                browsing_context: uncertain_handle.browsing_context(),
            },
        )
        .map_err(|_| "independently qualified identity fact must settle")?;
    assert!(recovery.recovery_evidence().is_empty());
    assert_eq!(recovery.create_attempt_recovery_evidence().len(), 1);
    assert_eq!(recovery.state(), BrowserSessionState::RecoveryRequired);

    let transaction_fact = recovery
        .create_attempt_recovery_fact(0)
        .ok_or("create-attempt fact must remain separately addressable")?;
    recovery
        .settle_recovery_fact(
            transaction_fact,
            ObservedAbsentProof {
                browsing_context: uncertain_handle.browsing_context(),
            },
        )
        .map_err(|_| "independently qualified create-attempt fact must settle")?;
    assert!(recovery.recovery_evidence().is_empty());
    assert!(recovery.create_attempt_recovery_evidence().is_empty());
    assert_eq!(recovery.state(), BrowserSessionState::Ended);
    assert_eq!(settlement_calls.get(), 2);
    assert_eq!(settled_recovery.borrow().len(), 1);
    assert_eq!(settled_create_attempts.borrow().len(), 1);
    Ok(())
}
