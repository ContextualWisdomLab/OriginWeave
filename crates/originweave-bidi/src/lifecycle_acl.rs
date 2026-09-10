use std::collections::BTreeMap;
use std::marker::PhantomData;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, DisposableContextCreateError,
    DisposableContextDestroyError, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, PresentationMutationAuthority,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};
use originweave_fingerprint::{DevicePixelRatio, PresentationTimeZone, ViewportBounds};

use crate::presentation_capabilities::WebDriverBidiBrowsingContext;

/// Failure while projecting current Browser Session authority into a WebDriver BiDi target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBidiAclError {
    /// The Browser Session rejected the retained authority as non-current or non-active.
    BrowserSession(BrowserSessionError),
    /// The lifecycle adapter has no exact remote-context binding for the current authority.
    LifecycleBindingMissing,
}

/// Browser-issued identities returned by the reviewed WebDriver BiDi lifecycle backend.
///
/// The user-context isolation identity and remote browsing-context string are adapter addressability,
/// not mutation authority. They become usable for presentation planning only after the owning
/// [`BrowserSession`] revalidates the exact retained authority against its current epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBidiCreatedContext {
    isolation: DisposableIsolationId,
    context: WebDriverBidiBrowsingContext,
}

impl WebDriverBidiCreatedContext {
    /// Bind one browser-issued user-context identity to its exact remote browsing context.
    #[must_use]
    pub fn new(isolation: DisposableIsolationId, context: WebDriverBidiBrowsingContext) -> Self {
        Self { isolation, context }
    }

    /// Return the browser-issued disposable user-context identity.
    #[must_use]
    pub const fn isolation(&self) -> &DisposableIsolationId {
        &self.isolation
    }

    /// Return the exact opaque WebDriver BiDi browsing-context identity.
    #[must_use]
    pub const fn context(&self) -> &WebDriverBidiBrowsingContext {
        &self.context
    }
}

/// Reviewed transport boundary used by the WebDriver BiDi disposable-context adapter.
///
/// A production implementation maps creation to `browser.createUserContext` followed by
/// `browsingContext.create`, returning the browser-issued identities without coercing OriginWeave's
/// numeric domain context id. Destruction maps to the exact user-context boundary and may return
/// success only after the remote boundary is proven absent; a command acknowledgement alone is not a
/// destruction post-condition.
pub trait WebDriverBidiLifecycleBackend {
    /// Create one disposable user context and one independently navigable context inside it.
    fn create_disposable_context(
        &mut self,
        session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
    ) -> Result<WebDriverBidiCreatedContext, DisposableContextCreateError>;

    /// Destroy the exact disposable user-context boundary represented by these browser-issued ids.
    fn destroy_disposable_context(
        &mut self,
        session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        isolation: &DisposableIsolationId,
        remote_context: &WebDriverBidiBrowsingContext,
    ) -> Result<(), DisposableContextDestroyError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LifecycleBindingKey {
    session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    isolation: DisposableIsolationId,
    browsing_context: BrowsingContextId,
}

impl LifecycleBindingKey {
    fn from_handle(
        session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        handle: &DisposableContextHandle,
    ) -> Self {
        Self {
            session,
            incarnation,
            isolation: handle.isolation().clone(),
            browsing_context: handle.browsing_context(),
        }
    }

    fn from_authority(authority: &PresentationMutationAuthority) -> Self {
        Self {
            session: authority.browser_session(),
            incarnation: authority.incarnation(),
            isolation: authority.isolation().clone(),
            browsing_context: authority.browsing_context(),
        }
    }
}

/// WebDriver BiDi adapter that keeps remote context addressability bound to disposable lifecycle.
///
/// The private mapping is populated only by this adapter's successful lifecycle creation call. A raw
/// domain [`BrowsingContextId`] is never stringified into a protocol id, and callers cannot provide an
/// unrelated remote context to the authorization method. The mapping key includes Browser Session
/// incarnation and isolation identity so sequential id reuse cannot redirect a retained authority.
#[derive(Debug)]
pub struct WebDriverBidiLifecycleAdapter<B> {
    backend: B,
    next_browsing_context: u64,
    bindings: BTreeMap<LifecycleBindingKey, WebDriverBidiBrowsingContext>,
}

impl<B> WebDriverBidiLifecycleAdapter<B> {
    /// Create an adapter with a fresh, process-local monotonic domain-context allocator.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            next_browsing_context: 1,
            bindings: BTreeMap::new(),
        }
    }
}

impl<B: WebDriverBidiLifecycleBackend> WebDriverBidiLifecycleAdapter<B> {
    /// Revalidate one retained Browser Session authority and bind it to its exact BiDi context.
    ///
    /// The returned plan borrows both this adapter mapping and the Browser Session. While the plan is
    /// alive, safe Rust therefore cannot mutably advance/destroy/end that session or mutate this
    /// lifecycle adapter. Stale epoch, destruction, transport loss, recovery state, session end, or a
    /// different incarnation is rejected before a remote target is returned.
    pub fn authorize_standard_presentation<'a>(
        &'a self,
        session: &'a BrowserSession,
        authority: &PresentationMutationAuthority,
        viewport: ViewportBounds,
        device_pixel_ratio: DevicePixelRatio,
        timezone: PresentationTimeZone,
    ) -> Result<AuthorizedWebDriverBidiPresentationPlan<'a>, WebDriverBidiAclError> {
        let current = session
            .presentation_authority(authority.browsing_context())
            .map_err(WebDriverBidiAclError::BrowserSession)?;
        if current != *authority {
            return Err(WebDriverBidiAclError::BrowserSession(
                BrowserSessionError::AuthorityMismatch,
            ));
        }
        let key = LifecycleBindingKey::from_authority(&current);
        let context = self
            .bindings
            .get(&key)
            .ok_or(WebDriverBidiAclError::LifecycleBindingMissing)?;
        Ok(AuthorizedWebDriverBidiPresentationPlan {
            context,
            viewport,
            device_pixel_ratio,
            timezone,
            _session: PhantomData,
        })
    }
}

impl<B: WebDriverBidiLifecycleBackend> DisposableContextPort for WebDriverBidiLifecycleAdapter<B> {
    fn create_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        let browsing_context = reserve_domain_context_id(&mut self.next_browsing_context)?;
        let created = self
            .backend
            .create_disposable_context(browser_session, incarnation)?;
        let handle = DisposableContextHandle::new(created.isolation.clone(), browsing_context);
        let key = LifecycleBindingKey::from_handle(browser_session, incarnation, &handle);
        self.bindings.insert(key, created.context);
        Ok(handle)
    }

    fn destroy_disposable_context(
        &mut self,
        browser_session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        context: &DisposableContextHandle,
    ) -> Result<(), DisposableContextDestroyError> {
        let key = LifecycleBindingKey::from_handle(browser_session, incarnation, context);
        let remote_context = self
            .bindings
            .get(&key)
            .cloned()
            .ok_or(DisposableContextDestroyError::DestroyFailed)?;
        self.backend.destroy_disposable_context(
            browser_session,
            incarnation,
            context.isolation(),
            &remote_context,
        )?;
        self.bindings.remove(&key);
        Ok(())
    }
}

fn reserve_domain_context_id(
    next: &mut u64,
) -> Result<BrowsingContextId, DisposableContextCreateError> {
    let value = *next;
    *next = value
        .checked_add(1)
        .ok_or(DisposableContextCreateError::CreateFailedClean)?;
    BrowsingContextId::new(value).map_err(|_error| DisposableContextCreateError::CreateFailedClean)
}

/// Standard presentation operation authorized for one currently owned BiDi context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBidiPresentationOperation {
    /// Set viewport dimensions and device-pixel ratio.
    SetViewport,
    /// Set the named time zone.
    SetTimezone,
    /// Remove owned viewport and device-pixel-ratio overrides.
    ResetViewport,
    /// Remove the owned time-zone override.
    ResetTimezone,
}

/// Non-constructible, lifetime-bound standard presentation action.
///
/// Only [`AuthorizedWebDriverBidiPresentationPlan`] can create this value. The action retains the
/// plan's Browser Session lifetime and exact adapter-owned remote context, so raw ids cannot be
/// substituted between policy authorization and transport planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizedWebDriverBidiPresentationAction<'a> {
    operation: WebDriverBidiPresentationOperation,
    context: &'a WebDriverBidiBrowsingContext,
    viewport: Option<ViewportBounds>,
    device_pixel_ratio: Option<DevicePixelRatio>,
    timezone: Option<PresentationTimeZone>,
    _session: PhantomData<&'a BrowserSession>,
}

impl AuthorizedWebDriverBidiPresentationAction<'_> {
    /// Return the standard WebDriver BiDi presentation operation.
    #[must_use]
    pub const fn operation(&self) -> WebDriverBidiPresentationOperation {
        self.operation
    }

    /// Return the exact remote context selected by the lifecycle adapter mapping.
    #[must_use]
    pub const fn context(&self) -> &WebDriverBidiBrowsingContext {
        self.context
    }

    /// Return viewport payload only for [`WebDriverBidiPresentationOperation::SetViewport`].
    #[must_use]
    pub const fn viewport(&self) -> Option<ViewportBounds> {
        self.viewport
    }

    /// Return device-pixel-ratio payload only for [`WebDriverBidiPresentationOperation::SetViewport`].
    #[must_use]
    pub const fn device_pixel_ratio(&self) -> Option<DevicePixelRatio> {
        self.device_pixel_ratio
    }

    /// Return time-zone payload only for [`WebDriverBidiPresentationOperation::SetTimezone`].
    #[must_use]
    pub const fn timezone(&self) -> Option<PresentationTimeZone> {
        self.timezone
    }
}

/// Lifetime-bound plan for standard BiDi presentation mutation and cleanup.
///
/// Constructing this plan is policy admission, not browser success. A later transport must still
/// observe the browser/page post-condition and provenance required by OriginWeave before reporting a
/// successful interaction.
#[derive(Debug, PartialEq, Eq)]
pub struct AuthorizedWebDriverBidiPresentationPlan<'a> {
    context: &'a WebDriverBidiBrowsingContext,
    viewport: ViewportBounds,
    device_pixel_ratio: DevicePixelRatio,
    timezone: PresentationTimeZone,
    _session: PhantomData<&'a BrowserSession>,
}

impl AuthorizedWebDriverBidiPresentationPlan<'_> {
    /// Return the exact remote context selected by the lifecycle adapter mapping.
    #[must_use]
    pub const fn context(&self) -> &WebDriverBidiBrowsingContext {
        self.context
    }

    /// Materialize the two standard mutation actions while retaining this plan's lifetime.
    #[must_use]
    pub fn apply_actions(&self) -> [AuthorizedWebDriverBidiPresentationAction<'_>; 2] {
        [
            AuthorizedWebDriverBidiPresentationAction {
                operation: WebDriverBidiPresentationOperation::SetViewport,
                context: self.context,
                viewport: Some(self.viewport),
                device_pixel_ratio: Some(self.device_pixel_ratio),
                timezone: None,
                _session: PhantomData,
            },
            AuthorizedWebDriverBidiPresentationAction {
                operation: WebDriverBidiPresentationOperation::SetTimezone,
                context: self.context,
                viewport: None,
                device_pixel_ratio: None,
                timezone: Some(self.timezone),
                _session: PhantomData,
            },
        ]
    }

    /// Materialize default-reset cleanup for only the same currently owned lifecycle.
    #[must_use]
    pub fn cleanup_actions(&self) -> [AuthorizedWebDriverBidiPresentationAction<'_>; 2] {
        [
            AuthorizedWebDriverBidiPresentationAction {
                operation: WebDriverBidiPresentationOperation::ResetViewport,
                context: self.context,
                viewport: None,
                device_pixel_ratio: None,
                timezone: None,
                _session: PhantomData,
            },
            AuthorizedWebDriverBidiPresentationAction {
                operation: WebDriverBidiPresentationOperation::ResetTimezone,
                context: self.context,
                viewport: None,
                device_pixel_ratio: None,
                timezone: None,
                _session: PhantomData,
            },
        ]
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Backend {
        created: Option<WebDriverBidiCreatedContext>,
        fail_create: bool,
        fail_destroy: bool,
        destroy_calls: usize,
    }

    impl Backend {
        fn healthy() -> Self {
            Self {
                created: Some(WebDriverBidiCreatedContext::new(
                    isolation("user-context"),
                    WebDriverBidiBrowsingContext::new("remote-context")
                        .expect("valid remote context"),
                )),
                fail_create: false,
                fail_destroy: false,
                destroy_calls: 0,
            }
        }
    }

    impl WebDriverBidiLifecycleBackend for Backend {
        fn create_disposable_context(
            &mut self,
            _session: BrowserSessionId,
            _incarnation: BrowserSessionIncarnation,
        ) -> Result<WebDriverBidiCreatedContext, DisposableContextCreateError> {
            if self.fail_create {
                return Err(DisposableContextCreateError::CreateFailedClean);
            }
            self.created
                .take()
                .ok_or(DisposableContextCreateError::CreateFailedClean)
        }

        fn destroy_disposable_context(
            &mut self,
            _session: BrowserSessionId,
            _incarnation: BrowserSessionIncarnation,
            _isolation: &DisposableIsolationId,
            _remote_context: &WebDriverBidiBrowsingContext,
        ) -> Result<(), DisposableContextDestroyError> {
            self.destroy_calls += 1;
            if self.fail_destroy {
                Err(DisposableContextDestroyError::DestroyFailed)
            } else {
                Ok(())
            }
        }
    }

    fn isolation(value: &str) -> DisposableIsolationId {
        DisposableIsolationId::parse(value).expect("valid isolation")
    }

    fn session_id(value: u64) -> BrowserSessionId {
        BrowserSessionId::new(value).expect("valid session")
    }

    #[test]
    fn created_context_keeps_browser_issued_identities() {
        let created = WebDriverBidiCreatedContext::new(
            isolation("user-context-a"),
            WebDriverBidiBrowsingContext::new("remote-a").expect("valid remote context"),
        );
        assert_eq!(created.isolation().as_str(), "user-context-a");
        assert_eq!(created.context().as_str(), "remote-a");
    }

    #[test]
    fn domain_context_allocator_fails_closed_without_wrapping_or_zero() {
        let mut next = 1;
        assert_eq!(
            reserve_domain_context_id(&mut next)
                .expect("first id")
                .value(),
            1
        );
        assert_eq!(next, 2);

        let mut exhausted = u64::MAX;
        assert_eq!(
            reserve_domain_context_id(&mut exhausted),
            Err(DisposableContextCreateError::CreateFailedClean)
        );

        let mut zero = 0;
        assert_eq!(
            reserve_domain_context_id(&mut zero),
            Err(DisposableContextCreateError::CreateFailedClean)
        );
    }

    #[test]
    fn adapter_forwards_clean_create_failure_without_binding() {
        let mut backend = Backend::healthy();
        backend.fail_create = true;
        let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
        let incarnation = BrowserSessionIncarnation::from_test_value(1);
        assert_eq!(
            adapter.create_disposable_context(session_id(1), incarnation),
            Err(DisposableContextCreateError::CreateFailedClean)
        );
        assert!(adapter.bindings.is_empty());
    }

    #[test]
    fn destroy_requires_exact_binding_and_retains_binding_on_backend_failure() {
        let mut adapter = WebDriverBidiLifecycleAdapter::new(Backend::healthy());
        let incarnation = BrowserSessionIncarnation::from_test_value(2);
        let handle = adapter
            .create_disposable_context(session_id(2), incarnation)
            .expect("create mapping");
        adapter.backend.fail_destroy = true;
        assert_eq!(
            adapter.destroy_disposable_context(session_id(2), incarnation, &handle),
            Err(DisposableContextDestroyError::DestroyFailed)
        );
        assert_eq!(adapter.backend.destroy_calls, 1);
        assert_eq!(adapter.bindings.len(), 1);

        adapter.bindings.clear();
        assert_eq!(
            adapter.destroy_disposable_context(session_id(2), incarnation, &handle),
            Err(DisposableContextDestroyError::DestroyFailed)
        );
        assert_eq!(adapter.backend.destroy_calls, 1);
    }

    #[test]
    fn authorization_fails_if_lifecycle_mapping_is_missing() {
        let mut adapter = WebDriverBidiLifecycleAdapter::new(Backend::healthy());
        let mut session = BrowserSession::start(session_id(3)).expect("session");
        let authority = session
            .create_disposable_context(&mut adapter)
            .expect("owned context");
        adapter.bindings.clear();
        assert_eq!(
            adapter.authorize_standard_presentation(
                &session,
                &authority,
                ViewportBounds::new(800, 600).expect("viewport"),
                DevicePixelRatio::Quantized1,
                PresentationTimeZone::Utc,
            ),
            Err(WebDriverBidiAclError::LifecycleBindingMissing)
        );
    }
}
