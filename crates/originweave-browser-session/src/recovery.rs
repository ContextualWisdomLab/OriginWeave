use crate::browser_session::{
    BoundBrowserSession, BrowserSession, BrowserSessionState, DisposableContextPort,
};

/// Recovery-only custody of a Browser Session and its exact consumed lifecycle adapter.
///
/// This wrapper is obtained only by consuming a bound session that has already entered
/// [`BrowserSessionState::RecoveryRequired`] or [`BrowserSessionState::TransportLost`]. It exposes
/// immutable Browser Session evidence but deliberately provides none of the ordinary create,
/// presentation-authority, epoch-advance, destroy, or authorized-operation methods. The concrete
/// adapter remains private and is moved, not reconstructed or replaced.
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
/// Presentation-authority lookup is not available from recovery custody:
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
    /// Return immutable Browser Session recovery state and evidence.
    ///
    /// No adapter reference is exposed. Protocol-specific recovery code must consume a separately
    /// reviewed recovery operation boundary rather than regaining ordinary mutation authority.
    #[must_use]
    pub const fn browser_session(&self) -> &BrowserSession {
        self.bound.browser_session()
    }
}
