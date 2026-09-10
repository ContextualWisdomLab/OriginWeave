//! Browser Session lifecycle authority for OriginWeave.
//!
//! This crate owns the domain transition that turns a newly created disposable
//! browser context into presentation-mutation authority. Driver identifiers remain
//! adapter data: naming a context is never sufficient to mint authority.

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
}

/// Domain failure while changing Browser Session ownership state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSessionError {
    /// The requested transition requires an active Browser Session.
    SessionNotActive,
    /// No unused context epoch remains, so no new authority can be issued safely.
    EpochExhausted,
    /// The disposable-context port could not create the requested isolated context.
    ContextCreationFailed,
    /// The port returned a browsing-context identity already known to this session.
    DuplicateBrowsingContext,
    /// The requested context is not currently owned and active in this session.
    ContextNotOwned,
    /// The supplied authority belongs to another session, context, or context epoch.
    AuthorityMismatch,
    /// The disposable-context port could not prove destruction of the owned context.
    ContextDestructionFailed,
    /// Normal session end was requested while an owned or uncertain context remains.
    ActiveContextRemains,
}

/// Bounded failure reported by the adapter port used for disposable context lifecycle I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisposableContextPortError {
    /// Creation of a fresh disposable context failed.
    CreateFailed,
    /// Destruction of an owned disposable context failed or could not be proven.
    DestroyFailed,
}

/// Port implemented by a reviewed browser adapter for disposable context lifecycle operations.
///
/// `create_disposable_context` must create a fresh context owned exclusively by the supplied
/// Browser Session. An implementation that merely returns an existing/shared context violates this
/// port contract. `destroy_disposable_context` must return success only after the adapter has proved
/// that the task-owned disposable boundary is gone; a command acknowledgement alone is insufficient.
pub trait DisposableContextPort {
    /// Create one fresh disposable context for the Browser Session.
    fn create_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
    ) -> Result<BrowsingContextId, DisposableContextPortError>;

    /// Destroy one context previously created through this port for the same Browser Session.
    fn destroy_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
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
/// the Browser Session aggregate has successfully created a disposable context through its lifecycle
/// port, or after that already-owned context advances to a new epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationMutationAuthority {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    context_epoch: BrowserContextEpoch,
}

impl PresentationMutationAuthority {
    /// Return the Browser Session that owns this authority.
    #[must_use]
    pub const fn browser_session(self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the owned browsing-context identity.
    #[must_use]
    pub const fn browsing_context(self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Return the exact context epoch covered by this authority.
    #[must_use]
    pub const fn context_epoch(self) -> BrowserContextEpoch {
        self.context_epoch
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnedContextState {
    Active,
    Destroyed,
    Uncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OwnedContextRecord {
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
    /// Start an active Browser Session around an already validated session identity.
    #[must_use]
    pub fn start(id: BrowserSessionId) -> Self {
        Self {
            id,
            state: BrowserSessionState::Active,
            next_epoch: 1,
            contexts: BTreeMap::new(),
        }
    }

    /// Return this aggregate's stable browser-session identity.
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
    /// untrackable context. Epoch identifiers may therefore have gaps after failed creation or rejected
    /// duplicate adapter output. A duplicate identity is rejected without attempting cleanup because a
    /// port that violates the fresh-context contract may have returned another owner's existing context.
    pub fn create_disposable_context<P: DisposableContextPort>(
        &mut self,
        port: &mut P,
    ) -> Result<PresentationMutationAuthority, BrowserSessionError> {
        self.require_active()?;
        let epoch = self.reserve_epoch()?;
        let browsing_context = port
            .create_disposable_context(self.id)
            .map_err(|_error| BrowserSessionError::ContextCreationFailed)?;
        if self.contexts.contains_key(&browsing_context) {
            return Err(BrowserSessionError::DuplicateBrowsingContext);
        }
        self.contexts.insert(
            browsing_context,
            OwnedContextRecord {
                epoch,
                state: OwnedContextState::Active,
            },
        );
        Ok(self.authority_for(browsing_context, epoch))
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
        Ok(self.authority_for(browsing_context, record.epoch))
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
        Ok(self.authority_for(browsing_context, next))
    }

    /// Destroy the disposable context covered by the supplied exact-epoch authority.
    ///
    /// Failed or unproven destruction moves the context to an uncertain terminal state so its old
    /// authority cannot be reused. OriginWeave does not interpret an adapter ACK as destruction proof.
    pub fn destroy_disposable_context<P: DisposableContextPort>(
        &mut self,
        authority: PresentationMutationAuthority,
        port: &mut P,
    ) -> Result<(), BrowserSessionError> {
        let record = self.take_context_for_authority(authority)?;
        let result = port.destroy_disposable_context(self.id, authority.browsing_context);
        let state = if result.is_ok() {
            OwnedContextState::Destroyed
        } else {
            OwnedContextState::Uncertain
        };
        self.contexts.insert(
            authority.browsing_context,
            OwnedContextRecord {
                epoch: record.epoch,
                state,
            },
        );
        if result.is_ok() {
            Ok(())
        } else {
            Err(BrowserSessionError::ContextDestructionFailed)
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
        for record in self.contexts.values_mut() {
            if record.state == OwnedContextState::Active {
                record.state = OwnedContextState::Uncertain;
            }
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
        &self,
        browsing_context: BrowsingContextId,
        context_epoch: BrowserContextEpoch,
    ) -> PresentationMutationAuthority {
        PresentationMutationAuthority {
            browser_session: self.id,
            browsing_context,
            context_epoch,
        }
    }

    fn take_context_for_authority(
        &mut self,
        authority: PresentationMutationAuthority,
    ) -> Result<OwnedContextRecord, BrowserSessionError> {
        self.require_active()?;
        if authority.browser_session != self.id {
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        let Some(record) = self.contexts.remove(&authority.browsing_context) else {
            return Err(BrowserSessionError::ContextNotOwned);
        };
        if record.state != OwnedContextState::Active {
            self.contexts.insert(authority.browsing_context, record);
            return Err(BrowserSessionError::ContextNotOwned);
        }
        if record.epoch != authority.context_epoch {
            self.contexts.insert(authority.browsing_context, record);
            return Err(BrowserSessionError::AuthorityMismatch);
        }
        Ok(record)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPort {
        next_context: BrowsingContextId,
        fail_create: bool,
        fail_destroy: bool,
        create_calls: usize,
        destroy_calls: usize,
    }

    impl TestPort {
        fn new(next_context: u64) -> Self {
            Self {
                next_context: BrowsingContextId::new(next_context).expect("valid context id"),
                fail_create: false,
                fail_destroy: false,
                create_calls: 0,
                destroy_calls: 0,
            }
        }
    }

    impl DisposableContextPort for TestPort {
        fn create_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
        ) -> Result<BrowsingContextId, DisposableContextPortError> {
            self.create_calls += 1;
            if self.fail_create {
                Err(DisposableContextPortError::CreateFailed)
            } else {
                Ok(self.next_context)
            }
        }

        fn destroy_disposable_context(
            &mut self,
            _browser_session: BrowserSessionId,
            _browsing_context: BrowsingContextId,
        ) -> Result<(), DisposableContextPortError> {
            self.destroy_calls += 1;
            if self.fail_destroy {
                Err(DisposableContextPortError::DestroyFailed)
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

    #[test]
    fn disposable_creation_is_the_only_raw_context_entry_to_authority() {
        let mut session = BrowserSession::start(session_id(1));
        let mut port = TestPort::new(10);

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
        assert_eq!(authority.browsing_context(), context_id(10));
        assert_eq!(authority.context_epoch().value(), 1);
        assert_eq!(
            session.presentation_authority(context_id(10)),
            Ok(authority)
        );
    }

    #[test]
    fn creation_failure_duplicate_and_epoch_exhaustion_fail_closed() {
        let mut failed_session = BrowserSession::start(session_id(2));
        let mut failed_port = TestPort::new(20);
        failed_port.fail_create = true;
        assert_eq!(
            failed_session.create_disposable_context(&mut failed_port),
            Err(BrowserSessionError::ContextCreationFailed)
        );

        let mut duplicate_session = BrowserSession::start(session_id(3));
        let mut duplicate_port = TestPort::new(30);
        duplicate_session
            .create_disposable_context(&mut duplicate_port)
            .expect("first owned context");
        assert_eq!(
            duplicate_session.create_disposable_context(&mut duplicate_port),
            Err(BrowserSessionError::DuplicateBrowsingContext)
        );

        let mut exhausted_session = BrowserSession::start(session_id(4));
        exhausted_session.next_epoch = u64::MAX;
        let mut unused_port = TestPort::new(40);
        assert_eq!(
            exhausted_session.create_disposable_context(&mut unused_port),
            Err(BrowserSessionError::EpochExhausted)
        );
        assert_eq!(unused_port.create_calls, 0);
    }

    #[test]
    fn epoch_advance_invalidates_old_and_cross_session_authority() {
        let mut session = BrowserSession::start(session_id(5));
        let mut port = TestPort::new(50);
        let old = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        let new = session
            .advance_context_epoch(context_id(50))
            .expect("advanced epoch");
        assert_eq!(new.context_epoch().value(), 2);
        assert_eq!(
            session.destroy_disposable_context(old, &mut port),
            Err(BrowserSessionError::AuthorityMismatch)
        );

        let mut foreign = BrowserSession::start(session_id(6));
        let mut foreign_port = TestPort::new(60);
        foreign
            .create_disposable_context(&mut foreign_port)
            .expect("foreign context");
        assert_eq!(
            foreign.destroy_disposable_context(new, &mut foreign_port),
            Err(BrowserSessionError::AuthorityMismatch)
        );

        session
            .destroy_disposable_context(new, &mut port)
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
            session.destroy_disposable_context(new, &mut port),
            Err(BrowserSessionError::ContextNotOwned)
        );
        assert_eq!(port.destroy_calls, 1);
    }

    #[test]
    fn destroy_failure_quarantines_authority_and_transport_loss_is_idempotent() {
        let mut session = BrowserSession::start(session_id(7));
        let mut port = TestPort::new(70);
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        port.fail_destroy = true;
        assert_eq!(
            session.destroy_disposable_context(authority, &mut port),
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
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn successful_destruction_is_required_before_normal_end() {
        let mut session = BrowserSession::start(session_id(8));
        let mut port = TestPort::new(80);
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert_eq!(
            session.end(),
            Err(BrowserSessionError::ActiveContextRemains)
        );
        session
            .destroy_disposable_context(authority, &mut port)
            .expect("proven destruction");
        session.end().expect("all owned contexts destroyed");
        assert_eq!(session.state(), BrowserSessionState::Ended);
        assert_eq!(session.end(), Err(BrowserSessionError::SessionNotActive));
    }

    #[test]
    fn transport_loss_invalidates_still_active_contexts() {
        let mut session = BrowserSession::start(session_id(9));
        let mut port = TestPort::new(90);
        let authority = session
            .create_disposable_context(&mut port)
            .expect("owned context");
        assert!(session.record_transport_loss());
        assert_eq!(
            session.destroy_disposable_context(authority, &mut port),
            Err(BrowserSessionError::SessionNotActive)
        );
    }

    #[test]
    fn advance_context_epoch_rejects_unknown_and_exhausted_contexts() {
        let mut session = BrowserSession::start(session_id(10));
        assert_eq!(
            session.advance_context_epoch(context_id(100)),
            Err(BrowserSessionError::ContextNotOwned)
        );

        let mut port = TestPort::new(101);
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
