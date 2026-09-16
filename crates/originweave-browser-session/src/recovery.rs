use originweave_core::BrowserSessionId;

use crate::browser_session::{
    BoundBrowserSession, BrowserSessionIncarnation, BrowserSessionRecoveryEvidence,
    BrowserSessionState, DisposableContextCreateRecoveryEvidence, DisposableContextPort,
};

/// Opaque recovery-custody request for one purpose-bounded adapter operation.
///
/// Construction is private to [`BoundBrowserSessionRecovery`]. The request snapshots the exact
/// Browser Session identity, incarnation, unresolved lifecycle state, and both non-authorizing
/// recovery-evidence ledgers immediately before adapter I/O. None of these fields independently grant
/// ordinary creation, presentation mutation, or destruction authority.
pub struct RecoveryContextOperationRequest<O> {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    state: BrowserSessionState,
    recovery_evidence: Vec<BrowserSessionRecoveryEvidence>,
    create_attempt_recovery_evidence: Vec<DisposableContextCreateRecoveryEvidence>,
    operation: O,
}

impl<O> RecoveryContextOperationRequest<O> {
    /// Return the exact browser-session transport identity under recovery custody.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact process-local Browser Session incarnation under recovery custody.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the unresolved Browser Session lifecycle state captured before adapter I/O.
    #[must_use]
    pub const fn state(&self) -> BrowserSessionState {
        self.state
    }

    /// Return the exact non-authorizing remote-ownership evidence captured before adapter I/O.
    #[must_use]
    pub fn recovery_evidence(&self) -> &[BrowserSessionRecoveryEvidence] {
        &self.recovery_evidence
    }

    /// Return exact create-attempt recovery provenance captured before adapter I/O.
    #[must_use]
    pub fn create_attempt_recovery_evidence(&self) -> &[DisposableContextCreateRecoveryEvidence] {
        &self.create_attempt_recovery_evidence
    }

    /// Return the adapter-defined purpose-bounded recovery operation.
    #[must_use]
    pub const fn operation(&self) -> &O {
        &self.operation
    }
}

/// Adapter extension for purpose-bounded recovery operations on the exact consumed lifecycle port.
///
/// Browser Session remains protocol-agnostic. Implementations own their operation, output, and error
/// vocabularies, while recovery custody supplies only immutable lifecycle provenance and routes the
/// request through the same concrete adapter instance consumed by [`BoundBrowserSession`]. A successful
/// adapter return is not itself proof that remote ownership was reconciled or destroyed.
pub trait RecoveryContextOperationPort: DisposableContextPort {
    /// Adapter-defined recovery operation vocabulary.
    type Operation;
    /// Adapter-defined successful result.
    type Output;
    /// Adapter-defined bounded recovery failure.
    type Error;

    /// Execute one purpose-bounded recovery operation using the exact recovery-custody request.
    fn execute_recovery_context_operation(
        &mut self,
        request: &RecoveryContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error>;
}

/// Failure from executing one recovery operation through the exact retained adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryContextOperationError<E> {
    /// Recovery custody has already reached a terminal state with no unresolved command purpose.
    RecoveryClosed,
    /// The retained adapter attempted the recovery operation and returned its bounded failure.
    Adapter(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryFactLedger {
    Recovery,
    CreateAttempt,
}

/// Opaque Browser Session-issued handle for one current recovery fact.
///
/// The fields are private. A caller can retain or replay this value, but cannot construct a different
/// session, ledger, index, or revision. Every successful settlement advances the recovery-ledger
/// revision, which invalidates all handles issued before that mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryFact {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    revision: u64,
    ledger: RecoveryFactLedger,
    index: usize,
}

/// Opaque request used by the retained adapter to verify one independently qualified recovery proof.
///
/// Browser Session chooses exactly one current recovery fact before adapter I/O. The request carries
/// that fact's immutable domain evidence together with the adapter-defined proof. Neither the proof nor
/// the evidence is command authority, and a successful verifier return is committed by Browser Session
/// only after the opaque fact handle has already passed session/incarnation/revision validation.
pub struct RecoverySettlementRequest<P> {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    state: BrowserSessionState,
    recovery_evidence: Option<BrowserSessionRecoveryEvidence>,
    create_attempt_recovery_evidence: Option<DisposableContextCreateRecoveryEvidence>,
    proof: P,
}

impl<P> RecoverySettlementRequest<P> {
    /// Return the exact browser-session transport identity under recovery custody.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact process-local Browser Session incarnation under recovery custody.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the unresolved Browser Session lifecycle state captured before proof verification.
    #[must_use]
    pub const fn state(&self) -> BrowserSessionState {
        self.state
    }

    /// Return the selected identity-oriented recovery fact, when this request targets that ledger.
    #[must_use]
    pub const fn recovery_evidence(&self) -> Option<&BrowserSessionRecoveryEvidence> {
        self.recovery_evidence.as_ref()
    }

    /// Return the selected create-attempt recovery fact, when this request targets that ledger.
    #[must_use]
    pub const fn create_attempt_recovery_evidence(
        &self,
    ) -> Option<&DisposableContextCreateRecoveryEvidence> {
        self.create_attempt_recovery_evidence.as_ref()
    }

    /// Return the adapter-defined independent reconciliation proof.
    #[must_use]
    pub const fn proof(&self) -> &P {
        &self.proof
    }
}

/// Adapter extension that qualifies independent evidence for one exact recovery fact.
///
/// The adapter owns protocol-specific proof semantics such as WebDriver BiDi event correlation and
/// remote-liveness qualification. Browser Session owns the selected domain fact and commits its
/// retirement only after this verifier succeeds. Command acknowledgement alone must not be modeled as
/// proof merely because it was returned by the retained adapter.
pub trait RecoverySettlementPort: DisposableContextPort {
    /// Adapter-defined proof type for independent reconciliation evidence.
    type Proof;
    /// Adapter-defined proof-verification failure.
    type Error;

    /// Verify that the supplied proof reconciles exactly the domain fact carried by the request.
    fn verify_recovery_settlement(
        &mut self,
        request: &RecoverySettlementRequest<Self::Proof>,
    ) -> Result<(), Self::Error>;
}

/// Failure while settling one Browser Session-issued recovery fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverySettlementError<E> {
    /// The fact belongs to another Browser Session or process-local incarnation.
    AuthorityMismatch,
    /// The fact was issued for an older ledger revision or no longer addresses the current ledger.
    StaleFact,
    /// The monotonic recovery-ledger revision cannot advance without wrapping.
    RevisionExhausted,
    /// The retained adapter rejected the independently supplied proof.
    Adapter(E),
}

/// Recovery-only custody of a Browser Session and its exact consumed lifecycle adapter.
///
/// This wrapper is obtained only by consuming a bound session that has already entered
/// [`BrowserSessionState::RecoveryRequired`] or entered [`BrowserSessionState::TransportLost`] while
/// retaining unresolved remote-ownership evidence. It exposes lifecycle state, exact non-authorizing
/// recovery evidence, and purpose-bounded recovery-operation and recovery-settlement paths through the
/// retained adapter. It deliberately provides none of the ordinary create, presentation-authority,
/// epoch-advance, destroy, authorized-operation, or normal-finish methods, and it does not expose the
/// inner [`BoundBrowserSession`] or concrete port.
///
/// Ordinary context creation is not available from recovery custody:
///
/// ```compile_fail
/// use originweave_browser_session::{BoundBrowserSessionRecovery, DisposableContextPort};
/// fn forbidden<P: DisposableContextPort>(mut recovery: BoundBrowserSessionRecovery<P>) {
///     let _ = recovery.create_disposable_context();
/// }
/// ```
///
/// Presentation-authority lookup is not available directly or through an inner Browser Session:
///
/// ```compile_fail
/// use originweave_browser_session::{BoundBrowserSessionRecovery, DisposableContextPort};
/// use originweave_core::BrowsingContextId;
/// fn forbidden<P: DisposableContextPort>(
///     recovery: BoundBrowserSessionRecovery<P>,
///     context: BrowsingContextId,
/// ) {
///     let _ = recovery.presentation_authority(context);
/// }
/// ```
///
/// ```compile_fail
/// use originweave_browser_session::{BoundBrowserSessionRecovery, DisposableContextPort};
/// use originweave_core::BrowsingContextId;
/// fn forbidden<P: DisposableContextPort>(
///     recovery: BoundBrowserSessionRecovery<P>,
///     context: BrowsingContextId,
/// ) {
///     let _ = recovery.browser_session().presentation_authority(context);
/// }
/// ```
///
/// Context-epoch advancement is not available from recovery custody:
///
/// ```compile_fail
/// use originweave_browser_session::{BoundBrowserSessionRecovery, DisposableContextPort};
/// use originweave_core::BrowsingContextId;
/// fn forbidden<P: DisposableContextPort>(
///     mut recovery: BoundBrowserSessionRecovery<P>,
///     context: BrowsingContextId,
/// ) {
///     let _ = recovery.advance_context_epoch(context);
/// }
/// ```
///
/// Ordinary destruction is not available from recovery custody:
///
/// ```compile_fail
/// use originweave_browser_session::{
///     BoundBrowserSessionRecovery, DisposableContextPort, PresentationMutationAuthority,
/// };
/// fn forbidden<P: DisposableContextPort>(
///     mut recovery: BoundBrowserSessionRecovery<P>,
///     authority: PresentationMutationAuthority,
/// ) {
///     let _ = recovery.destroy_disposable_context(&authority);
/// }
/// ```
///
/// Normal session completion is not available from recovery custody:
///
/// ```compile_fail
/// use originweave_browser_session::{BoundBrowserSessionRecovery, DisposableContextPort};
/// fn forbidden<P: DisposableContextPort>(mut recovery: BoundBrowserSessionRecovery<P>) {
///     let _ = recovery.finish();
/// }
/// ```
///
/// Dropping this wrapper performs no browser I/O. The contained [`BoundBrowserSession`] retains its
/// existing abandonment accounting when unresolved remote ownership is finally dropped.
#[must_use = "persist or reconcile unresolved Browser Session ownership before dropping recovery custody"]
pub struct BoundBrowserSessionRecovery<P> {
    bound: BoundBrowserSession<P>,
    revision: u64,
}

impl<P: DisposableContextPort> BoundBrowserSession<P> {
    /// Consume an unresolved bound session into recovery-only custody without adapter I/O.
    ///
    /// `RecoveryRequired` always represents unresolved lifecycle ownership. `TransportLost` permits
    /// handoff only when the aggregate retained exact non-authorizing recovery evidence; transport loss
    /// by itself, before any remote ownership or after proven destruction, must not create an alternate
    /// adapter-operation capability. Active, ended, and ownership-clean transport-lost sessions are
    /// returned unchanged.
    pub fn into_recovery(self) -> Result<BoundBrowserSessionRecovery<P>, Self> {
        let state = self.browser_session().state();
        let has_recovery_evidence = !self.browser_session().recovery_evidence().is_empty()
            || !self
                .browser_session()
                .create_attempt_recovery_evidence()
                .is_empty();
        match state {
            BrowserSessionState::RecoveryRequired => Ok(BoundBrowserSessionRecovery {
                bound: self,
                revision: 1,
            }),
            BrowserSessionState::TransportLost if has_recovery_evidence => {
                Ok(BoundBrowserSessionRecovery {
                    bound: self,
                    revision: 1,
                })
            }
            BrowserSessionState::Active
            | BrowserSessionState::Ended
            | BrowserSessionState::TransportLost => Err(self),
        }
    }
}

impl<P: DisposableContextPort> BoundBrowserSessionRecovery<P> {
    /// Return the lifecycle state captured by the unresolved Browser Session aggregate.
    #[must_use]
    pub const fn state(&self) -> BrowserSessionState {
        self.bound.browser_session().state()
    }

    /// Return exact non-authorizing ownership-recovery evidence.
    #[must_use]
    pub fn recovery_evidence(&self) -> &[BrowserSessionRecoveryEvidence] {
        self.bound.browser_session().recovery_evidence()
    }

    /// Return exact non-authorizing create-attempt recovery provenance.
    #[must_use]
    pub fn create_attempt_recovery_evidence(&self) -> &[DisposableContextCreateRecoveryEvidence] {
        self.bound
            .browser_session()
            .create_attempt_recovery_evidence()
    }

    /// Issue an opaque handle for one current identity-oriented recovery fact.
    #[must_use]
    pub fn recovery_fact(&self, index: usize) -> Option<RecoveryFact> {
        self.recovery_evidence().get(index)?;
        Some(RecoveryFact {
            browser_session: self.bound.browser_session().id(),
            incarnation: self.bound.browser_session().incarnation(),
            revision: self.revision,
            ledger: RecoveryFactLedger::Recovery,
            index,
        })
    }

    /// Issue an opaque handle for one current create-attempt recovery fact.
    #[must_use]
    pub fn create_attempt_recovery_fact(&self, index: usize) -> Option<RecoveryFact> {
        self.create_attempt_recovery_evidence().get(index)?;
        Some(RecoveryFact {
            browser_session: self.bound.browser_session().id(),
            incarnation: self.bound.browser_session().incarnation(),
            revision: self.revision,
            ledger: RecoveryFactLedger::CreateAttempt,
            index,
        })
    }
}

impl<P: RecoveryContextOperationPort> BoundBrowserSessionRecovery<P> {
    /// Execute one purpose-bounded recovery operation through the exact retained lifecycle adapter.
    ///
    /// The request snapshots the unresolved aggregate state and both recovery-evidence ledgers before
    /// adapter I/O. Adapter success or failure leaves Browser Session state and evidence unchanged;
    /// protocol-specific code must provide separate, reviewed reconciliation proof before uncertainty
    /// can be resolved. Once exact-fact settlement closes recovery to `Ended`, later operations fail
    /// before retained-adapter I/O.
    pub fn execute_recovery_context_operation(
        &mut self,
        operation: P::Operation,
    ) -> Result<P::Output, RecoveryContextOperationError<P::Error>> {
        if !matches!(
            self.state(),
            BrowserSessionState::RecoveryRequired | BrowserSessionState::TransportLost
        ) {
            return Err(RecoveryContextOperationError::RecoveryClosed);
        }
        self.bound.dispatch_recovery_operation(|session, port| {
            let request = RecoveryContextOperationRequest {
                browser_session: session.id(),
                incarnation: session.incarnation(),
                state: session.state(),
                recovery_evidence: session.recovery_evidence().to_vec(),
                create_attempt_recovery_evidence: session
                    .create_attempt_recovery_evidence()
                    .to_vec(),
                operation,
            };
            port.execute_recovery_context_operation(&request)
                .map_err(RecoveryContextOperationError::Adapter)
        })
    }
}

impl<P: RecoverySettlementPort> BoundBrowserSessionRecovery<P> {
    /// Verify and retire exactly one current recovery fact through the retained adapter.
    ///
    /// Session/incarnation, current ledger revision, and current fact address are validated before the
    /// adapter sees the proof. A successful proof retires only the selected fact, advances the ledger
    /// revision, and therefore invalidates every handle issued before the mutation. Adapter failure or
    /// any pre-I/O validation failure leaves Browser Session state and both recovery ledgers unchanged.
    pub fn settle_recovery_fact(
        &mut self,
        fact: RecoveryFact,
        proof: P::Proof,
    ) -> Result<(), RecoverySettlementError<P::Error>> {
        let session = self.bound.browser_session();
        if fact.browser_session != session.id() || fact.incarnation != session.incarnation() {
            return Err(RecoverySettlementError::AuthorityMismatch);
        }
        if fact.revision != self.revision {
            return Err(RecoverySettlementError::StaleFact);
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(RecoverySettlementError::RevisionExhausted)?;
        let (recovery_evidence, create_attempt_recovery_evidence) = match fact.ledger {
            RecoveryFactLedger::Recovery => {
                let evidence = self
                    .recovery_evidence()
                    .get(fact.index)
                    .cloned()
                    .ok_or(RecoverySettlementError::StaleFact)?;
                (Some(evidence), None)
            }
            RecoveryFactLedger::CreateAttempt => {
                let evidence = self
                    .create_attempt_recovery_evidence()
                    .get(fact.index)
                    .cloned()
                    .ok_or(RecoverySettlementError::StaleFact)?;
                (None, Some(evidence))
            }
        };
        let request = RecoverySettlementRequest {
            browser_session: session.id(),
            incarnation: session.incarnation(),
            state: session.state(),
            recovery_evidence: recovery_evidence.clone(),
            create_attempt_recovery_evidence: create_attempt_recovery_evidence.clone(),
            proof,
        };
        self.bound
            .dispatch_recovery_operation(|_, port| port.verify_recovery_settlement(&request))
            .map_err(RecoverySettlementError::Adapter)?;

        let retired = match fact.ledger {
            RecoveryFactLedger::Recovery => self.bound.settle_recovery_evidence_at(
                fact.index,
                recovery_evidence
                    .as_ref()
                    .ok_or(RecoverySettlementError::StaleFact)?,
            ),
            RecoveryFactLedger::CreateAttempt => self.bound.settle_create_attempt_recovery_evidence_at(
                fact.index,
                create_attempt_recovery_evidence
                    .as_ref()
                    .ok_or(RecoverySettlementError::StaleFact)?,
            ),
        };
        if !retired {
            return Err(RecoverySettlementError::StaleFact);
        }
        self.revision = next_revision;
        Ok(())
    }
}
