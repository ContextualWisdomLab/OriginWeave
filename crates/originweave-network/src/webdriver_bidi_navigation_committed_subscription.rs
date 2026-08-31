use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WEBDRIVER_BIDI_NAVIGATION_COMMITTED_METHOD,
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
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

    /// Revalidate, register, and write this exact subscription on an established verified BiDi stream.
    ///
    /// Context binding is revalidated immediately before command correlation and network I/O so a
    /// command retained across registry retirement cannot subscribe a stale or replacement context.
    /// Correlation registration then occurs before the first possible remote side effect. A binding
    /// or correlation failure therefore writes nothing. After successful registration, a frame-write
    /// failure consumes the transport and intentionally leaves the identifier outstanding because a
    /// partial or fully emitted frame has ambiguous remote effect.
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
        require_registered_context(
            registry,
            self.browser_session,
            self.browsing_context,
            &self.external_context,
        )?;
        correlation
            .register_command(self.command_id)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::Correlation { source }
            })?;
        let message = self.serialized();
        established
            .write_text_frame(&message, masking_key, frame_timeout)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedSubscriptionCommandError::FrameWrite { source }
            })
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
    /// The external protocol context does not name the exact registered OriginWeave context.
    ContextBinding {
        /// Exact typed browser-registry authority failure.
        source: BrowserRegistryError,
    },
    /// The bounded local correlation registry rejected the command before network I/O.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// Writing the already-registered command frame failed and the transport is not reusable.
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
    fn constructor_binds_the_exact_registered_context_and_js_uint_range()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry.register_session("session-a")?;
        let context = registry.register_context(session, "context-a")?;

        let command = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
            MAX_WEBDRIVER_BIDI_JS_UINT,
            &registry,
            session,
            context,
            "context-a",
        );
        assert!(command.is_ok());
        assert_eq!(
            command.as_ref().map(|command| command.command_id()).ok(),
            Some(MAX_WEBDRIVER_BIDI_JS_UINT)
        );
        assert_eq!(
            command
                .as_ref()
                .map(|command| command.browser_session())
                .ok(),
            Some(session)
        );
        assert_eq!(
            command
                .as_ref()
                .map(|command| command.browsing_context())
                .ok(),
            Some(context)
        );
        assert_eq!(
            command
                .as_ref()
                .map(|command| command.external_context())
                .ok(),
            Some("context-a")
        );

        let range = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
            MAX_WEBDRIVER_BIDI_JS_UINT + 1,
            &registry,
            session,
            context,
            "context-a",
        );
        assert_eq!(
            range.err().map(|error| error.to_string()).as_deref(),
            Some("WebDriver BiDi navigation subscription command id is outside the js-uint range")
        );

        let mismatch = WebDriverBiDiNavigationCommittedSubscriptionCommand::new(
            1,
            &registry,
            session,
            context,
            "context-b",
        );
        assert_eq!(
            mismatch.err().map(|error| error.to_string()).as_deref(),
            Some(
                "WebDriver BiDi navigation subscription context does not match registered authority"
            )
        );
        Ok(())
    }

    #[test]
    fn serialization_is_narrow_exact_and_json_escapes_the_registered_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = BrowserAuthorityRegistry::new();
        let session = registry.register_session("session-a")?;
        let context = registry.register_context(session, "context-a")?;
        let command = WebDriverBiDiNavigationCommittedSubscriptionCommand {
            command_id: 42,
            browser_session: session,
            browsing_context: context,
            external_context: "context-\"a\\b".to_owned(),
        };
        assert_eq!(
            command.serialized(),
            r#"{"id":42,"method":"session.subscribe","params":{"events":["browsingContext.navigationCommitted"],"contexts":["context-\"a\\b"]}}"#
        );
        Ok(())
    }

    #[test]
    fn command_errors_have_stable_messages_and_typed_sources() {
        let range = WebDriverBiDiNavigationCommittedSubscriptionCommandError::CommandIdOutOfRange {
            command_id: MAX_WEBDRIVER_BIDI_JS_UINT + 1,
            maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
        };
        assert!(range.source().is_none());

        let context = WebDriverBiDiNavigationCommittedSubscriptionCommandError::ContextBinding {
            source: BrowserRegistryError::UnknownBrowserSession,
        };
        assert!(context.source().is_some());

        let correlation = WebDriverBiDiNavigationCommittedSubscriptionCommandError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding,
        };
        assert!(correlation.source().is_some());

        let frame = WebDriverBiDiNavigationCommittedSubscriptionCommandError::FrameWrite {
            source: WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 0,
                source: io::Error::other("test frame failure"),
            },
        };
        assert!(frame.source().is_some());
    }
}
