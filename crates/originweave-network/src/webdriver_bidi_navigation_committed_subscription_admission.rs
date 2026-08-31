use std::{error::Error, fmt};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
};

use crate::{
    WebDriverBiDiNavigationCommittedObservation, WebDriverBiDiNavigationCommittedObservationError,
    WebDriverBiDiNavigationCommittedSubscriptionResult,
    WebDriverBiDiNavigationCommittedUnsubscribeCommand,
    WebDriverBiDiNavigationCommittedUnsubscribeCommandError, WebDriverBiDiWebSocketTextMessage,
};

/// Maximum distinct committed-navigation identifiers retained by one active subscription admission.
///
/// Exhaustion fails closed instead of evicting old identifiers because eviction would permit an old
/// protocol event to become fresh state-changing evidence again. Callers can explicitly unsubscribe
/// and establish a new typed subscription when this reviewed per-subscription resource bound is met.
pub const MAX_WEBDRIVER_BIDI_NAVIGATION_COMMITTED_ADMISSIONS: usize = 256;

/// Immutable command-side binding retained before a committed-navigation subscription is sent.
///
/// The binding carries only the exact local command identifier and the already-registered
/// OriginWeave session/context association used to serialize that command. The external BiDi
/// context identifier is retained privately for immediate registry revalidation and is not exposed
/// as durable OriginWeave authority.
pub struct WebDriverBiDiNavigationCommittedSubscriptionBinding {
    command_id: u64,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    external_context: String,
}

impl fmt::Debug for WebDriverBiDiNavigationCommittedSubscriptionBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiNavigationCommittedSubscriptionBinding")
            .field("command_id", &self.command_id)
            .field("browser_session", &self.browser_session.value())
            .field("browsing_context", &self.browsing_context.value())
            .field("external_context_bytes", &self.external_context.len())
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedSubscriptionBinding {
    pub(crate) fn new(
        command_id: u64,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        external_context: &str,
    ) -> Self {
        Self {
            command_id,
            browser_session,
            browsing_context,
            external_context: external_context.to_owned(),
        }
    }

    /// Return the exact local command identifier this binding was captured from.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the exact registered OriginWeave browser session bound to the subscription command.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact registered OriginWeave browsing context bound to the subscription command.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }
}

/// Active local admission capability for one exact committed-navigation BiDi subscription.
///
/// Construction requires both the correlated remote subscription receipt and the immutable binding
/// captured from the exact command that requested it. The command identifiers must match and the
/// original external context mapping must still resolve to the exact OriginWeave session/context.
/// Holding this value is therefore narrower than holding an opaque protocol subscription string.
/// It grants only admission of the matching committed-navigation event through the existing bounded
/// parser; it grants no navigation, destination, origin, policy, secret, node, or Agent authority.
/// Each admitted non-null WebDriver BiDi navigation identifier is retained until unsubscribe so a
/// replayed remote event cannot mint a second state-changing observation from the same navigation.
pub struct WebDriverBiDiNavigationCommittedSubscriptionAdmission {
    subscription: WebDriverBiDiNavigationCommittedSubscriptionResult,
    binding: WebDriverBiDiNavigationCommittedSubscriptionBinding,
    admitted_navigation_ids: Vec<String>,
}

impl fmt::Debug for WebDriverBiDiNavigationCommittedSubscriptionAdmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiNavigationCommittedSubscriptionAdmission")
            .field("command_id", &self.binding.command_id)
            .field("browser_session", &self.binding.browser_session.value())
            .field("browsing_context", &self.binding.browsing_context.value())
            .field(
                "subscription_id_bytes",
                &self.subscription.subscription_id().len(),
            )
            .field(
                "admitted_navigation_count",
                &self.admitted_navigation_ids.len(),
            )
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedSubscriptionAdmission {
    /// Bind one correlated subscription receipt to the exact command-side session/context intent.
    ///
    /// A response correlated to a different command cannot be rebound to this capability. The
    /// original external BiDi context is revalidated before the capability exists, so a retired or
    /// replaced registry mapping fails closed without creating active event-admission state.
    pub fn new(
        subscription: WebDriverBiDiNavigationCommittedSubscriptionResult,
        binding: WebDriverBiDiNavigationCommittedSubscriptionBinding,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedSubscriptionAdmissionError> {
        if subscription.command_id() != binding.command_id {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionAdmissionError::CommandIdMismatch {
                    subscription_command_id: subscription.command_id(),
                    binding_command_id: binding.command_id,
                },
            );
        }
        require_current_binding(registry, &binding).map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionAdmissionError::ContextBinding { source }
        })?;
        Ok(Self {
            subscription,
            binding,
            admitted_navigation_ids: Vec::new(),
        })
    }

    /// Return the exact registered OriginWeave browser session admitted by this capability.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.binding.browser_session
    }

    /// Return the exact registered OriginWeave browsing context admitted by this capability.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.binding.browsing_context
    }

    /// Admit one exact committed-navigation event while this subscription capability remains active.
    ///
    /// The original command-side external-context mapping is revalidated immediately before parsing
    /// the event. The event must then independently carry that same registered context and the exact
    /// declared URL. State-changing admission additionally requires the WebDriver BiDi navigation
    /// identifier to be present and unique within this active subscription. The specification defines
    /// non-null navigation identifiers as unique identifiers for ongoing navigations; retaining them
    /// prevents replay of an already-admitted event. The history is resource bounded and fails closed
    /// at capacity rather than evicting evidence that would make an older replay admissible again.
    /// The returned subscribed observation is the only navigation observation type accepted by the
    /// state-changing document-advance boundary.
    pub fn admit(
        &mut self,
        message: &WebDriverBiDiWebSocketTextMessage,
        registry: &BrowserAuthorityRegistry,
        expected_url: &str,
    ) -> Result<
        WebDriverBiDiNavigationCommittedSubscribedObservation,
        WebDriverBiDiNavigationCommittedSubscriptionEventError,
    > {
        require_current_binding(registry, &self.binding).map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionEventError::ContextBinding { source }
        })?;
        let observation = WebDriverBiDiNavigationCommittedObservation::parse_and_match(
            message,
            registry,
            self.binding.browser_session,
            self.binding.browsing_context,
            expected_url,
        )
        .map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionEventError::Observation { source }
        })?;
        let navigation_id = observation.navigation_id().ok_or(
            WebDriverBiDiNavigationCommittedSubscriptionEventError::MissingNavigationIdentity,
        )?;
        if self
            .admitted_navigation_ids
            .iter()
            .any(|admitted| admitted == navigation_id)
        {
            return Err(WebDriverBiDiNavigationCommittedSubscriptionEventError::ReplayedNavigation);
        }
        if self.admitted_navigation_ids.len() >= MAX_WEBDRIVER_BIDI_NAVIGATION_COMMITTED_ADMISSIONS
        {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionEventError::ReplayHistoryExhausted {
                    maximum_events: MAX_WEBDRIVER_BIDI_NAVIGATION_COMMITTED_ADMISSIONS,
                },
            );
        }
        self.admitted_navigation_ids.push(navigation_id.to_owned());
        Ok(WebDriverBiDiNavigationCommittedSubscribedObservation(
            observation,
        ))
    }

    /// Consume active event admission and construct teardown for this exact subscription receipt.
    ///
    /// Consumption deliberately ends local event admission before the unsubscribe command can be
    /// emitted. If later transport or remote teardown fails, callers must explicitly establish a new
    /// typed subscription before admitting more events; ambiguous teardown never restores authority.
    pub fn into_unsubscribe(
        self,
        command_id: u64,
    ) -> Result<
        WebDriverBiDiNavigationCommittedUnsubscribeCommand,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError,
    > {
        WebDriverBiDiNavigationCommittedUnsubscribeCommand::new(command_id, &self.subscription)
    }
}

fn require_current_binding(
    registry: &BrowserAuthorityRegistry,
    binding: &WebDriverBiDiNavigationCommittedSubscriptionBinding,
) -> Result<(), BrowserRegistryError> {
    registry.require_registered_context_external_identifier(
        binding.browser_session,
        binding.browsing_context,
        &binding.external_context,
    )
}

/// One committed-navigation observation admitted through an active exact subscription capability.
///
/// Unlike the lower-level protocol observation, this value proves that local admission was bound to
/// the exact typed `session.subscribe` command/receipt pair for the same registered context at the
/// time the event was admitted. It still does not prove action causality or grant destination,
/// origin, policy, node, secret, process, profile, or reusable Agent authority.
pub struct WebDriverBiDiNavigationCommittedSubscribedObservation(
    WebDriverBiDiNavigationCommittedObservation,
);

impl fmt::Debug for WebDriverBiDiNavigationCommittedSubscribedObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("WebDriverBiDiNavigationCommittedSubscribedObservation")
            .field(&self.0)
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedSubscribedObservation {
    /// Return the exact OriginWeave browser session whose active subscription admitted the event.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.0.browser_session()
    }

    /// Return the exact OriginWeave browsing context whose active subscription admitted the event.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.0.browsing_context()
    }

    /// Borrow the optional opaque WebDriver BiDi navigation identifier.
    #[must_use]
    pub fn navigation_id(&self) -> Option<&str> {
        self.0.navigation_id()
    }

    /// Return the admitted WebDriver BiDi monotonic event timestamp.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.0.timestamp()
    }

    /// Borrow the exact bounded serialized URL admitted through the active subscription.
    #[must_use]
    pub fn url(&self) -> &str {
        self.0.url()
    }
}

/// Fail-closed failures while binding a correlated subscription receipt to command-side authority.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedSubscriptionAdmissionError {
    /// The correlated response belongs to a different local command than the supplied binding.
    CommandIdMismatch {
        /// Exact command identifier carried by the correlated subscription receipt.
        subscription_command_id: u64,
        /// Exact command identifier captured from the intended subscription command.
        binding_command_id: u64,
    },
    /// The original external context no longer maps to the exact registered session/context pair.
    ContextBinding {
        /// Exact browser-registry authority failure.
        source: BrowserRegistryError,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedSubscriptionAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIdMismatch { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription response does not match its command binding",
            ),
            Self::ContextBinding { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription context is no longer registered authority",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedSubscriptionAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandIdMismatch { .. } => None,
            Self::ContextBinding { source } => Some(source),
        }
    }
}

/// Fail-closed failures while admitting an event through one active subscription capability.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedSubscriptionEventError {
    /// The original external context no longer maps to the exact registered session/context pair.
    ContextBinding {
        /// Exact browser-registry authority failure.
        source: BrowserRegistryError,
    },
    /// The bounded committed-navigation observation itself could not be admitted.
    Observation {
        /// Underlying typed observation failure.
        source: WebDriverBiDiNavigationCommittedObservationError,
    },
    /// The event did not carry a non-null navigation identity suitable for state-changing evidence.
    MissingNavigationIdentity,
    /// The same WebDriver BiDi navigation identity was already admitted by this active subscription.
    ReplayedNavigation,
    /// The bounded replay-prevention history is full and must not evict older evidence.
    ReplayHistoryExhausted {
        /// Maximum distinct navigation identities retained by one active subscription admission.
        maximum_events: usize,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedSubscriptionEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextBinding { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription context is no longer registered authority",
            ),
            Self::Observation { .. } => {
                formatter.write_str("WebDriver BiDi navigation-committed event is not admissible")
            }
            Self::MissingNavigationIdentity => formatter.write_str(
                "WebDriver BiDi navigation-committed event has no reusable-safe navigation identity",
            ),
            Self::ReplayedNavigation => formatter.write_str(
                "WebDriver BiDi navigation-committed event was already admitted by this active subscription",
            ),
            Self::ReplayHistoryExhausted { maximum_events } => write!(
                formatter,
                "WebDriver BiDi navigation subscription reached its {maximum_events}-event replay-history limit"
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedSubscriptionEventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ContextBinding { source } => Some(source),
            Self::Observation { source } => Some(source),
            Self::MissingNavigationIdentity
            | Self::ReplayedNavigation
            | Self::ReplayHistoryExhausted { .. } => None,
        }
    }
}
