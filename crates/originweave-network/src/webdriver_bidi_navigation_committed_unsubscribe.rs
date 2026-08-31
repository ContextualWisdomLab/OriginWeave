use std::{error::Error, fmt, time::Duration};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiNavigationCommittedSubscriptionResult,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_UNSUBSCRIBE_METHOD: &str = "session.unsubscribe";

/// One bounded WebDriver BiDi `session.unsubscribe` command for a validated subscription receipt.
///
/// The command deliberately accepts only the typed opaque identifier returned by OriginWeave's
/// `session.subscribe` response boundary. It cannot introduce arbitrary event names, contexts,
/// user contexts, or ambient subscription identifiers. Writing the frame does not prove remote
/// teardown; callers must admit and correlate the later protocol response separately.
#[derive(Clone, Eq, PartialEq)]
pub struct WebDriverBiDiNavigationCommittedUnsubscribeCommand {
    command_id: u64,
    subscription_id: String,
}

impl fmt::Debug for WebDriverBiDiNavigationCommittedUnsubscribeCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiNavigationCommittedUnsubscribeCommand")
            .field("command_id", &self.command_id)
            .field("subscription_id_len", &self.subscription_id.len())
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedUnsubscribeCommand {
    /// Construct one unsubscribe command from an already validated typed subscription receipt.
    pub fn new(
        command_id: u64,
        subscription: &WebDriverBiDiNavigationCommittedSubscriptionResult,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedUnsubscribeCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(
                WebDriverBiDiNavigationCommittedUnsubscribeCommandError::CommandIdOutOfRange {
                    command_id,
                    maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
                },
            );
        }
        Ok(Self {
            command_id,
            subscription_id: subscription.subscription_id().to_owned(),
        })
    }

    /// Return the exact local correlation identifier serialized for this command.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Register and write this exact unsubscribe command on an established verified BiDi stream.
    ///
    /// Correlation registration occurs before the first possible remote side effect. A local
    /// registration failure therefore writes nothing. Once registered, a frame-write failure keeps
    /// the identifier outstanding because the peer may have received a partial or complete command;
    /// silently retiring the id would make later response correlation or identifier reuse unsafe.
    pub fn send(
        self,
        established: WebDriverBiDiWebSocketEstablished,
        correlation: &mut WebDriverBiDiCommandCorrelation,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<
        WebDriverBiDiWebSocketEstablished,
        WebDriverBiDiNavigationCommittedUnsubscribeCommandError,
    > {
        correlation
            .register_command(self.command_id)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedUnsubscribeCommandError::Correlation { source }
            })?;
        let message = self.serialized();
        established
            .write_text_frame(&message, masking_key, frame_timeout)
            .map_err(|source| {
                WebDriverBiDiNavigationCommittedUnsubscribeCommandError::FrameWrite { source }
            })
    }

    fn serialized(&self) -> String {
        serialize_unsubscribe_command(self.command_id, &self.subscription_id)
    }
}

/// Fail-closed errors while constructing or sending one typed `session.unsubscribe` command.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedUnsubscribeCommandError {
    /// The requested command identifier is outside WebDriver BiDi's `js-uint` range.
    CommandIdOutOfRange {
        /// Rejected command identifier.
        command_id: u64,
        /// Largest JavaScript-safe identifier admitted by this boundary.
        maximum_command_id: u64,
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

impl fmt::Display for WebDriverBiDiNavigationCommittedUnsubscribeCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIdOutOfRange { .. } => formatter.write_str(
                "WebDriver BiDi session.unsubscribe command id is outside the js-uint range",
            ),
            Self::Correlation { .. } => formatter
                .write_str("WebDriver BiDi session.unsubscribe command correlation was rejected"),
            Self::FrameWrite { .. } => formatter
                .write_str("WebDriver BiDi session.unsubscribe command frame write failed"),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedUnsubscribeCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandIdOutOfRange { .. } => None,
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

fn serialize_unsubscribe_command(command_id: u64, subscription_id: &str) -> String {
    let mut message = format!(
        "{{\"id\":{command_id},\"method\":\"{SESSION_UNSUBSCRIBE_METHOD}\",\"params\":{{\"subscriptions\":[\""
    );
    push_json_string_content(&mut message, subscription_id);
    message.push_str("\"]}}");
    message
}

fn push_json_string_content(output: &mut String, input: &str) {
    for character in input.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{001f}' => {
                let code = character as usize;
                let digits = b"0123456789abcdef";
                output.push_str("\\u00");
                output.push(char::from(digits[(code >> 4) & 0x0f]));
                output.push(char::from(digits[code & 0x0f]));
            }
            character => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn serializer_preserves_utf8_and_escapes_every_json_control_class() {
        let input = "quote\" slash\\ back\u{0008} form\u{000c} line\n return\r tab\t nul\u{0000} unit\u{0001} 구독";
        assert_eq!(
            serialize_unsubscribe_command(42, input),
            r#"{"id":42,"method":"session.unsubscribe","params":{"subscriptions":["quote\" slash\\ back\b form\f line\n return\r tab\t nul\u0000 unit\u0001 구독"]}}"#
        );
    }

    #[test]
    fn command_errors_have_stable_messages_and_typed_sources() {
        let range = WebDriverBiDiNavigationCommittedUnsubscribeCommandError::CommandIdOutOfRange {
            command_id: MAX_WEBDRIVER_BIDI_JS_UINT + 1,
            maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
        };
        assert_eq!(
            range.to_string(),
            "WebDriver BiDi session.unsubscribe command id is outside the js-uint range"
        );
        assert!(range.source().is_none());

        let correlation = WebDriverBiDiNavigationCommittedUnsubscribeCommandError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.unsubscribe command correlation was rejected"
        );
        assert!(correlation.source().is_some());

        let frame = WebDriverBiDiNavigationCommittedUnsubscribeCommandError::FrameWrite {
            source: WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 0,
                source: io::Error::other("test frame failure"),
            },
        };
        assert_eq!(
            frame.to_string(),
            "WebDriver BiDi session.unsubscribe command frame write failed"
        );
        assert!(frame.source().is_some());
    }
}
