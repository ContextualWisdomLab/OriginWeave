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
    /// Destruction of this exact owned handle failed or could not be proven.
    UnprovenDestruction(DisposableContextHandle),
}

/// Port implemented by a reviewed browser adapter for disposable context lifecycle operations.
///
/// `incarnation` is domain-issued and must participate in the adapter's lifecycle mapping; ignoring it
/// would reintroduce sequential ABA aliasing. `create_disposable_context` must create a fresh isolation
/// boundary and context owned exclusively by the supplied Browser Session incarnation. For WebDriver
/// BiDi the isolation identity maps one-to-one to the user-context identifier returned by
/// `browser.createUserContext`.
///
/// [`DisposableContextCreateError::CreateFailedClean`] is allowed only when the adapter proves that no
/// disposable state was created. If a user-context identity is already known when later creation or
/// verification becomes uncertain, the adapter must return it inside
/// [`DisposableContextCreateError::CreateFailedUncertain`].
///
/// `destroy_disposable_context` must destroy the exact boundary carried by the supplied handle and
/// return success only after destruction is proven. Reconstructing cleanup authority from raw driver
/// identifiers is forbidden, and a command acknowledgement alone is insufficient evidence.
pub trait DisposableContextPort {
    /// Create one fresh disposable isolation boundary and browsing context for this incarnation.
    fn create_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError>;

    /// Destroy the exact disposable isolation boundary represented by this handle and incarnation.
    fn destroy_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        context: &DisposableContextHandle,
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
/// Browser Session has created a disposable boundary through its lifecycle port. Session incarnation,
/// isolation identity, context identity, and epoch must all still match before adapter I/O is allowed.
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

impl BrowserSession {
    /// Start an active Browser Session around an already validated transport session identity.
    ///
    /// A fresh process-local incarnation is allocated before any browser I/O. Exhaustion fails closed
    /// rather than wrapping and making an older authority structurally valid again.
    pub fn start(id: BrowserSessionId) -> Result<Self, BrowserSessionError> {
        let incarnation = allocate_incarnation(&NEXT_BROWSER_SESSION_INCARNATION)?;
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

    /// Create and register one disposable context, then mint authority for its first epoch.
    pub fn create_disposable_context<P: DisposableContextPort>(
        &mut self,
        port: &mut P,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let epoch = self.reserve_epoch()?;
        let handle = match port.create_disposable_context(self.id, self.incarnation) {
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

        if self
            .contexts
            .values()
            .any(|record| record.handle.isolation == handle.isolation)
        {
            self.recovery_evidence
                .push(BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                    handle,
                ));
            self.enter_recovery_required();
            return Err(BrowserSessionError::DuplicateDisposableIsolation);
        }
        if self.contexts.contains_key(&handle.browsing_context) {
            self.recovery_evidence
                .push(BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                    handle,
                ));
            self.enter_recovery_required();
            return Err(BrowserSessionError::DuplicateBrowsingContext);
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
        if !self
            .contexts
            .get(&browsing_context)
            .is_some_and(|record| record.state == OwnedContextState::Active)
        {
            return Err(BrowserSessionError::ContextNotOwned);
        }
        let next = self.reserve_epoch()?;
        let record = self
            .contexts
            .get_mut(&browsing_context)
            .ok_or(BrowserSessionError::ContextNotOwned)?;
        record.epoch = next;
        Ok(Self::authority_for(
            self.id,
            self.incarnation,
            &record.handle,
            next,
        ))
    }

    /// Destroy the disposable isolation boundary covered by the supplied exact-epoch authority.
    pub fn destroy_disposable_context<P: DisposableContextPort>(
        &mut self,
        authority: &PresentationMutationAuthority,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        let browser_session = self.id;
        let incarnation = self.incarnation;
        let record = self.context_for_authority_mut(authority)?;
        let handle = record.handle.clone();
        match port.destroy_disposable_context(browser_session, incarnation, &handle) {
            Ok(()) => {
                record.state = OwnedContextState::Destroyed;
                Ok(())
            }
            Err(DisposableContextDestroyError::DestroyFailed) => {
                record.state = OwnedContextState::Uncertain;
                self.recovery_evidence
                    .push(BrowserSessionRecoveryEvidence::UnprovenDestruction(handle));
                self.enter_recovery_required();
                Err(BrowserSessionError::ContextDestructionFailed)
            }
        }
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

    #[derive(Debug)]
    struct TestPort {
        next_handle: DisposableContextHandle,
        create_error: Option<DisposableContextCreateError>,
        fail_destroy: bool,
        create_calls: usize,
        destroy_calls: usize,
        create_incarnations: Vec<BrowserSessionIncarnation>,
        destroy_incarnations: Vec<BrowserSessionIncarnation>,
        destroyed_isolations: Vec<DisposableIsolationId>,
    }

    impl TestPort {
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
                create_incarnations: Vec::new(),
                destroy_incarnations: Vec::new(),
                destroyed_isolations: Vec::new(),
            }
        }
    }

    impl DisposableContextPort for TestPort {
        fn create_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
            incarnation: BrowserSessionIncarnation,
        ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
            self.create_calls += 1;
            self.create_incarnations.push(incarnation);
            match self.create_error.clone() {
                Some(error) => Err(error),
                None => Ok(self.next_handle.clone()),
            }
        }

        fn destroy_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
            incarnation: BrowserSessionIncarnation,
            context: &DisposableContextHandle,
        ) -> Result<(), DisposableContextDestroyError> {
            self.destroy_calls += 1;
            self.destroy_incarnations.push(incarnation);
            self.destroyed_isolations.push(context.isolation.clone());
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
    fn disposable_creation_is_the_only_raw_context_entry_to_authority() {
        let mut session = session(1);
        let mut port = TestPort::new(10, "isolation-10");
        assert_eq!(session.id(), session_id(1));
        assert_ne!(session.incarnation().value(), 0);
        assert!(!session.transport_is_lost());
        assert!(session.recovery_evidence().is_empty());
        assert_eq!(
            session.presentation_authority(context_id(10)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned disposable context");
        assert_eq!(port.create_incarnations, vec![session.incarnation()]);
        assert_eq!(authority.browser_session(), session_id(1));
        assert_eq!(authority.incarnation(), session.incarnation());
        assert_eq!(authority.isolation().as_str(), "isolation-10");
        assert_eq!(authority.browsing_context(), context_id(10));
        assert_eq!(authority.context_epoch().value(), 1);
        assert_eq!(
            session.presentation_authority(context_id(10)),
            Ok(authority)
        );
    }

    #[test]
    fn creation_failure_preserves_known_recovery_identity() {
        let mut clean_session = session(2);
        let mut clean_port = TestPort::new(20, "isolation-20");
        clean_port.create_error = Some(DisposableContextCreateError::CreateFailedClean);
        assert_eq!(
            clean_session.create_disposable_context(&mut clean_port),
            Err(BrowserSessionError::ContextCreationFailed)
        );
        assert_eq!(clean_session.state(), BrowserSessionState::Active);
        clean_session.end().expect("clean failure can end");

        let mut unknown_session = session(21);
        let mut unknown_port = TestPort::new(210, "isolation-210");
        unknown_port.create_error = Some(DisposableContextCreateError::CreateFailedUncertain(None));
        assert_eq!(
            unknown_session.create_disposable_context(&mut unknown_port),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert!(unknown_session.recovery_evidence().is_empty());

        let known = isolation_id("partial-user-context-211");
        let mut known_session = session(22);
        let mut known_port = TestPort::new(211, "unused");
        known_port.create_error = Some(DisposableContextCreateError::CreateFailedUncertain(Some(
            known.clone(),
        )));
        assert_eq!(
            known_session.create_disposable_context(&mut known_port),
            Err(BrowserSessionError::ContextCreationUncertain)
        );
        assert_eq!(
            known_session.recovery_evidence(),
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
        let mut duplicate_context_session = session(3);
        let mut first_context_port = TestPort::new(30, "isolation-30-a");
        duplicate_context_session
            .create_disposable_context(&mut first_context_port)
            .expect("first owned context");
        let duplicate_context_handle =
            DisposableContextHandle::new(isolation_id("isolation-30-b"), context_id(30));
        let mut duplicate_context_port = TestPort::new(30, "isolation-30-b");
        assert_eq!(
            duplicate_context_session.create_disposable_context(&mut duplicate_context_port),
            Err(BrowserSessionError::DuplicateBrowsingContext)
        );
        assert_eq!(
            duplicate_context_session.recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                duplicate_context_handle
            )]
        );

        let mut duplicate_isolation_session = session(31);
        let mut first_isolation_port = TestPort::new(310, "isolation-31");
        duplicate_isolation_session
            .create_disposable_context(&mut first_isolation_port)
            .expect("first owned isolation");
        let duplicate_isolation_handle =
            DisposableContextHandle::new(isolation_id("isolation-31"), context_id(311));
        let mut duplicate_isolation_port = TestPort::new(311, "isolation-31");
        assert_eq!(
            duplicate_isolation_session.create_disposable_context(&mut duplicate_isolation_port),
            Err(BrowserSessionError::DuplicateDisposableIsolation)
        );
        assert_eq!(
            duplicate_isolation_session.recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(
                duplicate_isolation_handle
            )]
        );
    }

    #[test]
    fn epoch_exhaustion_prevents_creation_io() {
        let mut exhausted_session = session(4);
        exhausted_session.next_epoch = u64::MAX;
        let mut unused_port = TestPort::new(40, "isolation-40");
        assert_eq!(
            exhausted_session.create_disposable_context(&mut unused_port),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(unused_port.create_calls, 0);
    }

    #[test]
    fn epoch_advance_invalidates_old_and_unknown_authority() {
        let mut session = session(5);
        let mut port = TestPort::new(50, "isolation-50");
        let old = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert_eq!(
            session.advance_context_epoch(context_id(51)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        let new = session
            .advance_context_epoch(context_id(50))
            .expect("advanced epoch");
        assert_eq!(new.context_epoch().value(), 2);
        assert_eq!(
            session.destroy_disposable_context(&old, &mut port),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        session
            .destroy_disposable_context(&new, &mut port)
            .expect("destroy current epoch");
        assert_eq!(port.destroy_incarnations, vec![session.incarnation()]);
        assert_eq!(
            session.presentation_authority(context_id(50)),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(
            session.destroy_disposable_context(&new, &mut port),
            Err(BrowserSessionError::ContextNotOwned)
        );
    }

    #[test]
    fn cross_session_and_foreign_isolation_authority_fail_before_io() {
        let mut owner = session(6);
        let mut owner_port = TestPort::new(60, "isolation-60");
        let authority = owner
            .create_disposable_context(&mut owner_port)
            .expect("owner context");

        let mut foreign = session(7);
        let mut foreign_port = TestPort::new(60, "isolation-60");
        foreign
            .create_disposable_context(&mut foreign_port)
            .expect("foreign context");
        assert_eq!(
            foreign.destroy_disposable_context(&authority, &mut foreign_port),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(foreign_port.destroy_calls, 0);

        let forged = PresentationMutationAuthority {
            browser_session: owner.id(),
            incarnation: owner.incarnation(),
            isolation: isolation_id("foreign-isolation"),
            browsing_context: authority.browsing_context(),
            context_epoch: authority.context_epoch(),
        };
        assert_eq!(
            owner.destroy_disposable_context(&forged, &mut owner_port),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(owner_port.destroy_calls, 0);
    }

    #[test]
    fn sequential_incarnation_reuse_rejects_stale_authority() {
        let shared_id = session_id(8);
        let mut session_a = BrowserSession::start(shared_id).expect("A incarnation");
        let mut port_a = TestPort::new(80, "reused-user-context");
        let authority_a = session_a
            .create_disposable_context(&mut port_a)
            .expect("A context");
        session_a
            .destroy_disposable_context(&authority_a, &mut port_a)
            .expect("A destroy");
        session_a.end().expect("A end");

        let mut session_b = BrowserSession::start(shared_id).expect("B incarnation");
        let mut port_b = TestPort::new(80, "reused-user-context");
        let authority_b = session_b
            .create_disposable_context(&mut port_b)
            .expect("B context");
        assert_ne!(session_a.incarnation(), session_b.incarnation());
        assert_eq!(
            session_b.destroy_disposable_context(&authority_a, &mut port_b),
            Err(BrowserSessionError::AuthorityMismatch)
        );
        assert_eq!(port_b.destroy_calls, 0);
        session_b
            .destroy_disposable_context(&authority_b, &mut port_b)
            .expect("B destroy");
        assert_eq!(port_b.destroy_calls, 1);
    }

    #[test]
    fn destroy_failure_retains_handle_and_transport_loss_orthogonally() {
        let mut session = session(9);
        let mut port = TestPort::new(90, "isolation-90");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        let expected_handle =
            DisposableContextHandle::new(isolation_id("isolation-90"), context_id(90));
        port.fail_destroy = true;
        assert_eq!(
            session.destroy_disposable_context(&authority, &mut port),
            Err(BrowserSessionError::ContextDestructionFailed)
        );
        assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);
        assert_eq!(
            session.recovery_evidence(),
            &[BrowserSessionRecoveryEvidence::UnprovenDestruction(
                expected_handle
            )]
        );
        assert!(!session.transport_is_lost());
        assert!(session.record_transport_loss());
        assert!(session.transport_is_lost());
        assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);
        assert!(!session.record_transport_loss());
        assert_eq!(
            session.create_disposable_context(&mut port),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            session.presentation_authority(context_id(90)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(
            session.advance_context_epoch(context_id(90)),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn transport_loss_invalidates_active_contexts_and_is_idempotent() {
        let mut session = session(10);
        let mut port = TestPort::new(100, "isolation-100");
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert!(session.record_transport_loss());
        assert_eq!(session.state(), BrowserSessionState::TransportLost);
        assert!(session.transport_is_lost());
        assert!(!session.record_transport_loss());
        assert_eq!(
            session.destroy_disposable_context(&authority, &mut port),
            Err(BrowserSessionError::SessionNotActive)
        );
        assert_eq!(port.destroy_calls, 0);
    }

    #[test]
    fn normal_end_requires_proven_destruction_and_ignores_late_transport_report() {
        let mut session = session(11);
        let mut port = TestPort::new(110, "isolation-110");
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
        session.end().expect("normal end");
        assert_eq!(session.state(), BrowserSessionState::Ended);
        assert!(!session.record_transport_loss());
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn incarnation_allocator_fails_closed_before_wrap() {
        let counter = AtomicU64::new(u64::MAX);
        assert_eq!(
            allocate_incarnation(&counter),
            Err(BrowserSessionError::IncarnationExhausted)
        );
    }
}
