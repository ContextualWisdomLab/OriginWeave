//! Browser Session lifecycle authority for OriginWeave.
//!
//! This crate owns the domain transition that turns a newly created disposable
//! browser isolation boundary into presentation-mutation authority. Driver identifiers
//! remain adapter data: naming a session or browsing context is never sufficient to mint authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use originweave_core::{BrowserSessionId, BrowsingContextId};

static NEXT_BROWSER_SESSION_INCARNATION: AtomicU64 = AtomicU64::new(1);

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
    /// No unused context epoch remains, so no new authority can be issued safely.
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

/// Validation failure for a browser-issued disposable isolation identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableIsolationIdError {
    /// The identity is empty.
    Empty,
    /// The identity exceeds the bounded adapter evidence size.
    TooLong,
    /// The identity contains surrounding whitespace or control characters.
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
    /// Parse one bounded browser-issued isolation identity.
    pub fn parse(value: &str) -> Result<Self, DisposableIsolationIdError> {
        if value.is_empty() {
            return Err(DisposableIsolationIdError::Empty);
        }
        if value.len() > 4096 {
            return Err(DisposableIsolationIdError::TooLong);
        }
        if value.trim() != value || value.chars().any(char::is_control) {
            return Err(DisposableIsolationIdError::InvalidCharacter);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the validated browser-issued isolation identity.
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
    /// Destruction of this exact owned handle failed or could not be proven.
    UnprovenDestruction(DisposableContextHandle),
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

    /// Return the Browser Session incarnation for adapter correlation.
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
/// validating the supplied presentation authority against current ownership. A caller cannot rebuild
/// cleanup authority from raw browser identifiers.
#[derive(Debug)]
pub struct DisposableContextDestroyRequest {
    browser_session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    context: DisposableContextHandle,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnedContextState {
    Active,
    Destroyed,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedContextRecord {
    handle: DisposableContextHandle,
    epoch: BrowserContextEpoch,
    state: OwnedContextState,
}

/// Aggregate root for disposable browser-context lifecycle and presentation mutation authority.
#[derive(Debug)]
pub struct BrowserSession {
    id: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    state: BrowserSessionState,
    transport_lost: bool,
    next_epoch: u64,
    contexts: BTreeMap<BrowsingContextId, OwnedContextRecord>,
    recovery_evidence: Vec<BrowserSessionRecoveryEvidence>,
}

/// Browser Session composed with the one lifecycle-port instance allowed to mutate its remote state.
///
/// Construction consumes both the aggregate and the concrete port. The port is not exposed mutably and
/// no public Browser Session lifecycle method accepts an arbitrary port parameter. This makes adapter
/// ownership structural rather than dependent on a caller-selected scalar or an adapter callback.
#[derive(Debug)]
pub struct BoundBrowserSession<P> {
    session: BrowserSession,
    port: P,
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
            contexts: BTreeMap::new(),
            recovery_evidence: Vec::new(),
        })
    }

    /// Consume this aggregate and one concrete lifecycle port into a linear bound session.
    ///
    /// Binding invokes no adapter method. All subsequent create/destroy I/O is reachable only through
    /// the owned port inside the returned wrapper.
    #[must_use]
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

    /// Return current presentation authority for an already-owned active context.
    pub fn presentation_authority(
        &self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let record = self
            .contexts
            .get(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        Ok(Self::authority_for(
            self.id,
            self.incarnation,
            &record.handle,
            record.epoch,
        ))
    }

    /// Advance one active owned context to a new authority epoch.
    pub fn advance_context_epoch(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let browser_session = self.id;
        let incarnation = self.incarnation;
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        let next = reserve_epoch(&mut self.next_epoch)?;
        record.epoch = next;
        Ok(Self::authority_for(
            browser_session,
            incarnation,
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
            self.state = BrowserSessionState::TransportLost;
            self.mark_active_contexts_uncertain();
        }
        true
    }

    /// End the Browser Session only after every owned context has proven destruction.
    pub fn end(&mut self) -> Result<(), BrowserSessionError> {
        self.require_active()?;
        if self
            .contexts
            .values()
            .any(|record| record.state != OwnedContextState::Destroyed)
        {
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
            self.recovery_evidence
                .push(BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                    handle.clone(),
                ));
            if port
                .complete_disposable_context_creation(&completion)
                .is_err()
            {
                self.recovery_evidence
                    .push(BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(
                        handle,
                    ));
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
            },
        );
        Ok(authority)
    }

    fn destroy_disposable_context_with_port<P: DisposableContextPort>(
        &mut self,
        authority: &PresentationMutationAuthority,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        let browser_session = self.id;
        let incarnation = self.incarnation;
        let record = self.context_for_authority_mut(authority)?;
        let request = DisposableContextDestroyRequest {
            browser_session,
            incarnation,
            context: record.handle.clone(),
        };
        match port.destroy_disposable_context(&request) {
            Ok(()) => {
                record.state = OwnedContextState::Destroyed;
                Ok(())
            }
            Err(DisposableContextDestroyError::DestroyFailed) => {
                record.state = OwnedContextState::Uncertain;
                self.recovery_evidence
                    .push(BrowserSessionRecoveryEvidence::UnprovenDestruction(
                        request.context,
                    ));
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
        if record.epoch != authority.context_epoch || record.handle.isolation != authority.isolation
        {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        Ok(record)
    }

    fn enter_recovery_required(&mut self) {
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

    /// Advance one active owned context to a new authority epoch.
    pub fn advance_context_epoch(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.session.advance_context_epoch(browsing_context)
    }

    /// Destroy the exact owned disposable boundary through the bound lifecycle port.
    pub fn destroy_disposable_context(
        &mut self,
        authority: &PresentationMutationAuthority,
    ) -> Result<(), BrowserSessionError> {
        self.session
            .destroy_disposable_context_with_port(authority, &mut self.port)
    }

    /// Record browser transport loss without exposing mutable lifecycle-port access.
    pub fn record_transport_loss(&mut self) -> bool {
        self.session.record_transport_loss()
    }

    /// End the Browser Session only after every owned context has proven destruction.
    pub fn end(&mut self) -> Result<(), BrowserSessionError> {
        self.session.end()
    }
}

fn reserve_epoch(next_epoch: &mut u64) -> Result<BrowserContextEpoch, BrowserSessionError> {
    let epoch = BrowserContextEpoch(*next_epoch);
    *next_epoch = next_epoch
        .checked_add(1)
        .ok_or(BrowserSessionError::EpochExhausted)?;
    Ok(epoch)
}

fn allocate_incarnation(
    counter: &AtomicU64,
) -> Result<BrowserSessionIncarnation, BrowserSessionError> {
    let value = counter
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
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
        DisposableIsolationId::parse(value).expect("valid isolation id")
    }

    fn session(value: u64) -> BrowserSession {
        BrowserSession::start(session_id(value)).expect("incarnation capacity")
    }

    #[test]
    fn isolation_identity_validation_is_bounded() {
        assert_eq!(
            DisposableIsolationId::parse(""),
            Err(DisposableIsolationIdError::Empty)
        );
        assert_eq!(
            DisposableIsolationId::parse(&"x".repeat(4097)),
            Err(DisposableIsolationIdError::TooLong)
        );
        assert_eq!(
            DisposableIsolationId::parse(" user-context "),
            Err(DisposableIsolationIdError::InvalidCharacter)
        );
        assert_eq!(
            DisposableIsolationId::parse("user\ncontext"),
            Err(DisposableIsolationIdError::InvalidCharacter)
        );
        let valid = isolation_id("webdriver-user-context-10");
        assert_eq!(valid.as_str(), "webdriver-user-context-10");
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
                known
            )]
        );
        assert_eq!(
            known_session.end(),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    #[test]
    fn duplicate_adapter_output_preserves_offending_handle() {
        let duplicate_context_handle =
            DisposableContextHandle::new(isolation_id("isolation-30-b"), context_id(30));
        let context_port = TestPort::with_handles(vec![
            DisposableContextHandle::new(isolation_id("isolation-30-a"), context_id(30)),
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
            &[BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                duplicate_context_handle
            )]
        );

        let duplicate_isolation_handle =
            DisposableContextHandle::new(isolation_id("isolation-31"), context_id(311));
        let isolation_port = TestPort::with_handles(vec![
            DisposableContextHandle::new(isolation_id("isolation-31"), context_id(310)),
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
            &[BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                duplicate_isolation_handle
            )]
        );
    }

    #[test]
    fn create_completion_failure_preserves_non_authorizing_recovery_evidence() {
        let expected =
            DisposableContextHandle::new(isolation_id("isolation-315"), context_id(315));
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
                expected
            )]
        );
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
        let first =
            DisposableContextHandle::new(isolation_id("isolation-316-a"), context_id(316));
        let duplicate =
            DisposableContextHandle::new(isolation_id("isolation-316-b"), context_id(316));
        let mut port = TestPort::with_handles(vec![first, duplicate.clone()]);
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
                BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(duplicate),
            ]
        );
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
            &[BrowserSessionRecoveryEvidence::UnprovenDestruction(
                expected_handle
            )]
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
    fn incarnation_allocator_fails_closed_before_wrap() {
        let counter = AtomicU64::new(u64::MAX);
        let error = BrowserSession::start_with_counter(session_id(12), &counter)
            .expect_err("incarnation allocation must fail closed before wrapping");
        assert_eq!(error, BrowserSessionError::IncarnationExhausted);
    }
}
