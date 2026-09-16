//! Browser Session lifecycle authority for OriginWeave.
//!
//! This crate owns the domain transition that turns a newly created disposable
//! browser isolation boundary into presentation-mutation authority. Driver identifiers
//! remain adapter data: naming a session or browsing context is never sufficient to mint authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use originweave_core::{BrowserSessionId, BrowsingContextId};

static NEXT_BROWSER_SESSION_INCARNATION: AtomicU64 = AtomicU64::new(1);
static ABANDONED_BOUND_SESSIONS: AtomicU64 = AtomicU64::new(0);

/// Return the number of bound Browser Sessions abandoned with unresolved remote ownership.
///
/// This is a process-local, non-I/O operability signal. It deliberately does not claim that remote
/// browser cleanup happened and is not a substitute for persisting exact recovery evidence before a
/// process exits.
#[must_use]
pub fn abandoned_bound_session_count() -> u64 {
    ABANDONED_BOUND_SESSIONS.load(Ordering::Relaxed)
}

/// Current lifecycle state of one Browser Session aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSessionState {
    /// The session may create and own disposable contexts.
    Active,
    /// Every owned context was destroyed and the session was ended normally.
    Ended,
    /// The browser transport was lost while no ownership-recovery condition preceded it.
    TransportLost,
    /// Browser lifecycle ownership became uncertain and requires external reconciliation.
    RecoveryRequired,
}

/// Domain failure while changing Browser Session ownership state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSessionError {
    /// The requested transition requires an active Browser Session.
    SessionNotActive,
    /// No unused session-incarnation identity remains in this process.
    IncarnationExhausted,
    /// No unused monotonic authority or navigation generation remains, so issuance must fail closed.
    EpochExhausted,
    /// The disposable-context port proved that context creation failed without creating a boundary.
    ContextCreationFailed,
    /// Context creation may have created browser state that the aggregate cannot safely own or destroy.
    ContextCreationUncertain,
    /// The port returned a browsing-context identity already known to this aggregate.
    DuplicateBrowsingContext,
    /// The port returned an isolation identity already known to this aggregate.
    DuplicateDisposableIsolation,
    /// The requested context is not currently owned and active in this session.
    ContextNotOwned,
    /// The supplied authority belongs to another incarnation, isolation boundary, session, context, or epoch.
    AuthorityMismatch,
    /// The disposable-context port could not prove destruction of the owned isolation boundary.
    ContextDestructionFailed,
    /// Normal session end was requested while an owned or uncertain context remains.
    ActiveContextRemains,
}

/// Bounded failure from disposable-context creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisposableContextCreateError {
    /// Creation failed and the adapter proved that no disposable boundary was created.
    CreateFailedClean,
    /// Creation failed after ownership may have changed. The optional identity is the exact
    /// browser-issued isolation identity already known at the failure boundary, when available.
    CreateFailedUncertain(Option<DisposableIsolationId>),
}

/// Bounded failure from disposable-context destruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableContextDestroyError {
    /// Destruction of an owned disposable context failed or could not be proven.
    DestroyFailed,
}

/// Compatibility error type for parsing a browser-issued disposable isolation identity.
///
/// Browser Session preserves protocol text exactly and therefore does not currently emit either
/// variant. The result-shaped API remains stable for callers while protocol/runtime qualification
/// stays in the adapter boundary rather than being redefined as Browser Session lexical grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableIsolationIdError {
    /// Reserved for compatibility with callers compiled against the earlier non-empty constraint.
    Empty,
    /// Reserved for compatibility with callers compiled against the earlier character constraint.
    InvalidCharacter,
}

/// Browser-issued identity for one disposable isolation boundary.
///
/// This value is addressability, not mutation authority. A conforming adapter must return a value
/// that is non-aliasing for the live lifetime of the created boundary. A WebDriver BiDi adapter
/// should map this one-to-one to the specification-defined unique user-context identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisposableIsolationId(String);

impl DisposableIsolationId {
    /// Preserve one browser-issued isolation identity exactly as protocol text.
    ///
    /// Browser Session does not trim, normalize, reject empty text, reject control characters, or
    /// impose an implementation-selected length limit. Any narrower runtime grammar must be proven
    /// and enforced by the versioned adapter before this remote lifecycle address enters the domain.
    pub fn parse(value: &str) -> Result<Self, DisposableIsolationIdError> {
        Ok(Self(value.to_owned()))
    }

    /// Return the exact browser-issued isolation identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Process-local, non-reused identity for one Browser Session aggregate incarnation.
///
/// Presentation authority is intentionally non-serializable. A process restart therefore destroys
/// every outstanding authority value. Within one process this monotonic identity prevents a later
/// aggregate from revalidating an authority retained from an earlier aggregate that reused the same
/// transport/session and browser-issued context identifiers. The identity is also passed through the
/// lifecycle port so an adapter must scope its remote ownership mapping to the same incarnation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrowserSessionIncarnation(u64);

impl BrowserSessionIncarnation {
    /// Return the monotonic process-local incarnation value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Adapter result for one newly created disposable browser context.
///
/// The isolation identity scopes the lifecycle boundary used for destruction; the browsing-context
/// identity addresses the independently navigable context inside that boundary. Neither field alone
/// is presentation-mutation authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisposableContextHandle {
    isolation: DisposableIsolationId,
    browsing_context: BrowsingContextId,
}

impl DisposableContextHandle {
    /// Bind one validated isolation identity to its created browsing context.
    #[must_use]
    pub fn new(isolation: DisposableIsolationId, browsing_context: BrowsingContextId) -> Self {
        Self {
            isolation,
            browsing_context,
        }
    }

    /// Return the non-aliasing disposable isolation identity.
    #[must_use]
    pub fn isolation(&self) -> &DisposableIsolationId {
        &self.isolation
    }

    /// Return the browsing-context address inside the disposable boundary.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }
}

/// Lossless evidence retained when browser lifecycle ownership is no longer proven.
///
/// These values authorize no browser command. They exist only so a separately reviewed recovery
/// path can later reconcile exact remote identities instead of guessing from raw session/context ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserSessionRecoveryEvidence {
    /// A partial creation exposed a browser-issued isolation identity before completion became uncertain.
    PartialCreationIsolation(DisposableIsolationId),
    /// A create call returned a complete handle that aliased an already-owned context or isolation.
    DuplicateAdapterHandle(DisposableContextHandle),
    /// A complete create result could not be settled with the bound adapter after domain validation.
    UnsettledAdapterHandle(DisposableContextHandle),
    /// Destruction of this exact owned handle and validated authority epoch failed or could not be proven.
    UnprovenDestruction {
        /// Exact owned context whose remote boundary remains uncertain.
        context: DisposableContextHandle,
        /// Browser Session epoch validated immediately before destroy I/O.
        context_epoch: BrowserContextEpoch,
    },
    /// A recovery condition elsewhere in the session made this active owned handle uncertain.
    RecoveryRequiredOwnedHandle(DisposableContextHandle),
    /// Transport loss made this previously active owned handle uncertain.
    TransportLossOwnedHandle(DisposableContextHandle),
}

/// Exact create-attempt facts retained when one Browser Session creation transaction becomes uncertain.
///
/// The legacy identity-oriented [`BrowserSessionRecoveryEvidence`] remains useful to recovery code that
/// reconciles remote handles. This companion evidence preserves the aggregate-issued attempt epoch and
/// completion disposition so two lifecycle facts with the same remote values cannot be collapsed into
/// one transaction. These values grant no browser command authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisposableContextCreateRecoveryEvidence {
    /// The adapter reported an uncertain create failure before a complete handle was available.
    FailedUncertain {
        /// Exact aggregate-issued epoch reserved for the failed create attempt.
        attempt_epoch: BrowserContextEpoch,
        /// Browser-issued isolation identity known at the failure boundary, when available.
        isolation: Option<DisposableIsolationId>,
    },
    /// A complete candidate aliased already-owned browser state and was rejected by the aggregate.
    DuplicateCandidate {
        /// Exact aggregate-issued epoch reserved for the rejected create attempt.
        attempt_epoch: BrowserContextEpoch,
        /// Exact adapter-returned candidate associated with that attempt.
        context: DisposableContextHandle,
    },
    /// The adapter could not prove completion settlement for one exact create attempt.
    CompletionUnsettled {
        /// Exact aggregate-issued epoch reserved for the unsettled create attempt.
        attempt_epoch: BrowserContextEpoch,
        /// Aggregate decision whose delivery to the adapter could not be proven.
        disposition: DisposableContextCreateDisposition,
        /// Exact adapter-returned candidate associated with that attempt.
        context: DisposableContextHandle,
    },
}

/// Opaque Browser Session-issued request for one disposable-context creation attempt.
///
/// There is deliberately no public constructor. A request is created only inside a
/// [`BoundBrowserSession`], after Browser Session has validated that the aggregate is active. Raw
/// session, incarnation, context, isolation, or adapter-selected identifiers cannot recreate it.
#[derive(Debug)]
pub struct DisposableContextCreateRequest {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    attempt_epoch: BrowserContextEpoch,
}

impl DisposableContextCreateRequest {
    /// Return the Browser Session transport identity for adapter addressability.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the non-reused Browser Session incarnation for adapter lifecycle mapping.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the unique context epoch reserved for this create attempt.
    #[must_use]
    pub const fn attempt_epoch(&self) -> BrowserContextEpoch {
        self.attempt_epoch
    }
}

/// Domain disposition for one completed disposable-context create attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableContextCreateDisposition {
    /// The returned handle passed Browser Session ownership validation and may become authorizing.
    Accepted,
    /// The returned handle failed Browser Session ownership validation and must remain non-authorizing.
    Rejected,
}

/// Opaque Browser Session-issued completion for one exact create attempt.
///
/// The adapter may stage remote protocol state while executing a create request, but it must not
/// promote that state into an authorizing binding until it receives an `Accepted` completion for the
/// same session incarnation and attempt epoch. `Rejected` candidates are recovery/quarantine evidence
/// only. There is deliberately no public constructor.
#[derive(Debug)]
pub struct DisposableContextCreateCompletion {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    attempt_epoch: BrowserContextEpoch,
    disposition: DisposableContextCreateDisposition,
}

impl DisposableContextCreateCompletion {
    /// Return the Browser Session transport identity for adapter correlation.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the non-reused Browser Session incarnation for adapter correlation.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the create-attempt epoch that this completion settles.
    #[must_use]
    pub const fn attempt_epoch(&self) -> BrowserContextEpoch {
        self.attempt_epoch
    }

    /// Return whether Browser Session accepted or rejected the created candidate.
    #[must_use]
    pub const fn disposition(&self) -> DisposableContextCreateDisposition {
        self.disposition
    }
}

/// Failure while settling one exact create attempt with the bound lifecycle adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableContextCreateCompletionError {
    /// The adapter could not prove that the exact pending create attempt reached the requested state.
    CompletionFailed,
}

/// Opaque Browser Session-issued request for destruction of one exact owned disposable context.
///
/// There is deliberately no public constructor. The bound aggregate creates this request only after
/// validating current lifecycle custody. A caller cannot rebuild cleanup authority from raw browser
/// identifiers. The epoch is correlation evidence for the already-authorized request; it is not
/// independently sufficient to destroy state.
#[derive(Debug)]
pub struct DisposableContextDestroyRequest {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    context: DisposableContextHandle,
    context_epoch: BrowserContextEpoch,
}

impl DisposableContextDestroyRequest {
    /// Return the Browser Session transport identity for adapter addressability.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the non-reused Browser Session incarnation for adapter lifecycle mapping.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the exact domain handle whose remote isolation boundary must be destroyed.
    #[must_use]
    pub const fn context(&self) -> &DisposableContextHandle {
        &self.context
    }

    /// Return the exact Browser Session epoch validated before destroy I/O.
    #[must_use]
    pub const fn context_epoch(&self) -> BrowserContextEpoch {
        self.context_epoch
    }
}

/// Port implemented by a reviewed browser adapter for disposable context lifecycle operations.
///
/// The port never self-asserts an instance identifier. Instead, Browser Session consumes one concrete
/// port value into [`BoundBrowserSession`]. Public lifecycle methods then use only that owned port, so a
/// caller cannot swap a second adapter instance into create or destroy after binding. The port receives
/// only aggregate-issued request values with private construction paths.
///
/// For WebDriver BiDi, creation should map the isolation identity one-to-one to the user-context
/// identifier returned by `browser.createUserContext`. [`DisposableContextCreateError::CreateFailedClean`]
/// is allowed only when the adapter proves that no disposable state was created. If a user-context
/// identity is already known when later creation or verification becomes uncertain, the adapter must
/// return it inside [`DisposableContextCreateError::CreateFailedUncertain`].
///
/// `destroy_disposable_context` must destroy the exact boundary carried by the supplied request and
/// return success only after destruction is proven. Reconstructing cleanup authority from raw driver
/// identifiers is forbidden, and a command acknowledgement alone is insufficient evidence.
pub trait DisposableContextPort {
    /// Create one fresh disposable isolation boundary and browsing context for this authorized request.
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError>;

    /// Settle the exact create attempt after Browser Session validates the returned domain handle.
    ///
    /// An adapter must keep a successful remote create result non-authorizing until this completion
    /// accepts the matching attempt. A rejected attempt must remain non-authorizing and be retained
    /// only for recovery/quarantine processing.
    fn complete_disposable_context_creation(
        &mut self,
        completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError>;

    /// Destroy the exact disposable isolation boundary represented by this authorized request.
    fn destroy_disposable_context(
        &mut self,
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError>;
}

/// Opaque aggregate-authorized request for one purpose-bounded adapter operation.
///
/// The caller supplies only the adapter-defined operation value. Browser Session validates the
/// accompanying presentation authority first and privately binds the operation to the exact owned
/// context and validated epoch before the consumed adapter can observe it. There is deliberately no
/// public constructor, and the epoch is correlation/provenance rather than standalone authority.
pub struct AuthorizedContextOperationRequest<O> {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    context: DisposableContextHandle,
    context_epoch: BrowserContextEpoch,
    operation: O,
}

impl<O> AuthorizedContextOperationRequest<O> {
    /// Return the Browser Session transport identity for adapter addressability.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the non-reused Browser Session incarnation for adapter lifecycle correlation.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the exact currently owned context validated before adapter I/O.
    #[must_use]
    pub const fn context(&self) -> &DisposableContextHandle {
        &self.context
    }

    /// Return the exact Browser Session epoch validated before adapter I/O.
    #[must_use]
    pub const fn context_epoch(&self) -> BrowserContextEpoch {
        self.context_epoch
    }

    /// Return the adapter-defined purpose-bounded operation payload.
    #[must_use]
    pub const fn operation(&self) -> &O {
        &self.operation
    }
}

/// Failure from executing an aggregate-authorized operation through the consumed adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizedContextOperationError<E> {
    /// Browser Session rejected the authority before adapter I/O.
    BrowserSession(BrowserSessionError),
    /// The bound adapter attempted the authorized operation and returned its bounded failure.
    Adapter(E),
}

/// Adapter extension for purpose-bounded operations that must use the exact consumed adapter.
///
/// Browser Session remains protocol-agnostic: the adapter owns the operation, output, and error
/// types. The wrapper only proves current ownership and routes the opaque request to the same concrete
/// adapter instance used for lifecycle creation and destruction. Implementations must not treat the
/// request as permission to mutate any other context.
pub trait AuthorizedContextOperationPort: DisposableContextPort {
    /// Adapter-defined operation vocabulary, such as a reviewed BiDi presentation command.
    type Operation;
    /// Adapter-defined successful result.
    type Output;
    /// Adapter-defined bounded operation failure.
    type Error;

    /// Execute one aggregate-authorized operation against the exact context carried by the request.
    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error>;
}

/// Monotonic identity for one owned browsing-context authority epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrowserContextEpoch(u64);

impl BrowserContextEpoch {
    /// Return the internal monotonic epoch value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Opaque proof that Browser Session currently owns presentation mutation for one context epoch.
///
/// The fields are private and no public constructor exists. A caller obtains this value only after
/// Browser Session has created a disposable boundary through its bound lifecycle port. Session
/// incarnation, isolation identity, context identity, and epoch must all still match before adapter I/O
/// is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationMutationAuthority {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    isolation: DisposableIsolationId,
    browsing_context: BrowsingContextId,
    context_epoch: BrowserContextEpoch,
}

impl PresentationMutationAuthority {
    /// Return the Browser Session transport identity associated with this authority.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the Browser Session incarnation that minted this authority.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the owned disposable isolation identity.
    #[must_use]
    pub fn isolation(&self) -> &DisposableIsolationId {
        &self.isolation
    }

    /// Return the owned browsing-context identity.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Return the exact context epoch covered by this authority.
    #[must_use]
    pub const fn context_epoch(&self) -> BrowserContextEpoch {
        self.context_epoch
    }
}

/// Opaque Browser Session-issued witness for one admitted navigation generation.
///
/// Raw protocol navigation/context identifiers are evidence only. This value is minted only after the
/// aggregate validates the current session incarnation, owned context, and presentation epoch. Its
/// fields remain private so a caller cannot manufacture terminal or commit authority from raw BiDi
/// event data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationSettlementAuthority {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    browsing_context: BrowsingContextId,
    context_epoch: BrowserContextEpoch,
    navigation_generation: u64,
}

/// Typed negative terminal outcome for one admitted navigation witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTerminationOutcome {
    /// The browser reported navigation abortion.
    Aborted,
    /// The browser reported navigation failure.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnedContextState {
    Active,
    Uncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentationNavigationState {
    Established,
    Pending {
        navigation_generation: u64,
        committed: bool,
    },
    Eligible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedContextRecord {
    handle: DisposableContextHandle,
    epoch: BrowserContextEpoch,
    state: OwnedContextState,
    presentation_navigation: PresentationNavigationState,
}

/// Aggregate root for disposable browser-context lifecycle and presentation mutation authority.
#[derive(Debug)]
pub struct BrowserSession {
    id: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    state: BrowserSessionState,
    transport_lost: bool,
    next_epoch: u64,
    next_navigation_generation: u64,
    contexts: BTreeMap<BrowsingContextId, OwnedContextRecord>,
    recovery_evidence: Vec<BrowserSessionRecoveryEvidence>,
    create_recovery_evidence: Vec<DisposableContextCreateRecoveryEvidence>,
}

/// Browser Session composed with the one lifecycle-port instance allowed to mutate its remote state.
///
/// Construction consumes both the aggregate and the concrete port. The port is not exposed mutably and
/// no public Browser Session lifecycle method accepts an arbitrary port parameter. This makes adapter
/// ownership structural rather than dependent on a caller-selected scalar or an adapter callback.
#[must_use = "destroy owned browser state and finish the session, or hand unresolved ownership to recovery"]
pub struct BoundBrowserSession<P> {
    session: BrowserSession,
    port: P,
}

impl<P> fmt::Debug for BoundBrowserSession<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundBrowserSession")
            .field("browser_session", &self.session.id)
            .field("incarnation", &self.session.incarnation)
            .field("state", &self.session.state)
            .field("transport_lost", &self.session.transport_lost)
            .field("owned_context_count", &self.session.contexts.len())
            .field(
                "recovery_evidence_count",
                &self.session.recovery_evidence.len(),
            )
            .field(
                "create_recovery_evidence_count",
                &self.session.create_recovery_evidence.len(),
            )
            .field("port", &"<redacted>")
            .finish()
    }
}

impl<P> Drop for BoundBrowserSession<P> {
    fn drop(&mut self) {
        if self.session.has_unresolved_remote_ownership() {
            let _ = ABANDONED_BOUND_SESSIONS.try_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |value| Some(value.saturating_add(1)),
            );
        }
    }
}

impl BrowserSession {
    /// Start an active Browser Session around an already validated transport session identity.
    ///
    /// A fresh process-local incarnation is allocated before any browser I/O. Exhaustion fails closed
    /// rather than wrapping and making an older authority structurally valid again.
    pub fn start(id: BrowserSessionId) -> Result<Self, BrowserSessionError> {
        Self::start_with_counter(id, &NEXT_BROWSER_SESSION_INCARNATION)
    }

    fn start_with_counter(
        id: BrowserSessionId,
        counter: &AtomicU64,
    ) -> Result<Self, BrowserSessionError> {
        let incarnation = allocate_incarnation(counter)?;
        Ok(Self {
            id,
            incarnation,
            state: BrowserSessionState::Active,
            transport_lost: false,
            next_epoch: 1,
            next_navigation_generation: 1,
            contexts: BTreeMap::new(),
            recovery_evidence: Vec::new(),
            create_recovery_evidence: Vec::new(),
        })
    }

    /// Consume this aggregate and one concrete lifecycle port into a linear bound session.
    ///
    /// Binding invokes no adapter method. All subsequent create/destroy I/O is reachable only through
    /// the owned port inside the returned wrapper.
    pub fn bind_lifecycle_port<P: DisposableContextPort>(self, port: P) -> BoundBrowserSession<P> {
        BoundBrowserSession {
            session: self,
            port,
        }
    }

    /// Return this aggregate's browser-session transport identity.
    #[must_use]
    pub const fn id(&self) -> BrowserSessionId {
        self.id
    }

    /// Return this aggregate's non-reused process-local incarnation.
    #[must_use]
    pub const fn incarnation(&self) -> BrowserSessionIncarnation {
        self.incarnation
    }

    /// Return the current aggregate lifecycle state.
    #[must_use]
    pub const fn state(&self) -> BrowserSessionState {
        self.state
    }

    /// Report whether browser transport loss has been observed for this aggregate.
    #[must_use]
    pub const fn transport_is_lost(&self) -> bool {
        self.transport_lost
    }

    /// Return immutable recovery evidence retained after uncertain browser lifecycle outcomes.
    #[must_use]
    pub fn recovery_evidence(&self) -> &[BrowserSessionRecoveryEvidence] {
        &self.recovery_evidence
    }

    /// Return exact create-attempt recovery facts retained for transaction correlation.
    #[must_use]
    pub fn create_attempt_recovery_evidence(&self) -> &[DisposableContextCreateRecoveryEvidence] {
        &self.create_recovery_evidence
    }

    fn presentation_authority(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let record = self
            .contexts
            .get(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.presentation_navigation != PresentationNavigationState::Established {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        Ok(Self::authority_for(
            self.id,
            self.incarnation,
            &record.handle,
            record.epoch,
        ))
    }

    /// Advance one active, presentation-authorized owned context to a new authority epoch.
    pub fn advance_context_epoch(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.presentation_navigation != PresentationNavigationState::Established {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let next = reserve_epoch(&mut self.next_epoch)?;
        record.epoch = next;
        Ok(Self::authority_for(
            self.id,
            self.incarnation,
            &record.handle,
            next,
        ))
    }

    /// Record browser transport loss independently from ownership-recovery state.
    ///
    /// Returns `true` only for the first observed transport loss. If ownership was already uncertain,
    /// `RecoveryRequired` remains the lifecycle state while the transport-loss fact is retained.
    pub fn record_transport_loss(&mut self) -> bool {
        if self.transport_lost || self.state == BrowserSessionState::Ended {
            return false;
        }
        self.transport_lost = true;
        if self.state == BrowserSessionState::Active {
            self.recovery_evidence.extend(
                self.contexts
                    .values()
                    .filter(|record| record.state == OwnedContextState::Active)
                    .map(|record| {
                        BrowserSessionRecoveryEvidence::TransportLossOwnedHandle(
                            record.handle.clone(),
                        )
                    }),
            );
            self.state = BrowserSessionState::TransportLost;
            self.mark_active_contexts_uncertain();
        }
        true
    }

    /// End the Browser Session only after every owned context has proven destruction.
    pub fn end(&mut self) -> Result<(), BrowserSessionError> {
        self.require_active()?;
        if !self.contexts.is_empty() {
            return Err(BrowserSessionError::ActiveContextRemains);
        }
        self.state = BrowserSessionState::Ended;
        Ok(())
    }

    fn create_disposable_context_with_port<P: DisposableContextPort>(
        &mut self,
        port: &mut P,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let epoch = reserve_epoch(&mut self.next_epoch)?;
        let request = DisposableContextCreateRequest {
            browser_session: self.id,
            incarnation: self.incarnation,
            attempt_epoch: epoch,
        };
        let handle = match port.create_disposable_context(&request) {
            Ok(handle) => handle,
            Err(DisposableContextCreateError::CreateFailedClean) => {
                return Err(BrowserSessionError::ContextCreationFailed);
            }
            Err(DisposableContextCreateError::CreateFailedUncertain(isolation)) => {
                self.create_recovery_evidence.push(
                    DisposableContextCreateRecoveryEvidence::FailedUncertain {
                        attempt_epoch: epoch,
                        isolation: isolation.clone(),
                    },
                );
                if let Some(isolation) = isolation {
                    self.recovery_evidence.push(
                        BrowserSessionRecoveryEvidence::PartialCreationIsolation(isolation),
                    );
                }
                self.enter_recovery_required();
                return Err(BrowserSessionError::ContextCreationUncertain);
            }
        };

        let duplicate_error = if self
            .contexts
            .values()
            .any(|record| record.handle.isolation == handle.isolation)
        {
            Some(BrowserSessionError::DuplicateDisposableIsolation)
        } else if self.contexts.contains_key(&handle.browsing_context) {
            Some(BrowserSessionError::DuplicateBrowsingContext)
        } else {
            None
        };

        if let Some(error) = duplicate_error {
            let completion = DisposableContextCreateCompletion {
                browser_session: self.id,
                incarnation: self.incarnation,
                attempt_epoch: epoch,
                disposition: DisposableContextCreateDisposition::Rejected,
            };
            self.create_recovery_evidence.push(
                DisposableContextCreateRecoveryEvidence::DuplicateCandidate {
                    attempt_epoch: epoch,
                    context: handle.clone(),
                },
            );
            self.recovery_evidence
                .push(BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                    handle.clone(),
                ));
            if port
                .complete_disposable_context_creation(&completion)
                .is_err()
            {
                self.create_recovery_evidence.push(
                    DisposableContextCreateRecoveryEvidence::CompletionUnsettled {
                        attempt_epoch: epoch,
                        disposition: DisposableContextCreateDisposition::Rejected,
                        context: handle.clone(),
                    },
                );
                self.recovery_evidence.push(
                    BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(handle),
                );
                self.enter_recovery_required();
                return Err(BrowserSessionError::ContextCreationUncertain);
            }
            self.enter_recovery_required();
            return Err(error);
        }

        let completion = DisposableContextCreateCompletion {
            browser_session: self.id,
            incarnation: self.incarnation,
            attempt_epoch: epoch,
            disposition: DisposableContextCreateDisposition::Accepted,
        };
        if port
            .complete_disposable_context_creation(&completion)
            .is_err()
        {
            self.create_recovery_evidence.push(
                DisposableContextCreateRecoveryEvidence::CompletionUnsettled {
                    attempt_epoch: epoch,
                    disposition: DisposableContextCreateDisposition::Accepted,
                    context: handle.clone(),
                },
            );
            self.recovery_evidence
                .push(BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(
                    handle,
                ));
            self.enter_recovery_required();
            return Err(BrowserSessionError::ContextCreationUncertain);
        }

        let browsing_context = handle.browsing_context;
        let authority = Self::authority_for(self.id, self.incarnation, &handle, epoch);
        self.contexts.insert(
            browsing_context,
            OwnedContextRecord {
                handle,
                epoch,
                state: OwnedContextState::Active,
                presentation_navigation: PresentationNavigationState::Established,
            },
        );
        Ok(authority)
    }

    fn begin_observed_navigation(
        &mut self,
        incarnation: BrowserSessionIncarnation,
        browsing_context: BrowsingContextId,
        context_epoch: BrowserContextEpoch,
    ) -> Result<NavigationSettlementAuthority, BrowserSessionError> {
        self.require_active()?;
        if incarnation != self.incarnation {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.epoch != context_epoch {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let navigation_generation =
            reserve_navigation_generation(&mut self.next_navigation_generation)?;
        record.presentation_navigation = PresentationNavigationState::Pending {
            navigation_generation,
            committed: false,
        };
        Ok(NavigationSettlementAuthority {
            browser_session: self.id,
            incarnation: self.incarnation,
            browsing_context,
            context_epoch,
            navigation_generation,
        })
    }

    fn current_pending_navigation_mut(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<&mut OwnedContextRecord, BrowserSessionError> {
        self.require_active()?;
        if authority.browser_session != self.id || authority.incarnation != self.incarnation {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let record = self
            .contexts
            .get_mut(&authority.browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.epoch != authority.context_epoch {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        match record.presentation_navigation {
            PresentationNavigationState::Pending {
                navigation_generation,
                ..
            } if navigation_generation == authority.navigation_generation => Ok(record),
            _ => Err(BrowserSessionError::AuthorityMismatch),
        }
    }

    fn mark_observed_navigation_committed(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<(), BrowserSessionError> {
        let record = self.current_pending_navigation_mut(authority)?;
        match record.presentation_navigation {
            PresentationNavigationState::Pending {
                navigation_generation,
                committed: false,
            } => {
                record.presentation_navigation = PresentationNavigationState::Pending {
                    navigation_generation,
                    committed: true,
                };
                Ok(())
            }
            PresentationNavigationState::Pending {
                committed: true, ..
            } => Err(BrowserSessionError::AuthorityMismatch),
            _ => Err(BrowserSessionError::AuthorityMismatch),
        }
    }

    fn close_observed_navigation(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<(), BrowserSessionError> {
        let record = self.current_pending_navigation_mut(authority)?;
        record.presentation_navigation = PresentationNavigationState::Eligible;
        Ok(())
    }

    fn reestablish_presentation_authority_for_context(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.presentation_navigation != PresentationNavigationState::Eligible {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let next = reserve_epoch(&mut self.next_epoch)?;
        record.epoch = next;
        record.presentation_navigation = PresentationNavigationState::Established;
        Ok(Self::authority_for(
            self.id,
            self.incarnation,
            &record.handle,
            next,
        ))
    }

    fn destroy_disposable_context_with_port<P: DisposableContextPort>(
        &mut self,
        authority: &PresentationMutationAuthority,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        let browser_session = self.id;
        let incarnation = self.incarnation;
        let browsing_context = authority.browsing_context;
        let request = {
            let record = self.context_for_authority_mut(authority)?;
            DisposableContextDestroyRequest {
                browser_session,
                incarnation,
                context: record.handle.clone(),
                context_epoch: record.epoch,
            }
        };
        self.destroy_request_with_port(browsing_context, request, port)
    }

    fn destroy_owned_disposable_context_with_port<P: DisposableContextPort>(
        &mut self,
        browsing_context: BrowsingContextId,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        self.require_active()?;
        let request = {
            let record = self
                .contexts
                .get(&browsing_context)
                .filter(|record| record.state == OwnedContextState::Active)
                .ok_or(BrowserSessionError::ContextNotOwned)?;
            DisposableContextDestroyRequest {
                browser_session: self.id,
                incarnation: self.incarnation,
                context: record.handle.clone(),
                context_epoch: record.epoch,
            }
        };
        self.destroy_request_with_port(browsing_context, request, port)
    }

    fn destroy_request_with_port<P: DisposableContextPort>(
        &mut self,
        browsing_context: BrowsingContextId,
        request: DisposableContextDestroyRequest,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        match port.destroy_disposable_context(&request) {
            Ok(()) => {
                let _ = self.contexts.remove(&browsing_context);
                Ok(())
            }
            Err(DisposableContextDestroyError::DestroyFailed) => {
                if let Some(record) = self.contexts.get_mut(&browsing_context) {
                    record.state = OwnedContextState::Uncertain;
                }
                self.recovery_evidence
                    .push(BrowserSessionRecoveryEvidence::UnprovenDestruction {
                        context: request.context,
                        context_epoch: request.context_epoch,
                    });
                self.enter_recovery_required();
                Err(BrowserSessionError::ContextDestructionFailed)
            }
        }
    }

    fn require_active(&self) -> Result<(), BrowserSessionError> {
        if self.state == BrowserSessionState::Active {
            Ok(())
        } else {
            Err(BrowserSessionError::SessionNotActive)
        }
    }

    fn authority_for(
        browser_session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        handle: &DisposableContextHandle,
        context_epoch: BrowserContextEpoch,
    ) -> PresentationMutationAuthority {
        PresentationMutationAuthority {
            browser_session,
            incarnation,
            isolation: handle.isolation.clone(),
            browsing_context: handle.browsing_context,
            context_epoch,
        }
    }

    fn context_for_authority_mut(
        &mut self,
        authority: &PresentationMutationAuthority,
    ) -> Result<&mut OwnedContextRecord, BrowserSessionError> {
        self.require_active()?;
        if authority.browser_session != self.id || authority.incarnation != self.incarnation {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let record = self
            .contexts
            .get_mut(&authority.browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        if record.epoch != authority.context_epoch
            || record.handle.isolation != authority.isolation
            || record.presentation_navigation != PresentationNavigationState::Established
        {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        Ok(record)
    }

    fn enter_recovery_required(&mut self) {
        let sibling_handles = self
            .contexts
            .values()
            .filter(|record| record.state == OwnedContextState::Active)
            .map(|record| record.handle.clone())
            .filter(|handle| {
                !self.recovery_evidence.iter().any(|evidence| match evidence {
                    BrowserSessionRecoveryEvidence::PartialCreationIsolation(_)
                    | BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(_)
                    | BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(_) => false,
                    BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(existing)
                    | BrowserSessionRecoveryEvidence::TransportLossOwnedHandle(existing) => {
                        existing == handle
                    }
                    BrowserSessionRecoveryEvidence::UnprovenDestruction {
                        context: existing,
                        ..
                    } => existing == handle,
                })
            })
            .collect::<Vec<_>>();
        self.recovery_evidence.extend(
            sibling_handles
                .into_iter()
                .map(BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle),
        );
        self.state = BrowserSessionState::RecoveryRequired;
        self.mark_active_contexts_uncertain();
    }

    fn mark_active_contexts_uncertain(&mut self) {
        for record in self.contexts.values_mut() {
            if record.state == OwnedContextState::Active {
                record.state = OwnedContextState::Uncertain;
            }
        }
    }

    fn has_unresolved_remote_ownership(&self) -> bool {
        matches!(self.state, BrowserSessionState::RecoveryRequired)
            || self.contexts.values().any(|record| {
                matches!(record.state, OwnedContextState::Active | OwnedContextState::Uncertain)
            })
    }
}

impl<P: DisposableContextPort> BoundBrowserSession<P> {
    /// Return the bound Browser Session for read-only policy and ACL validation.
    #[must_use]
    pub const fn browser_session(&self) -> &BrowserSession {
        &self.session
    }

    /// Create one disposable context through the exact port consumed when this session was bound.
    pub fn create_disposable_context(
        &mut self,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.session
            .create_disposable_context_with_port(&mut self.port)
    }

    /// Return current presentation authority for an already-owned active context.
    pub fn presentation_authority(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.session.presentation_authority(browsing_context)
    }

    /// Advance one presentation-authorized owned context to a new authority epoch.
    pub fn advance_context_epoch(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.session.advance_context_epoch(browsing_context)
    }

    /// Admit one browser-observed navigation start for the exact current ownership generation.
    ///
    /// Admission revokes presentation mutation immediately, performs no browser I/O, consumes no
    /// presentation epoch, and returns the only witness accepted by later commit or terminal methods.
    pub fn record_observed_navigation(
        &mut self,
        incarnation: BrowserSessionIncarnation,
        browsing_context: BrowsingContextId,
        context_epoch: BrowserContextEpoch,
    ) -> Result<NavigationSettlementAuthority, BrowserSessionError> {
        self.session
            .begin_observed_navigation(incarnation, browsing_context, context_epoch)
    }

    /// Record the first qualified commit-progress event for one current navigation witness.
    ///
    /// Commit is non-terminal and does not create presentation re-establishment eligibility.
    pub fn record_observed_navigation_committed(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<(), BrowserSessionError> {
        self.session.mark_observed_navigation_committed(authority)
    }

    /// Close one current navigation through a complete positive browser observation.
    pub fn record_observed_navigation_settled(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<(), BrowserSessionError> {
        self.session.close_observed_navigation(authority)
    }

    /// Close one current navigation through an explicit typed negative browser outcome.
    pub fn record_observed_navigation_terminated(
        &mut self,
        authority: &NavigationSettlementAuthority,
        outcome: NavigationTerminationOutcome,
    ) -> Result<(), BrowserSessionError> {
        match outcome {
            NavigationTerminationOutcome::Aborted | NavigationTerminationOutcome::Failed => {
                self.session.close_observed_navigation(authority)
            }
        }
    }

    /// Close one current navigation's liveness boundary when download start is observed.
    ///
    /// This does not claim file completion, persistence, scanning, egress authorization, or any
    /// download-security outcome; those remain outside Browser Session.
    pub fn record_observed_navigation_download_started(
        &mut self,
        authority: &NavigationSettlementAuthority,
    ) -> Result<(), BrowserSessionError> {
        self.session.close_observed_navigation(authority)
    }

    /// Explicitly mint the next presentation authority after one qualified navigation closure.
    ///
    /// This is the only navigation path that consumes a new presentation epoch. The opportunity is
    /// single-use; a newer navigation start supersedes any unused opportunity for the same context.
    pub fn reestablish_presentation_authority(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.session
            .reestablish_presentation_authority_for_context(browsing_context)
    }

    /// Destroy the exact owned disposable boundary through current presentation authority.
    pub fn destroy_disposable_context(
        &mut self,
        authority: &PresentationMutationAuthority,
    ) -> Result<(), BrowserSessionError> {
        self.session
            .destroy_disposable_context_with_port(authority, &mut self.port)
    }

    /// Destroy an owned disposable boundary through structural lifecycle custody.
    ///
    /// This cleanup path remains available while navigation has revoked presentation mutation, but it
    /// can select only a context currently owned by this exact bound aggregate. It never re-establishes
    /// presentation authority and consumes the retained browser-issued lifecycle handle on success.
    pub fn destroy_owned_disposable_context(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<(), BrowserSessionError> {
        self.session
            .destroy_owned_disposable_context_with_port(browsing_context, &mut self.port)
    }

    /// Record browser transport loss without exposing mutable lifecycle-port access.
    pub fn record_transport_loss(&mut self) -> bool {
        self.session.record_transport_loss()
    }

    /// End the Browser Session only after every owned context has proven destruction.
    pub fn end(&mut self) -> Result<(), BrowserSessionError> {
        self.session.end()
    }

    /// Verify normal completion without relinquishing the exact bound lifecycle owner on failure.
    ///
    /// A rejected finish leaves the wrapper intact so the caller can destroy or reconcile outstanding
    /// contexts and retry. After success the aggregate is `Ended`; dropping the wrapper is then inert.
    pub fn finish(&mut self) -> Result<(), BrowserSessionError> {
        self.session.end()
    }

    /// Route one crate-internal recovery operation through the exact retained adapter.
    ///
    /// The callback is deliberately crate-private: external consumers never receive the raw adapter,
    /// while recovery custody can bind one purpose-bounded request to the same adapter instance that
    /// performed lifecycle creation and destruction.
    pub(crate) fn dispatch_recovery_operation<R>(
        &mut self,
        dispatch: impl FnOnce(&BrowserSession, &mut P) -> R,
    ) -> R {
        dispatch(&self.session, &mut self.port)
    }
}

impl<P: AuthorizedContextOperationPort> BoundBrowserSession<P> {
    /// Execute one adapter-defined operation through the exact consumed adapter after authority validation.
    ///
    /// Browser Session validates session incarnation, isolation identity, browsing-context identity,
    /// current presentation state, and epoch before the adapter receives the operation. Stale or foreign
    /// authority therefore fails before adapter I/O, while the adapter-specific operation vocabulary
    /// remains outside this domain.
    pub fn execute_authorized_context_operation(
        &mut self,
        authority: &PresentationMutationAuthority,
        operation: P::Operation,
    ) -> Result<P::Output, AuthorizedContextOperationError<P::Error>> {
        let browser_session = self.session.id;
        let incarnation = self.session.incarnation;
        let record = self
            .session
            .context_for_authority_mut(authority)
            .map_err(AuthorizedContextOperationError::BrowserSession)?;
        let context = record.handle.clone();
        let context_epoch = record.epoch;
        let request = AuthorizedContextOperationRequest {
            browser_session,
            incarnation,
            context,
            context_epoch,
            operation,
        };
        self.port
            .execute_authorized_context_operation(&request)
            .map_err(AuthorizedContextOperationError::Adapter)
    }
}

fn reserve_epoch(next_epoch: &mut u64) -> Result<BrowserContextEpoch, BrowserSessionError> {
    let epoch = BrowserContextEpoch(*next_epoch);
    *next_epoch = next_epoch
        .checked_add(1)
        .ok_or(BrowserSessionError::EpochExhausted)?;
    Ok(epoch)
}

fn reserve_navigation_generation(next_generation: &mut u64) -> Result<u64, BrowserSessionError> {
    let generation = *next_generation;
    *next_generation = next_generation
        .checked_add(1)
        .ok_or(BrowserSessionError::EpochExhausted)?;
    Ok(generation)
}

fn allocate_incarnation(
    counter: &AtomicU64,
) -> Result<BrowserSessionIncarnation, BrowserSessionError> {
    let value = counter
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            current.checked_add(1)
        })
        .map_err(|_| BrowserSessionError::IncarnationExhausted)?;
    Ok(BrowserSessionIncarnation(value))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug)]
    struct TestPort {
        handles: VecDeque<DisposableContextHandle>,
        create_error: Option<DisposableContextCreateError>,
        fail_destroy: bool,
        fail_completion: bool,
        create_calls: usize,
        destroy_calls: usize,
        create_sessions: Vec<BrowserSessionId>,
        create_incarnations: Vec<BrowserSessionIncarnation>,
        create_attempts: Vec<BrowserContextEpoch>,
        create_completions: Vec<(
            BrowserSessionId,
            BrowserSessionIncarnation,
            BrowserContextEpoch,
            DisposableContextCreateDisposition,
        )>,
        destroy_sessions: Vec<BrowserSessionId>,
        destroy_incarnations: Vec<BrowserSessionIncarnation>,
        destroyed_isolations: Vec<DisposableIsolationId>,
    }

    impl TestPort {
        fn new(context: u64, isolation: &str) -> Self {
            Self::with_handles(vec![DisposableContextHandle::new(
                isolation_id(isolation),
                context_id(context),
            )])
        }

        fn with_handles(handles: Vec<DisposableContextHandle>) -> Self {
            Self {
                handles: handles.into(),
                create_error: None,
                fail_destroy: false,
                fail_completion: false,
                create_calls: 0,
                destroy_calls: 0,
                create_sessions: Vec::new(),
                create_incarnations: Vec::new(),
                create_attempts: Vec::new(),
                create_completions: Vec::new(),
                destroy_sessions: Vec::new(),
                destroy_incarnations: Vec::new(),
                destroyed_isolations: Vec::new(),
            }
        }
    }

    impl DisposableContextPort for TestPort {
        fn create_disposable_context(
            &mut self,
            request: &DisposableContextCreateRequest,
        ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
            self.create_calls += 1;
            self.create_sessions.push(request.browser_session());
            self.create_incarnations.push(request.incarnation());
            self.create_attempts.push(request.attempt_epoch());
            match self.create_error.clone() {
                Some(error) => Err(error),
                None => Ok(self
                    .handles
                    .pop_front()
                    .expect("test must provide one handle per successful creation")),
            }
        }

        fn complete_disposable_context_creation(
            &mut self,
            completion: &DisposableContextCreateCompletion,
        ) -> Result<(), DisposableContextCreateCompletionError> {
            self.create_completions.push((
                completion.browser_session(),
                completion.incarnation(),
                completion.attempt_epoch(),
                completion.disposition(),
            ));
            if self.fail_completion {
                Err(DisposableContextCreateCompletionError::CompletionFailed)
            } else {
                Ok(())
            }
        }

        fn destroy_disposable_context(
            &mut self,
            request: &DisposableContextDestroyRequest,
        ) -> Result<(), DisposableContextDestroyError> {
            self.destroy_calls += 1;
            self.destroy_sessions.push(request.browser_session());
            self.destroy_incarnations.push(request.incarnation());
            self.destroyed_isolations
                .push(request.context().isolation.clone());
            if self.fail_destroy {
                Err(DisposableContextDestroyError::DestroyFailed)
            } else {
                Ok(())
            }
        }
    }

    fn session_id(value: u64) -> BrowserSessionId {
        BrowserSessionId::new(value).expect("valid session id")
    }

    fn context_id(value: u64) -> BrowsingContextId {
        BrowsingContextId::new(value).expect("valid context id")
    }

    fn isolation_id(value: &str) -> DisposableIsolationId {
        DisposableIsolationId::parse(value).expect("protocol text representation is infallible")
    }

    fn session(value: u64) -> BrowserSession {
        BrowserSession::start(session_id(value)).expect("incarnation capacity")
    }

    #[test]
    fn isolation_identity_preserves_protocol_text_without_domain_grammar() {
        let cases = [
            String::new(),
            " user-context ".to_owned(),
            "user\ncontext".to_owned(),
            "x".repeat(4097),
            "webdriver-user-context-10".to_owned(),
        ];

        for remote_user_context in cases {
            let identity = DisposableIsolationId::parse(&remote_user_context)
                .expect("protocol text representation is infallible");
            assert_eq!(identity.as_str(), remote_user_context);
        }

        let valid = isolation_id("webdriver-user-context-10");
        let handle = DisposableContextHandle::new(valid.clone(), context_id(10));
        assert_eq!(handle.isolation(), &valid);
        assert_eq!(handle.browsing_context(), context_id(10));
    }

    #[test]
    fn bound_creation_is_the_only_raw_context_entry_to_authority() {
        let raw_session = session(1);
        assert_eq!(raw_session.id(), session_id(1));
        assert_ne!(raw_session.incarnation().value(), 0);
        assert!(!raw_session.transport_is_lost());
        assert!(raw_session.recovery_evidence().is_empty());
        assert_eq!(
            raw_session.presentation_authority(context_id(10)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        let mut bound = raw_session.bind_lifecycle_port(TestPort::new(10, "isolation-10"));
        let authority = bound
            .create_disposable_context()
            .expect("owned disposable context");
        assert_eq!(bound.port.create_sessions, vec![session_id(1)]);
        assert_eq!(
            bound.port.create_incarnations,
            vec![bound.browser_session().incarnation()]
        );
        assert_eq!(bound.port.create_attempts, vec![BrowserContextEpoch(1)]);
        assert_eq!(
            bound.port.create_completions,
            vec![(
                session_id(1),
                bound.browser_session().incarnation(),
                BrowserContextEpoch(1),
                DisposableContextCreateDisposition::Accepted,
            )]
        );
        assert_eq!(authority.browser_session(), session_id(1));
        assert_eq!(
            authority.incarnation(),
            bound.browser_session().incarnation()
        );
        assert_eq!(authority.isolation().as_str(), "isolation-10");
        assert_eq!(authority.browsing_context(), context_id(10));
        assert_eq!(authority.context_epoch().value(), 1);
        assert_eq!(bound.presentation_authority(context_id(10)), Ok(authority));
    }

    #[test]
    fn creation_failure_preserves_known_recovery_identity() {
        let mut clean_port = TestPort::new(20, "isolation-20");
        clean_port.create_error = Some(DisposableContextCreateError::CreateFailedClean);
        let mut clean = session(2).bind_lifecycle_port(clean_port);
        assert_eq!(
            clean.create_disposable_context(),
            Err(BrowserSessionError::ContextCreationFailed)
        );
        assert_eq!(clean.browser_session().state(), BrowserSessionState::Active);
        clean.end().expect("clean failure can end");

        let mut unknown_port = TestPort::new(210, "isolation-210");
        unknown_port.create_error = Some(DisposableContextCreateError::CreateFailedUncertain(None));
        let mut unknown = session(21).bind_lifecycle_port(unknown_port);
        assert_eq!(
            unknown.create_disposable_context(),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert!(unknown.browser_session().recovery_evidence().is_empty());
        assert_eq!(unknown.browser_session().create_attempt_recovery_evidence().len(), 1);
        match &unknown.browser_session().create_attempt_recovery_evidence()[0] {
            DisposableContextCreateRecoveryEvidence::FailedUncertain {
                attempt_epoch,
                isolation,
            } => {
                assert_eq!(attempt_epoch.value(), 1);
                assert_eq!(isolation, &None);
            }
            other => panic!("unexpected recovery evidence: {other:?}"),
        }

        let known = isolation_id("partial-user-context-211");
        let mut known_port = TestPort::new(211, "unused");
        known_port.create_error = Some(DisposableContextCreateError::CreateFailedUncertain(Some(
            known.clone(),
        )));
        let mut known_session = session(22).bind_lifecycle_port(known_port);
        assert_eq!(
            known_session.create_disposable_context(),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            known_session.browser_session().recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::PartialCreationIsolation(
                known.clone()
            )]
        );
        assert_eq!(known_session.browser_session().create_attempt_recovery_evidence().len(), 1);
        match &known_session.browser_session().create_attempt_recovery_evidence()[0] {
            DisposableContextCreateRecoveryEvidence::FailedUncertain {
                attempt_epoch,
                isolation,
            } => {
                assert_eq!(attempt_epoch.value(), 1);
                assert_eq!(isolation.as_ref(), Some(&known));
            }
            other => panic!("unexpected recovery evidence: {other:?}"),
        }
        assert_eq!(
            known_session.end(),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    #[test]
    fn duplicate_adapter_output_preserves_offending_handle() {
        let first_context_handle =
            DisposableContextHandle::new(isolation_id("isolation-30-a"), context_id(30));
        let duplicate_context_handle =
            DisposableContextHandle::new(isolation_id("isolation-30-b"), context_id(30));
        let context_port = TestPort::with_handles(vec![
            first_context_handle.clone(),
            duplicate_context_handle.clone(),
        ]);
        let mut duplicate_context = session(3).bind_lifecycle_port(context_port);
        duplicate_context
            .create_disposable_context()
            .expect("first owned context");
        assert_eq!(
            duplicate_context.create_disposable_context(),
            Err(BrowserSessionError::DuplicateBrowsingContext)
        );
        assert_eq!(
            duplicate_context.browser_session().recovery_evidence(),
            &[
                BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(duplicate_context_handle),
                BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(first_context_handle),
            ]
        );

        let first_isolation_handle =
            DisposableContextHandle::new(isolation_id("isolation-31"), context_id(310));
        let duplicate_isolation_handle =
            DisposableContextHandle::new(isolation_id("isolation-31"), context_id(311));
        let isolation_port = TestPort::with_handles(vec![
            first_isolation_handle.clone(),
            duplicate_isolation_handle.clone(),
        ]);
        let mut duplicate_isolation = session(31).bind_lifecycle_port(isolation_port);
        duplicate_isolation
            .create_disposable_context()
            .expect("first owned isolation");
        assert_eq!(
            duplicate_isolation.create_disposable_context(),
            Err(BrowserSessionError::DuplicateDisposableIsolation)
        );
        assert_eq!(
            duplicate_isolation.browser_session().recovery_evidence(),
            &[
                BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(duplicate_isolation_handle),
                BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(first_isolation_handle),
            ]
        );
    }

    #[test]
    fn create_completion_failure_preserves_non_authorizing_recovery_evidence() {
        let expected = DisposableContextHandle::new(isolation_id("isolation-315"), context_id(315));
        let mut port = TestPort::with_handles(vec![expected.clone()]);
        port.fail_completion = true;
        let mut bound = session(315).bind_lifecycle_port(port);

        assert_eq!(
            bound.create_disposable_context(),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            bound.browser_session().state(),
            BrowserSessionState::RecoveryRequired
        );
        assert_eq!(
            bound.browser_session().recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(
                expected.clone()
            )]
        );
        assert_eq!(bound.browser_session().create_attempt_recovery_evidence().len(), 1);
        match &bound.browser_session().create_attempt_recovery_evidence()[0] {
            DisposableContextCreateRecoveryEvidence::CompletionUnsettled {
                attempt_epoch,
                disposition,
                context,
            } => {
                assert_eq!(attempt_epoch.value(), 1);
                assert_eq!(*disposition, DisposableContextCreateDisposition::Accepted);
                assert_eq!(context, &expected);
            }
            other => panic!("unexpected recovery evidence: {other:?}"),
        }
        assert_eq!(
            bound.port.create_completions[0].3,
            DisposableContextCreateDisposition::Accepted
        );
        assert_eq!(
            bound.presentation_authority(context_id(315)),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    #[test]
    fn rejected_create_completion_failure_preserves_duplicate_and_unsettled_evidence() {
        let first = DisposableContextHandle::new(isolation_id("isolation-316-a"), context_id(316));
        let duplicate =
            DisposableContextHandle::new(isolation_id("isolation-316-b"), context_id(316));
        let port = TestPort::with_handles(vec![first.clone(), duplicate.clone()]);
        let mut bound = session(316).bind_lifecycle_port(port);

        bound
            .create_disposable_context()
            .expect("first candidate accepted");
        bound.port.fail_completion = true;
        assert_eq!(
            bound.create_disposable_context(),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            bound.browser_session().recovery_evidence(),
            &[
                BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(duplicate.clone()),
                BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(duplicate.clone()),
                BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(first),
            ]
        );
        assert_eq!(bound.browser_session().create_attempt_recovery_evidence().len(), 2);
        match &bound.browser_session().create_attempt_recovery_evidence()[0] {
            DisposableContextCreateRecoveryEvidence::DuplicateCandidate {
                attempt_epoch,
                context,
            } => {
                assert_eq!(attempt_epoch.value(), 2);
                assert_eq!(context, &duplicate);
            }
            other => panic!("unexpected recovery evidence: {other:?}"),
        }
        match &bound.browser_session().create_attempt_recovery_evidence()[1] {
            DisposableContextCreateRecoveryEvidence::CompletionUnsettled {
                attempt_epoch,
                disposition,
                context,
            } => {
                assert_eq!(attempt_epoch.value(), 2);
                assert_eq!(*disposition, DisposableContextCreateDisposition::Rejected);
                assert_eq!(context, &duplicate);
            }
            other => panic!("unexpected recovery evidence: {other:?}"),
        }
        assert_eq!(
            bound.port.create_completions[1].3,
            DisposableContextCreateDisposition::Rejected
        );
    }

    #[test]
    fn bound_port_is_structural_and_not_swappable() {
        let approved = TestPort::new(320, "isolation-320");
        let other = TestPort::new(321, "isolation-321");
        let mut bound = session(32).bind_lifecycle_port(approved);
        let authority = bound
            .create_disposable_context()
            .expect("owned context uses consumed port");
        assert_eq!(other.create_calls, 0);
        assert_eq!(other.destroy_calls, 0);
        bound
            .destroy_disposable_context(&authority)
            .expect("same structurally bound port destroys context");
        assert_eq!(bound.port.create_calls, 1);
        assert_eq!(bound.port.destroy_calls, 1);
    }

    #[test]
    fn epoch_exhaustion_prevents_creation_io() {
        let mut bound = session(4).bind_lifecycle_port(TestPort::new(40, "isolation-40"));
        bound.session.next_epoch = u64::MAX;
        assert_eq!(
            bound.create_disposable_context(),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(bound.port.create_calls, 0);
    }

    #[test]
    fn epoch_exhaustion_prevents_advance_mutation() {
        let mut bound = session(41).bind_lifecycle_port(TestPort::new(410, "isolation-410"));
        let authority = bound.create_disposable_context().expect("owned context");
        bound.session.next_epoch = u64::MAX;
        assert_eq!(
            bound.advance_context_epoch(context_id(410)),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(bound.presentation_authority(context_id(410)), Ok(authority));
    }

    #[test]
    fn epoch_advance_invalidates_old_and_unknown_authority() {
        let mut bound = session(5).bind_lifecycle_port(TestPort::new(50, "isolation-50"));
        let old = bound.create_disposable_context().expect("owned context");
        assert_eq!(
            bound.advance_context_epoch(context_id(51)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        let new = bound
            .advance_context_epoch(context_id(50))
            .expect("advanced epoch");
        assert_eq!(new.context_epoch().value(), 2);
        assert_eq!(
            bound.destroy_disposable_context(&old),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        bound
            .destroy_disposable_context(&new)
            .expect("destroy current epoch");
        assert_eq!(
            bound.port.destroy_incarnations,
            vec![bound.browser_session().incarnation()]
        );
        assert_eq!(
            bound.presentation_authority(context_id(50)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            bound.destroy_disposable_context(&new),
            Err(BrowserSessionError::ContextNotOwned)
        );
    }

    #[test]
    fn cross_session_and_foreign_isolation_authority_fail_before_io() {
        let mut owner = session(6).bind_lifecycle_port(TestPort::new(60, "isolation-60"));
        let authority = owner.create_disposable_context().expect("owner context");

        let mut foreign = session(7).bind_lifecycle_port(TestPort::new(60, "isolation-60"));
        foreign
            .create_disposable_context()
            .expect("foreign context");
        assert_eq!(
            foreign.destroy_disposable_context(&authority),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(foreign.port.destroy_calls, 0);

        let forged = PresentationMutationAuthority {
            browser_session: owner.browser_session().id(),
            incarnation: owner.browser_session().incarnation(),
            isolation: isolation_id("foreign-isolation"),
            browsing_context: authority.browsing_context(),
            context_epoch: authority.context_epoch(),
        };
        assert_eq!(
            owner.destroy_disposable_context(&forged),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(owner.port.destroy_calls, 0);
    }

    #[test]
    fn sequential_incarnation_reuse_rejects_stale_authority() {
        let shared_id = session_id(8);
        let mut session_a = BrowserSession::start(shared_id)
            .expect("A incarnation")
            .bind_lifecycle_port(TestPort::new(80, "reused-user-context"));
        let authority_a = session_a.create_disposable_context().expect("A context");
        session_a
            .destroy_disposable_context(&authority_a)
            .expect("A destroy");
        session_a.end().expect("A end");

        let mut session_b = BrowserSession::start(shared_id)
            .expect("B incarnation")
            .bind_lifecycle_port(TestPort::new(80, "reused-user-context"));
        let authority_b = session_b.create_disposable_context().expect("B context");
        assert_ne!(
            session_a.browser_session().incarnation(),
            session_b.browser_session().incarnation()
        );
        assert_eq!(
            session_b.destroy_disposable_context(&authority_a),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(session_b.port.destroy_calls, 0);
        session_b
            .destroy_disposable_context(&authority_b)
            .expect("B destroy");
        assert_eq!(session_b.port.destroy_calls, 1);
    }

    #[test]
    fn destroy_failure_retains_handle_and_transport_loss_orthogonally() {
        let expected_handle =
            DisposableContextHandle::new(isolation_id("isolation-90"), context_id(90));
        let mut port = TestPort::new(90, "isolation-90");
        port.fail_destroy = true;
        let mut bound = session(9).bind_lifecycle_port(port);
        let authority = bound.create_disposable_context().expect("owned context");
        let expected_epoch = authority.context_epoch();
        assert_eq!(
            bound.destroy_disposable_context(&authority),
            Err(BrowserSessionError::ContextDestructionFailed)
        );
        assert_eq!(
            bound.browser_session().state(),
            BrowserSessionState::RecoveryRequired
        );
        assert_eq!(
            bound.browser_session().recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::UnprovenDestruction {
                context: expected_handle,
                context_epoch: expected_epoch,
            }]
        );
        assert!(!bound.browser_session().transport_is_lost());
        assert!(bound.record_transport_loss());
        assert!(bound.browser_session().transport_is_lost());
        assert_eq!(
            bound.browser_session().state(),
            BrowserSessionState::RecoveryRequired
        );
        assert!(!bound.record_transport_loss());
        assert_eq!(
            bound.create_disposable_context(),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            bound.presentation_authority(context_id(90)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            bound.advance_context_epoch(context_id(90)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(bound.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn transport_loss_invalidates_active_contexts_and_is_idempotent() {
        let mut bound = session(10).bind_lifecycle_port(TestPort::new(100, "isolation-100"));
        let authority = bound.create_disposable_context().expect("owned context");
        assert!(bound.record_transport_loss());
        assert_eq!(
            bound.browser_session().state(),
            BrowserSessionState::TransportLost
        );
        assert!(bound.browser_session().transport_is_lost());
        assert!(!bound.record_transport_loss());
        assert_eq!(
            bound.destroy_disposable_context(&authority),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(bound.port.destroy_calls, 0);
    }

    #[test]
    fn normal_end_requires_proven_destruction_and_ignores_late_transport_report() {
        let mut bound = session(11).bind_lifecycle_port(TestPort::new(110, "isolation-110"));
        let authority = bound.create_disposable_context().expect("owned context");
        assert_eq!(bound.end(), Err(BrowserSessionError::ActiveContextRemains));
        bound
            .destroy_disposable_context(&authority)
            .expect("proven destruction");
        assert_eq!(bound.port.destroy_sessions, vec![session_id(11)]);
        assert_eq!(
            bound.port.destroyed_isolations,
            vec![isolation_id("isolation-110")]
        );
        bound.end().expect("normal end");
        assert_eq!(bound.browser_session().state(), BrowserSessionState::Ended);
        assert!(!bound.record_transport_loss());
        assert_eq!(bound.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn navigation_state_machine_separates_presentation_from_lifecycle_cleanup() {
        let mut bound = session(13).bind_lifecycle_port(TestPort::new(130, "isolation-130"));
        let initial = bound.create_disposable_context().expect("owned context");
        let calls_before_navigation = bound.port.destroy_calls;
        let pending = bound
            .record_observed_navigation(
                initial.incarnation(),
                initial.browsing_context(),
                initial.context_epoch(),
            )
            .expect("navigation start");
        assert_eq!(
            bound.presentation_authority(initial.browsing_context()),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(
            bound.destroy_disposable_context(&initial),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(bound.port.destroy_calls, calls_before_navigation);
        bound
            .record_observed_navigation_committed(&pending)
            .expect("first commit progress");
        assert_eq!(
            bound.record_observed_navigation_committed(&pending),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        bound
            .record_observed_navigation_settled(&pending)
            .expect("positive terminal");
        assert_eq!(
            bound.record_observed_navigation_download_started(&pending),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        let fresh = bound
            .reestablish_presentation_authority(initial.browsing_context())
            .expect("explicit re-establishment");
        assert_eq!(fresh.context_epoch().value(), initial.context_epoch().value() + 1);
        assert_eq!(
            bound.reestablish_presentation_authority(initial.browsing_context()),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        bound
            .destroy_owned_disposable_context(initial.browsing_context())
            .expect("bound lifecycle owner cleanup");
        assert_eq!(bound.port.destroy_calls, calls_before_navigation + 1);
        assert_eq!(
            bound.destroy_owned_disposable_context(initial.browsing_context()),
            Err(BrowserSessionError::ContextNotOwned)
        );
    }

    #[test]
    fn navigation_generation_rejects_foreign_stale_and_invalid_evidence_without_epoch_spend() {
        let handles = vec![
            DisposableContextHandle::new(isolation_id("isolation-140"), context_id(140)),
            DisposableContextHandle::new(isolation_id("isolation-141"), context_id(141)),
        ];
        let mut bound = session(14).bind_lifecycle_port(TestPort::with_handles(handles));
        let first = bound.create_disposable_context().expect("first context");
        let second = bound.create_disposable_context().expect("second context");
        assert_eq!(
            bound.record_observed_navigation(
                BrowserSessionIncarnation(first.incarnation().value() + 1),
                first.browsing_context(),
                first.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(
            bound.record_observed_navigation(
                first.incarnation(),
                context_id(999),
                first.context_epoch(),
            ),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            bound.record_observed_navigation(
                first.incarnation(),
                first.browsing_context(),
                second.context_epoch(),
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        let old_pending = bound
            .record_observed_navigation(
                first.incarnation(),
                first.browsing_context(),
                first.context_epoch(),
            )
            .expect("first pending");
        let current_pending = bound
            .record_observed_navigation(
                first.incarnation(),
                first.browsing_context(),
                first.context_epoch(),
            )
            .expect("superseding pending");
        assert_eq!(
            bound.record_observed_navigation_settled(&old_pending),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        bound
            .record_observed_navigation_terminated(
                &current_pending,
                NavigationTerminationOutcome::Aborted,
            )
            .expect("current negative terminal");
        let fresh = bound
            .reestablish_presentation_authority(first.browsing_context())
            .expect("fresh authority");
        assert_eq!(fresh.context_epoch().value(), second.context_epoch().value() + 1);
        assert_eq!(
            bound.record_observed_navigation_terminated(
                &current_pending,
                NavigationTerminationOutcome::Failed,
            ),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(bound.presentation_authority(first.browsing_context()), Ok(fresh));
    }

    #[test]
    fn navigation_exhaustion_and_cleanup_failure_fail_closed_without_hidden_mutation() {
        let mut bound = session(15).bind_lifecycle_port(TestPort::new(150, "isolation-150"));
        let initial = bound.create_disposable_context().expect("owned context");
        bound.session.next_navigation_generation = u64::MAX;
        assert_eq!(
            bound.record_observed_navigation(
                initial.incarnation(),
                initial.browsing_context(),
                initial.context_epoch(),
            ),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(
            bound.presentation_authority(initial.browsing_context()),
            Ok(initial.clone())
        );
        bound.session.next_navigation_generation = 1;
        let pending = bound
            .record_observed_navigation(
                initial.incarnation(),
                initial.browsing_context(),
                initial.context_epoch(),
            )
            .expect("pending navigation");
        bound
            .record_observed_navigation_download_started(&pending)
            .expect("download liveness closure");
        bound.session.next_epoch = u64::MAX;
        assert_eq!(
            bound.reestablish_presentation_authority(initial.browsing_context()),
            Err(BrowserSessionError::EpochExhausted)
        );
        bound.port.fail_destroy = true;
        assert_eq!(
            bound.destroy_owned_disposable_context(initial.browsing_context()),
            Err(BrowserSessionError::ContextDestructionFailed)
        );
        assert_eq!(bound.browser_session().state(), BrowserSessionState::RecoveryRequired);
        assert_eq!(
            bound.destroy_owned_disposable_context(initial.browsing_context()),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            bound.record_observed_navigation_settled(&pending),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    #[test]
    fn incarnation_allocator_fails_closed_before_wrap() {
        let counter = AtomicU64::new(u64::MAX);
        let error = BrowserSession::start_with_counter(session_id(12), &counter)
            .expect_err("incarnation allocation must fail closed before wrapping");
        assert_eq!(error, BrowserSessionError::IncarnationExhausted);
    }
}
