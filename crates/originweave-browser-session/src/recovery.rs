use crate::browser_session::{
    BoundBrowserSession, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateRecoveryEvidence, DisposableContextPort,
};

/// Recovery-only custody of a Browser Session and its exact consumed lifecycle adapter.
///
/// This wrapper is obtained only by consuming a bound session that has already entered
/// [`BrowserSessionState::RecoveryRequired`] or [`BrowserSessionState::TransportLost`]. It exposes
/// only lifecycle state and exact non-authorizing recovery evidence. It deliberately provides none
/// of the ordinary create, presentation-authority, epoch-advance, destroy, authorized-operation, or
/// normal-finish methods, and it does not expose the inner [`BoundBrowserSession`] or concrete port.
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
