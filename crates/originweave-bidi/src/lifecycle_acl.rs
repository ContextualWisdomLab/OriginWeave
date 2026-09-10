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

/// Browser-issued and domain identities returned together by the reviewed lifecycle backend.
///
/// The user-context isolation identity, domain context identity, and remote browsing-context string
/// are addressability, not mutation authority. Returning them as one value prevents the ACL from
/// stringifying a numeric domain id or accepting an unrelated remote context from its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBidiCreatedContext {
    isolation: DisposableIsolationId,
    browsing_context: BrowsingContextId,
    remote_context: WebDriverBidiBrowsingContext,
}

impl WebDriverBidiCreatedContext {
    /// Bind one adapter-allocated domain context to the exact browser-issued lifecycle identities.
    #[must_use]
    pub fn new(
        isolation: DisposableIsolationId,
        browsing_context: BrowsingContextId,
        remote_context: WebDriverBidiBrowsingContext,
    ) -> Self {
        Self {
            isolation,
            browsing_context,
            remote_context,
        }
    }

    /// Return the browser-issued disposable user-context identity.
    #[must_use]
    pub const fn isolation(&self) -> &DisposableIsolationId {
        &self.isolation
    }

    /// Return the adapter-allocated OriginWeave domain context identity.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Return the exact opaque WebDriver BiDi browsing-context identity.
    #[must_use]
    pub const fn remote_context(&self) -> &WebDriverBidiBrowsingContext {
        &self.remote_context
    }
}

/// Reviewed transport boundary used by the WebDriver BiDi disposable-context adapter.
///
/// A production implementation maps creation to `browser.createUserContext` followed by
/// `browsingContext.create`. It allocates the OriginWeave [`BrowsingContextId`] itself and returns that
/// domain identity together with the browser-issued user-context and remote browsing-context ids.
/// Destruction maps to the exact user-context boundary and may return success only after the remote
/// boundary is proven absent; a command acknowledgement alone is not a destruction post-condition.
pub trait WebDriverBidiLifecycleBackend {
    /// Create one disposable user context and independently navigable context inside it.
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
/// The private mapping is populated only from this adapter's successful lifecycle backend result. A
/// raw domain [`BrowsingContextId`] is never coerced into a protocol string, and authorization accepts
/// no caller-supplied remote context. The key includes Browser Session incarnation and isolation
/// identity so sequential id reuse cannot redirect a retained authority.
#[derive(Debug)]
pub struct WebDriverBidiLifecycleAdapter<B> {
    backend: B,
    bindings: BTreeMap<LifecycleBindingKey, WebDriverBidiBrowsingContext>,
}

impl<B> WebDriverBidiLifecycleAdapter<B> {
    /// Create an adapter with no ambient or caller-provided lifecycle bindings.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            bindings: BTreeMap::new(),
        }
    }
}

impl<B: WebDriverBidiLifecycleBackend> WebDriverBidiLifecycleAdapter<B> {
    /// Revalidate one retained Browser Session authority and bind it to its exact BiDi context.
    ///
    /// The returned plan borrows both this adapter mapping and the Browser Session. While the plan is
    /// alive, safe Rust cannot mutably advance, destroy, lose, or end that session or mutate this
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
        let created = self
            .backend
            .create_disposable_context(browser_session, incarnation)?;
        let handle = DisposableContextHandle::new(created.isolation.clone(), created.browsing_context);
        let key = LifecycleBindingKey::from_handle(browser_session, incarnation, &handle);
        self.bindings.insert(key, created.remote_context);
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
