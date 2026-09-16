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
    /// The retained adapter attempted the recovery operation and returned its bounded failure.
    Adapter(E),
}

/// Recovery-only custody of a Browser Session and its exact consumed lifecycle adapter.
///
/// This wrapper is obtained only by consuming a bound session that has already entered
/// [`BrowserSessionState::RecoveryRequired`] or [`BrowserSessionState::TransportLost`]. It exposes
/// lifecycle state, exact non-authorizing recovery evidence, and a purpose-bounded recovery-operation
/// path through the retained adapter. It deliberately provides none of the ordinary create,
/// presentation-authority, epoch-advance, destroy, authorized-operation, or normal-finish methods, and
/// it does not expose the inner [`BoundBrowserSession`] or concrete port.
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
}

impl<P: DisposableContextPort> BoundBrowserSession<P> {
    /// Consume an unresolved bound session into recovery-only custody without adapter I/O.
    ///
    /// The handoff succeeds only after Browser Session has entered `RecoveryRequired` or
    /// `TransportLost`. Active or normally ended sessions are returned unchanged so callers cannot
    /// use the recovery type as an alternate path around ordinary lifecycle policy.
    pub fn into_recovery(self) -> Result<BoundBrowserSessionRecovery<P>, Self> {
        match self.browser_session().state() {
            BrowserSessionState::RecoveryRequired | BrowserSessionState::TransportLost => {
                Ok(BoundBrowserSessionRecovery { bound: self })
            }
            BrowserSessionState::Active | BrowserSessionState::Ended => Err(self),
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
}

impl<P: RecoveryContextOperationPort> BoundBrowserSessionRecovery<P> {
    /// Execute one purpose-bounded recovery operation through the exact retained lifecycle adapter.
    ///
    /// The request snapshots the unresolved aggregate state and both recovery-evidence ledgers before
    /// adapter I/O. Adapter success or failure leaves Browser Session state and evidence unchanged;
    /// protocol-specific code must provide separate, reviewed reconciliation proof before uncertainty
    /// can be resolved.
    pub fn execute_recovery_context_operation(
        &mut self,
        operation: P::Operation,
    ) -> Result<P::Output, RecoveryContextOperationError<P::Error>> {
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
