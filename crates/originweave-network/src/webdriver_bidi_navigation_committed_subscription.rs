use std::{error::Error, fmt, sync::Arc, time::Duration};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserRegistryIdentity, BrowserSessionId,
    BrowsingContextId,
};

use crate::webdriver_bidi_websocket_frame::validate_frame_timeout;
use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WEBDRIVER_BIDI_NAVIGATION_COMMITTED_METHOD,
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiNavigationCommittedSubscriptionBinding,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_SUBSCRIBE_METHOD: &str = "session.subscribe";

/// One context-scoped subscription for the committed-navigation WebDriver BiDi event.
///
/// This command is deliberately narrower than the protocol's generic `session.subscribe` surface:
/// it can request only `browsingContext.navigationCommitted`, for one external context that already
/// maps to the exact supplied OriginWeave session/context pair. It does not expose arbitrary event
/// names, global subscriptions, user-context subscriptions, generic JSON, or arbitrary method
/// dispatch. Successful construction or transport does not authenticate Chromium, authorize a
/// navigation, grant destination or policy authority, or make later event data reusable Agent
/// authority.
pub struct WebDriverBiDiNavigationCommittedSubscriptionCommand {
    command_id: u64,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    external_context: String,
    subscription_intent: Arc<()>,
    registry_identity: BrowserRegistryIdentity,
}

impl WebDriverBiDiNavigationCommittedSubscriptionCommand {
    /// Construct one bounded context-scoped committed-navigation subscription command.
    ///
    /// The external protocol identifier must already name the exact registered OriginWeave
    /// session/context pair. No registry state is created as a side effect of untrusted adapter text.
    pub fn new(
        command_id: u64,
        registry: &BrowserAuthorityRegistry,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        external_context: &str,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedSubscriptionCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::CommandIdOutOfRange {
                    command_id,
                    maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
                },
            );
        }
        require_registered_context(
            registry,
            browser_session,
            browsing_context,
            external_context,
        )?;
        Ok(Self {
            command_id,
            browser_session,
            browsing_context,
            external_context: external_context.to_owned(),
            subscription_intent: Arc::new(()),
            registry_identity: registry.registry_identity(),
        })
    }

    /// Return the exact local correlation identifier serialized by this command.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the exact registered OriginWeave browser session bound during construction.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact registered OriginWeave browsing context bound during construction.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Borrow the exact external WebDriver BiDi context identifier serialized by this command.
    #[must_use]
    pub fn external_context(&self) -> &str {
        &self.external_context
    }

    /// Capture the exact command-side binding required to admit the correlated subscription later.
    ///
    /// The binding is intentionally captured before this single-use command is consumed by
    /// [`Self::send`]. It contains no remote subscription identifier and grants no event authority by
    /// itself; the exact correlated `session.subscribe` result must be bound to it separately.
    #[must_use]
    pub fn admission_binding(&self) -> WebDriverBiDiNavigationCommittedSubscriptionBinding {
        WebDriverBiDiNavigationCommittedSubscriptionBinding::new(
            self.command_id,
            self.browser_session,
            self.browsing_context,
            &self.external_context,
            Arc::clone(&self.subscription_intent),
            self.registry_identity.clone(),
        )
    }

    /// Revalidate, register, and write this exact subscription on an established verified BiDi stream.
    ///
    /// The original registry identity and context binding are revalidated before correlation and I/O so a
    /// command retained across registry retirement cannot subscribe a stale or replacement context.
    /// The verified transport's protocol session must also match the registry's canonical external
    /// session mapping; this comparison does not authenticate the browser process.
    /// Invalid frame deadlines fail before correlation registration. Registration then binds both the
    /// private command-instance identity and this established connection's process-local generation
    /// before the first possible remote side effect. A frame-owner preflight rejection that proves no
    /// write began retires this exact subscription again; currently that covers adjacent client
    /// masking-key reuse. Once frame emission can have begun, later failures conservatively leave the
    /// identifier outstanding because partial or full emission is ambiguous.
    pub fn send(
        self,
        registry: &BrowserAuthorityRegistry,
        established: WebDriverBiDiWebSocketEstablished,
        correlation: &mut WebDriverBiDiCommandCorrelation,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<
        WebDriverBiDiWebSocketEstablished,
        WebDriverBiDiNavigationCommittedSubscriptionCommandError,
    > {
        registry
            .require_identity(&self.registry_identity)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::ContextBinding { source }
            })?;
        require_registered_context(
            registry,
            self.browser_session,
            self.browsing_context,
            &self.external_context,
        )?;
        validate_frame_timeout(frame_timeout).map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionCommandError::FrameWrite { source }
        })?;
        registry
            .require_registered_session_external_identifier(
                self.browser_session,
                established
                    .transport_evidence()
                    .verified_peer()
                    .session_id(),
            )
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::ContextBinding { source }
            })?;
        let connection_generation = established.transport_evidence().connection_generation();
        correlation
            .register_subscription_command_for_connection(
                self.command_id,
                connection_generation,
                Arc::clone(&self.subscription_intent),
            )
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::Correlation { source }
            })?;
        let message = self.serialized();
        match established.write_command_frame(self.command_id, &message, masking_key, frame_timeout)
        {
            Ok(established) => Ok(established),
            Err(source) => Err(map_frame_failure(correlation, self.command_id, source)),
        }
    }

    fn serialized(&self) -> String {
        let mut message = format!(
            "{{\"id\":{},\"method\":\"{SESSION_SUBSCRIBE_METHOD}\",\"params\":{{\"events\":[\"{WEBDRIVER_BIDI_NAVIGATION_COMMITTED_METHOD}\"],\"contexts\":[",
            self.command_id
        );
        push_json_string(&mut message, &self.external_context);
        message.push_str("]}}");
        message
    }
}

fn map_frame_failure(
    correlation: &mut WebDriverBiDiCommandCorrelation,
    command_id: u64,
    source: WebDriverBiDiWebSocketFrameError,
) -> WebDriverBiDiNavigationCommittedSubscriptionCommandError {
    if matches!(
        source,
        WebDriverBiDiWebSocketFrameError::MalformedFrame { .. }
    ) {
        let _retirement = correlation.retire_command_for(
            command_id,
            WebDriverBiDiCommandKind::NavigationCommittedSubscription,
        );
    }
    WebDriverBiDiNavigationCommittedSubscriptionCommandError::FrameWrite { source }
}

fn require_registered_context(
    registry: &BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    external_context: &str,
) -> Result<(), WebDriverBiDiNavigationCommittedSubscriptionCommandError> {
    registry
        .require_registered_context_external_identifier(
            browser_session,
            browsing_context,
            external_context,
        )
        .map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionCommandError::ContextBinding { source }
        })
}

fn push_json_string(target: &mut String, value: &str) {
    target.push('"');
    for character in value.chars() {
        match character {
            '"' => target.push_str("\\\""),
            '\\' => target.push_str("\\\\"),
            _ => target.push(character),
        }
    }
    target.push('"');
}

/// Fail-closed failures while constructing or sending one typed committed-navigation subscription.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedSubscriptionCommandError {
    /// The requested command identifier is outside WebDriver BiDi's `js-uint` range.
    CommandIdOutOfRange {
        /// Rejected command identifier.
        command_id: u64,
        /// Largest JavaScript-safe identifier admitted by this boundary.
        maximum_command_id: u64,
    },
    /// The protocol session or context does not match the exact registered OriginWeave authority.
    ContextBinding {
        /// Exact typed browser-registry authority failure.
        source: BrowserRegistryError,
    },
    /// The bounded local correlation registry rejected the command before network I/O.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// Preparing or writing the command frame failed and the transport is not reusable.
    FrameWrite {
        /// Exact typed bounded WebSocket frame-write failure.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedSubscriptionCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIdOutOfRange { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription command id is outside the js-uint range",
            ),
            Self::ContextBinding { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription context does not match registered authority",
            ),
            Self::Correlation { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription command correlation was rejected",
            ),
            Self::FrameWrite { .. } => formatter.write_str(
                "WebDriver BiDi navigation subscription command frame write failed",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedSubscriptionCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandIdOutOfRange { .. } => None,
            Self::ContextBinding { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn only_provably_local_frame_failures_retire_subscription_correlation() {
        let mut correlation = WebDriverBiDiCommandCorrelation::new();
        assert!(
            correlation
                .register_command_for(1, WebDriverBiDiCommandKind::NavigationCommittedSubscription,)
                .is_ok()
        );
        let preflight = WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "test preflight rejection",
        };
        map_frame_failure(&mut correlation, 1, preflight);
        assert_eq!(correlation.outstanding_count(), 0);

        assert!(
            correlation
                .register_command_for(2, WebDriverBiDiCommandKind::NavigationCommittedSubscription,)
                .is_ok()
        );
        let ambiguous = WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
            bytes_written: 1,
            source: io::Error::other("test ambiguous write failure"),
        };
        map_frame_failure(&mut correlation, 2, ambiguous);
        assert_eq!(correlation.outstanding_count(), 1);
    }
}
