//! Browser Session lifecycle authority for OriginWeave.
//!
//! This crate owns the domain transition that turns a newly created disposable
//! browser isolation boundary into presentation-mutation authority. Driver identifiers
//! remain adapter data: naming a session or browsing context is never sufficient to mint authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeMap;

use originweave_core::{BrowserSessionId, BrowsingContextId};

/// Current lifecycle state of one Browser Session aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSessionState {
    /// The session may create and own disposable contexts.
    Active,
    /// Every owned context was destroyed and the session was ended normally.
    Ended,
    /// The browser transport was lost; remaining contexts have uncertain cleanup state.
    TransportLost,
    /// Browser lifecycle ownership became uncertain and requires external reconciliation.
    RecoveryRequired,
}

/// Domain failure while changing Browser Session ownership state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSessionError {
    /// The requested transition requires an active Browser Session.
    SessionNotActive,
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
    /// The supplied authority belongs to another isolation boundary, session, context, or epoch.
    AuthorityMismatch,
    /// The disposable-context port could not prove destruction of the owned isolation boundary.
    ContextDestructionFailed,
    /// Normal session end was requested while an owned or uncertain context remains.
    ActiveContextRemains,
}

/// Bounded failure reported by the adapter port used for disposable context lifecycle I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableContextPortError {
    /// Creation failed and the adapter proved that no disposable boundary was created.
    CreateFailedClean,
    /// Creation failed after ownership may have changed, so browser cleanup state is uncertain.
    CreateFailedUncertain,
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

/// Port implemented by a reviewed browser adapter for disposable context lifecycle operations.
///
/// `create_disposable_context` must create a fresh isolation boundary and context owned exclusively
/// by the supplied Browser Session. The returned [`DisposableIsolationId`] must be non-aliasing for
/// the lifetime of that boundary; for WebDriver BiDi this means a one-to-one mapping to the unique
/// user-context identifier returned by `browser.createUserContext`. An implementation that merely
/// returns an existing/shared context violates this port contract.
///
/// Creation failures are typed. `CreateFailedClean` is allowed only when the adapter can prove that
/// no disposable browser state was created. Any partial-create or uncertain post-condition must be
/// `CreateFailedUncertain`, which makes normal Browser Session completion ineligible until recovery.
///
/// `destroy_disposable_context` must destroy the exact isolation boundary carried by the supplied
/// handle and return success only after the adapter has proved that the task-owned boundary is gone.
/// Reconstructing cleanup authority from `(BrowserSessionId, BrowsingContextId)` is forbidden, and a
/// command acknowledgement alone is insufficient destruction evidence.
pub trait DisposableContextPort {
    /// Create one fresh disposable isolation boundary and browsing context for the Browser Session.
    fn create_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
    ) -> Result<DisposableContextHandle, DisposableContextPortError>;

    /// Destroy the exact disposable isolation boundary represented by this handle.
    fn destroy_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        context: &DisposableContextHandle,
    ) -> Result<(), DisposableContextPortError>;
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
/// The fields are private and no public constructor exists. A caller can obtain this value only after
/// the Browser Session aggregate has successfully created a disposable isolation boundary through its
/// lifecycle port, or after that already-owned context advances to a new epoch. The isolation identity
/// prevents two aggregate incarnations that reuse external session/context identifiers from aliasing
/// each other's mutation or destruction authority when their disposable boundaries are distinct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationMutationAuthority {
    browser_session: BrowserSessionId,
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
///
/// The aggregate never accepts a remote/WebDriver context string as authority. A context enters the
/// owned set only through [`BrowserSession::create_disposable_context`], which invokes the lifecycle
/// port before minting an opaque [`PresentationMutationAuthority`].
#[derive(Debug)]
pub struct BrowserSession {
    id: BrowserSessionId,
    state: BrowserSessionState,
    next_epoch: u64,
    contexts: BTreeMap<BrowsingContextId, OwnedContextRecord>,
}

impl BrowserSession {
    /// Start an active Browser Session around an already validated transport session identity.
    ///
    /// The transport identity may be reused by a later aggregate incarnation; it therefore does not
    /// participate alone in disposable ownership. Per-context authority additionally carries the
    /// adapter-proved non-aliasing isolation identity.
    #[must_use]
    pub fn start(id: BrowserSessionId) -> Self {
        Self {
            id,
            state: BrowserSessionState::Active,
            next_epoch: 1,
            contexts: BTreeMap::new(),
        }
    }

    /// Return this aggregate's browser-session transport identity.
    #[must_use]
    pub const fn id(&self) -> BrowserSessionId {
        self.id
    }

    /// Return the current aggregate lifecycle state.
    #[must_use]
    pub const fn state(&self) -> BrowserSessionState {
        self.state
    }

    /// Create and register one disposable context, then mint authority for its first epoch.
    ///
    /// Epoch capacity is reserved before external creation so an exhausted aggregate never creates an
    /// untrackable context. A clean creation failure leaves the aggregate active. An uncertain creation
    /// failure or duplicate adapter result enters `RecoveryRequired`, because the browser may contain an
    /// untracked isolation boundary and normal completion must not hide that lifecycle uncertainty.
    pub fn create_disposable_context<P: DisposableContextPort>(
        &mut self,
        port: &mut P,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let epoch = self.reserve_epoch()?;
        let handle = match port.create_disposable_context(self.id) {
            Ok(handle) => handle,
            Err(DisposableContextPortError::CreateFailedClean) => {
                return Err(BrowserSessionError::ContextCreationFailed);
            }
            Err(DisposableContextPortError::CreateFailedUncertain) => {
                self.enter_recovery_required();
                return Err(BrowserSessionError::ContextCreationUncertain);
            }
            Err(DisposableContextPortError::DestroyFailed) => {
                self.enter_recovery_required();
                return Err(BrowserSessionError::ContextCreationUncertain);
            }
        };

        if self
            .contexts
            .values()
            .any(|record| record.handle.isolation == handle.isolation)
        {
            self.enter_recovery_required();
            return Err(BrowserSessionError::DuplicateDisposableIsolation);
        }
        if self.contexts.contains_key(&handle.browsing_context) {
            self.enter_recovery_required();
            return Err(BrowserSessionError::DuplicateBrowsingContext);
        }

        let browsing_context = handle.browsing_context;
        let authority = Self::authority_for(self.id, &handle, epoch);
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

    /// Return current presentation authority for an already-owned active context.
    ///
    /// A raw context identity that was not created through this aggregate cannot enter the authority
    /// path and fails closed with [`BrowserSessionError::ContextNotOwned`].
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
        Ok(Self::authority_for(self.id, &record.handle, record.epoch))
    }

    /// Advance one active owned context to a new authority epoch.
    ///
    /// Navigation, renderer replacement, or another lifecycle boundary can call this transition to
    /// invalidate every previously issued token while preserving disposable-context ownership. Epoch
    /// identifiers are monotonic authority identities rather than gap-free business counters.
    pub fn advance_context_epoch(
        &mut self,
        browsing_context: BrowsingContextId,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let next = self.reserve_epoch()?;
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .filter(|record| record.state == OwnedContextState::Active)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        record.epoch = next;
        Ok(Self::authority_for(self.id, &record.handle, next))
    }

    /// Destroy the disposable isolation boundary covered by the supplied exact-epoch authority.
    ///
    /// Authority is validated before any adapter I/O. The same validated mutable record is retained
    /// across the port call, so no structurally unreachable second lookup is required. Failed or
    /// unproven destruction moves the context to an uncertain terminal state.
    pub fn destroy_disposable_context<P: DisposableContextPort>(
        &mut self,
        authority: &PresentationMutationAuthority,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        let browser_session = self.id;
        let record = self.context_for_authority_mut(authority)?;
        let handle = record.handle.clone();
        match port.destroy_disposable_context(browser_session, &handle) {
            Ok(()) => {
                record.state = OwnedContextState::Destroyed;
                Ok(())
            }
            Err(_error) => {
                record.state = OwnedContextState::Uncertain;
                Err(BrowserSessionError::ContextDestructionFailed)
            }
        }
    }

    /// Record browser transport loss and invalidate all still-active context authority.
    ///
    /// Returns `true` only for the first transition to `TransportLost`; repeated reports are idempotent.
    pub fn record_transport_loss(&mut self) -> bool {
        if self.state != BrowserSessionState::Active {
            return false;
        }
        self.state = BrowserSessionState::TransportLost;
        self.mark_active_contexts_uncertain();
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

    fn require_active(&self) -> Result<(), BrowserSessionError> {
        if self.state == BrowserSessionState::Active {
            Ok(())
        } else {
            Err(BrowserSessionError::SessionNotActive)
        }
    }

    fn reserve_epoch(&mut self) -> Result<BrowserContextEpoch, BrowserSessionError> {
        let epoch = BrowserContextEpoch(self.next_epoch);
        self.next_epoch = self
            .next_epoch
            .checked_add(1)
            .ok_or(BrowserSessionError::EpochExhausted)?;
        Ok(epoch)
    }

    fn authority_for(
        browser_session: BrowserSessionId,
        handle: &DisposableContextHandle,
        context_epoch: BrowserContextEpoch,
    ) -> PresentationMutationAuthority {
        PresentationMutationAuthority {
            browser_session,
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
        if authority.browser_session != self.id {
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPort {
        next_handle: DisposableContextHandle,
        create_error: Option<DisposableContextPortError>,
        fail_destroy: bool,
        create_calls: usize,
        destroy_calls: usize,
        destroyed_isolations: Vec<DisposableIsolationId>,
    }

    impl TestPort {
        /// Build a deterministic lifecycle port for one context/isolation pair.
        fn new(context: u64, isolation: &str) -> Self {
            Self {
                next_handle: DisposableContextHandle::new(
                    isolation_id(isolation),
                    context_id(context),
                ),
                create_error: None,
                fail_destroy: false,
                create_calls: 0,
                destroy_calls: 0,
                destroyed_isolations: Vec::new(),
            }
        }
    }

    impl DisposableContextPort for TestPort {
        /// Return the configured handle or bounded creation failure.
        fn create_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
        ) -> Result<DisposableContextHandle, DisposableContextPortError> {
            self.create_calls += 1;
            match self.create_error {
                Some(error) => Err(error),
                None => Ok(self.next_handle.clone()),
            }
        }

        /// Record exact isolation destruction before returning the configured result.
        fn destroy_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
            context: &DisposableContextHandle,
        ) -> Result<(), DisposableContextPortError> {
            self.destroy_calls += 1;
            self.destroyed_isolations.push(context.isolation.clone());
            if self.fail_destroy {
                Err(DisposableContextPortError::DestroyFailed)
            } else {
                Ok(())
            }
        }
    }

    /// Construct a validated Browser Session transport identifier.
    fn session_id(value: u64) -> BrowserSessionId {
        BrowserSessionId::new(value).expect("valid session id")
    }

    /// Construct a validated browsing-context identifier.
    fn context_id(value: u64) -> BrowsingContextId {
        BrowsingContextId::new(value).expect("valid context id")
    }

    /// Construct a validated disposable isolation identifier.
    fn isolation_id(value: &str) -> DisposableIsolationId {
        DisposableIsolationId::parse(value).expect("valid isolation id")
    }

    /// Validate isolation identity bounds and accessor behavior.
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

    /// Prove that raw context addressability cannot mint presentation authority.
    #[test]
    fn disposable_creation_is_the_only_raw_context_entry_to_authority() {
        let mut session = BrowserSession::start(session_id(1));
        let mut port = TestPort::new(10, "isolation-10");

        assert_eq!(session.id(), session_id(1));
        assert_eq!(session.state(), BrowserSessionState::Active);
        assert_eq!(
            session.presentation_authority(context_id(10)),
            Err(BrowserSessionError::ContextNotOwned)
        );

        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned disposable context");
        assert_eq!(port.create_calls, 1);
        assert_eq!(authority.browser_session(), session_id(1));
        assert_eq!(authority.isolation().as_str(), "isolation-10");
        assert_eq!(authority.browsing_context(), context_id(10));
        assert_eq!(authority.context_epoch().value(), 1);
        assert_eq!(
            session.presentation_authority(context_id(10)),
            Ok(authority)
        );
    }

    /// Distinguish proved-clean creation failure from uncertain partial creation.
    #[test]
    fn creation_failure_is_typed_clean_or_recovery_required() {
        let mut clean_session = BrowserSession::start(session_id(2));
        let mut clean_port = TestPort::new(20, "isolation-20");
        clean_port.create_error = Some(DisposableContextPortError::CreateFailedClean);
        assert_eq!(
            clean_session.create_disposable_context(&mut clean_port),
            Err(BrowserSessionError::ContextCreationFailed)
        );
        assert_eq!(clean_session.state(), BrowserSessionState::Active);
        clean_session.end().expect("proved-clean failure can end normally");

        let mut uncertain_session = BrowserSession::start(session_id(21));
        let mut uncertain_port = TestPort::new(210, "isolation-210");
        uncertain_port.create_error = Some(DisposableContextPortError::CreateFailedUncertain);
        assert_eq!(
            uncertain_session.create_disposable_context(&mut uncertain_port),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            uncertain_session.state(),
            BrowserSessionState::RecoveryRequired
        );
        assert_eq!(uncertain_session.end(), Err(BrowserSessionError::SessionNotActive));

        let mut invalid_error_session = BrowserSession::start(session_id(22));
        let mut invalid_error_port = TestPort::new(220, "isolation-220");
        invalid_error_port.create_error = Some(DisposableContextPortError::DestroyFailed);
        assert_eq!(
            invalid_error_session.create_disposable_context(&mut invalid_error_port),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            invalid_error_session.state(),
            BrowserSessionState::RecoveryRequired
        );
    }

    /// Duplicate adapter output must prevent a false normal session completion.
    #[test]
    fn duplicate_adapter_output_requires_recovery() {
        let mut duplicate_context_session = BrowserSession::start(session_id(3));
        let mut first_context_port = TestPort::new(30, "isolation-30-a");
        duplicate_context_session
            .create_disposable_context(&mut first_context_port)
            .expect("first owned context");
        let mut duplicate_context_port = TestPort::new(30, "isolation-30-b");
        assert_eq!(
            duplicate_context_session.create_disposable_context(&mut duplicate_context_port),
            Err(BrowserSessionError::DuplicateBrowsingContext)
        );
        assert_eq!(
            duplicate_context_session.state(),
            BrowserSessionState::RecoveryRequired
        );
        assert_eq!(
            duplicate_context_session.end(),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            duplicate_context_session.create_disposable_context(&mut duplicate_context_port),
            Err(BrowserSessionError::SessionNotActive)
        );

        let mut duplicate_isolation_session = BrowserSession::start(session_id(31));
        let mut first_isolation_port = TestPort::new(310, "isolation-31");
        duplicate_isolation_session
            .create_disposable_context(&mut first_isolation_port)
            .expect("first owned isolation");
        let mut duplicate_isolation_port = TestPort::new(311, "isolation-31");
        assert_eq!(
            duplicate_isolation_session.create_disposable_context(&mut duplicate_isolation_port),
            Err(BrowserSessionError::DuplicateDisposableIsolation)
        );
        assert_eq!(
            duplicate_isolation_session.state(),
            BrowserSessionState::RecoveryRequired
        );
        assert_eq!(
            duplicate_isolation_session.presentation_authority(context_id(310)),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    /// Reserve authority capacity before browser I/O so exhaustion cannot leak a context.
    #[test]
    fn epoch_exhaustion_prevents_creation_io() {
        let mut exhausted_session = BrowserSession::start(session_id(4));
        exhausted_session.next_epoch = u64::MAX;
        let mut unused_port = TestPort::new(40, "isolation-40");
        assert_eq!(
            exhausted_session.create_disposable_context(&mut unused_port),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(unused_port.create_calls, 0);
    }

    /// Reject stale epoch and foreign-session authority before destruction I/O.
    #[test]
    fn epoch_advance_invalidates_old_and_cross_session_authority() {
        let mut session = BrowserSession::start(session_id(5));
        let mut port = TestPort::new(50, "isolation-50");
        let old = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        let new = session
            .advance_context_epoch(context_id(50))
            .expect("advanced epoch");
        assert_eq!(new.context_epoch().value(), 2);
        assert_eq!(
            session.destroy_disposable_context(&old, &mut port),
            Err(BrowserSessionError::AuthorityMismatch)
        );

        let mut foreign = BrowserSession::start(session_id(6));
        let mut foreign_port = TestPort::new(60, "isolation-60");
        foreign
            .create_disposable_context(&mut foreign_port)
            .expect("foreign context");
        assert_eq!(
            foreign.destroy_disposable_context(&new, &mut foreign_port),
            Err(BrowserSessionError::AuthorityMismatch)
        );

        session
            .destroy_disposable_context(&new, &mut port)
            .expect("destroy current epoch");
        assert_eq!(port.destroy_calls, 1);
        assert_eq!(
            session.presentation_authority(context_id(50)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            session.advance_context_epoch(context_id(50)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            session.destroy_disposable_context(&new, &mut port),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(port.destroy_calls, 1);
    }

    /// Prove two aggregate incarnations cannot cross isolation ownership boundaries.
    #[test]
    fn two_aggregate_alias_cannot_cross_mutation_or_destruction_boundary() {
        let shared_session = session_id(12);
        let shared_context = context_id(120);
        let mut session_a = BrowserSession::start(shared_session);
        let mut session_b = BrowserSession::start(shared_session);
        let mut port_a = TestPort::new(120, "user-context-a");
        let mut port_b = TestPort::new(120, "user-context-b");

        let authority_a = session_a
            .create_disposable_context(&mut port_a)
            .expect("owner A context");
        let authority_b = session_b
            .create_disposable_context(&mut port_b)
            .expect("owner B context");
        assert_eq!(authority_a.browsing_context(), shared_context);
        assert_eq!(authority_b.browsing_context(), shared_context);
        assert_ne!(authority_a.isolation(), authority_b.isolation());

        assert_eq!(
            session_b.destroy_disposable_context(&authority_a, &mut port_b),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(port_b.destroy_calls, 0);

        session_b
            .destroy_disposable_context(&authority_b, &mut port_b)
            .expect("B destroys only its isolation boundary");
        assert_eq!(port_b.destroy_calls, 1);
        assert_eq!(
            port_b.destroyed_isolations,
            vec![isolation_id("user-context-b")]
        );
        assert_ne!(&port_b.destroyed_isolations[0], authority_a.isolation());
    }

    /// Reject an unknown context before any adapter destruction call.
    #[test]
    fn unknown_internal_authority_cannot_trigger_destroy_io() {
        let mut session = BrowserSession::start(session_id(11));
        let mut port = TestPort::new(110, "isolation-110");
        let unknown = PresentationMutationAuthority {
            browser_session: session_id(11),
            isolation: isolation_id("isolation-111"),
            browsing_context: context_id(111),
            context_epoch: BrowserContextEpoch(1),
        };

        assert_eq!(
            session.destroy_disposable_context(&unknown, &mut port),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(port.destroy_calls, 0);
    }

    /// Reject same-context authority with a foreign isolation identity before I/O.
    #[test]
    fn foreign_isolation_authority_cannot_trigger_destroy_io() {
        let mut session = BrowserSession::start(session_id(13));
        let mut port = TestPort::new(130, "isolation-130");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        let forged = PresentationMutationAuthority {
            browser_session: authority.browser_session(),
            isolation: isolation_id("isolation-foreign"),
            browsing_context: authority.browsing_context(),
            context_epoch: authority.context_epoch(),
        };
        assert_eq!(
            session.destroy_disposable_context(&forged, &mut port),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(port.destroy_calls, 0);
    }

    /// Quarantine failed destruction and keep transport-loss transitions idempotent.
    #[test]
    fn destroy_failure_quarantines_authority_and_transport_loss_is_idempotent() {
        let mut session = BrowserSession::start(session_id(7));
        let mut port = TestPort::new(70, "isolation-70");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        port.fail_destroy = true;
        assert_eq!(
            session.destroy_disposable_context(&authority, &mut port),
            Err(BrowserSessionError::ContextDestructionFailed)
        );
        assert_eq!(port.destroy_calls, 1);
        assert_eq!(
            session.presentation_authority(context_id(70)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            session.end(),
            Err(BrowserSessionError::ActiveContextRemains)
        );
        assert!(session.record_transport_loss());
        assert!(!session.record_transport_loss());
        assert_eq!(session.state(), BrowserSessionState::TransportLost);
        assert_eq!(
            session.create_disposable_context(&mut port),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            session.presentation_authority(context_id(70)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            session.advance_context_epoch(context_id(70)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    /// Require proven context destruction before a normal session end.
    #[test]
    fn successful_destruction_is_required_before_normal_end() {
        let mut session = BrowserSession::start(session_id(8));
        let mut port = TestPort::new(80, "isolation-80");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert_eq!(
            session.end(),
            Err(BrowserSessionError::ActiveContextRemains)
        );
        session
            .destroy_disposable_context(&authority, &mut port)
            .expect("proven destruction");
        session.end().expect("all owned contexts destroyed");
        assert_eq!(session.state(), BrowserSessionState::Ended);
        assert_eq!(
            session.presentation_authority(context_id(80)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            session.advance_context_epoch(context_id(80)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    /// Invalidate still-active authority immediately after transport loss.
    #[test]
    fn transport_loss_invalidates_still_active_contexts() {
        let mut session = BrowserSession::start(session_id(9));
        let mut port = TestPort::new(90, "isolation-90");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert!(session.record_transport_loss());
        assert_eq!(
            session.destroy_disposable_context(&authority, &mut port),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(port.destroy_calls, 0);
    }

    /// Reject epoch advancement for unknown and exhausted contexts.
    #[test]
    fn advance_context_epoch_rejects_unknown_and_exhausted_contexts() {
        let mut session = BrowserSession::start(session_id(10));
        assert_eq!(
            session.advance_context_epoch(context_id(100)),
            Err(BrowserSessionError::ContextNotOwned)
        );

        let mut port = TestPort::new(101, "isolation-101");
        session
            .create_disposable_context(&mut port)
            .expect("owned context");
        session.next_epoch = u64::MAX;
        assert_eq!(
            session.advance_context_epoch(context_id(101)),
            Err(BrowserSessionError::EpochExhausted)
        );
    }
}
