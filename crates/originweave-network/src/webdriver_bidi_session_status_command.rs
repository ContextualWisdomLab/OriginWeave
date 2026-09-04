use std::{error::Error, fmt, time::Duration};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiCommandKind,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

const SESSION_STATUS_METHOD: &str = "session.status";

/// One bounded WebDriver BiDi `session.status` command.
///
/// The command is deliberately concrete rather than a generic JSON or arbitrary-method escape
/// hatch. It carries only a WebDriver BiDi `js-uint` correlation identifier and always serializes
/// the standards-defined empty parameter map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionStatusCommand {
    command_id: u64,
}

impl WebDriverBiDiSessionStatusCommand {
    /// Construct one `session.status` command with a JavaScript-safe correlation identifier.
    pub fn new(command_id: u64) -> Result<Self, WebDriverBiDiSessionStatusCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(
                WebDriverBiDiSessionStatusCommandError::CommandIdOutOfRange {
                    command_id,
                    maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
                },
            );
        }
        Ok(Self { command_id })
    }

    /// Return the exact local correlation identifier serialized for this command.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Register and write this exact command on an already established verified BiDi stream.
    ///
    /// Locally invalid frame deadlines fail before correlation registration and before any remote
    /// side effect. Correlation then registers the command before the first possible frame write.
    /// A frame-owner preflight rejection that proves no write began retires this exact command
    /// again; currently that covers adjacent client masking-key reuse. Once frame emission can have
    /// begun, a later failure leaves the identifier outstanding because partial or full emission is
    /// ambiguous. Callers must treat that failed stream/correlation pairing as unusable or
    /// explicitly tear down its session state.
    pub fn send(
        self,
        established: WebDriverBiDiWebSocketEstablished,
        correlation: &mut WebDriverBiDiCommandCorrelation,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiSessionStatusCommandError> {
        if frame_timeout.is_zero() || frame_timeout > MAX_WEBSOCKET_FRAME_TIMEOUT {
            return Err(WebDriverBiDiSessionStatusCommandError::FrameWrite {
                source: WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
                    frame_timeout,
                    maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
                },
            });
        }
        correlation
            .register_command_for(self.command_id, WebDriverBiDiCommandKind::SessionStatus)
            .map_err(|source| WebDriverBiDiSessionStatusCommandError::Correlation { source })?;
        let message = self.serialized();
        match established.write_text_frame(&message, masking_key, frame_timeout) {
            Ok(established) => Ok(established),
            Err(source) => {
                if frame_failure_precedes_possible_write(&source) {
                    correlation
                        .retire_command_for(self.command_id, WebDriverBiDiCommandKind::SessionStatus)
                        .map_err(|source| {
                            WebDriverBiDiSessionStatusCommandError::Correlation { source }
                        })?;
                }
                Err(WebDriverBiDiSessionStatusCommandError::FrameWrite { source })
            }
        }
    }

    fn serialized(self) -> String {
        format!(
            "{{\"id\":{},\"method\":\"{SESSION_STATUS_METHOD}\",\"params\":{{}}}}",
            self.command_id
        )
    }
}

fn frame_failure_precedes_possible_write(source: &WebDriverBiDiWebSocketFrameError) -> bool {
    matches!(source, WebDriverBiDiWebSocketFrameError::MalformedFrame { .. })
}

/// Fail-closed errors while constructing or sending one typed `session.status` command.
#[derive(Debug)]
pub enum WebDriverBiDiSessionStatusCommandError {
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
    /// Frame preflight validation or a later write operation failed.
    FrameWrite {
        /// Exact typed bounded WebSocket frame validation/write failure.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiSessionStatusCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIdOutOfRange { .. } => formatter
                .write_str("WebDriver BiDi session.status command id is outside the js-uint range"),
            Self::Correlation { .. } => formatter
                .write_str("WebDriver BiDi session.status command correlation was rejected"),
            Self::FrameWrite { .. } => {
                formatter.write_str("WebDriver BiDi session.status command frame write failed")
            }
        }
    }
}

impl Error for WebDriverBiDiSessionStatusCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CommandIdOutOfRange { .. } => None,
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
    fn constructor_enforces_the_webdriver_bidi_js_uint_range() {
        let accepted = WebDriverBiDiSessionStatusCommand::new(MAX_WEBDRIVER_BIDI_JS_UINT);
        assert_eq!(
            accepted.ok().map(|command| command.command_id()),
            Some(MAX_WEBDRIVER_BIDI_JS_UINT)
        );

        let rejected = WebDriverBiDiSessionStatusCommand::new(MAX_WEBDRIVER_BIDI_JS_UINT + 1);
        assert_eq!(
            rejected.err().map(|error| error.to_string()).as_deref(),
            Some("WebDriver BiDi session.status command id is outside the js-uint range")
        );
    }

    #[test]
    fn command_serialization_is_static_and_exact() {
        let command = WebDriverBiDiSessionStatusCommand { command_id: 42 };
        assert_eq!(command.command_id(), 42);
        assert_eq!(
            command.serialized(),
            r#"{"id":42,"method":"session.status","params":{}}"#
        );
    }

    #[test]
    fn command_errors_have_stable_messages_and_typed_sources() {
        let range = WebDriverBiDiSessionStatusCommandError::CommandIdOutOfRange {
            command_id: MAX_WEBDRIVER_BIDI_JS_UINT + 1,
            maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
        };
        assert_eq!(
            range.to_string(),
            "WebDriver BiDi session.status command id is outside the js-uint range"
        );
        assert!(range.source().is_none());

        let correlation = WebDriverBiDiSessionStatusCommandError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.status command correlation was rejected"
        );
        assert!(correlation.source().is_some());

        let frame = WebDriverBiDiSessionStatusCommandError::FrameWrite {
            source: WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 0,
                source: io::Error::other("test frame failure"),
            },
        };
        assert_eq!(
            frame.to_string(),
            "WebDriver BiDi session.status command frame write failed"
        );
        assert!(frame.source().is_some());
    }

    #[test]
    fn only_frame_preflight_malformed_errors_retire_registered_correlation() {
        let preflight = WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "test preflight rejection",
        };
        assert!(frame_failure_precedes_possible_write(&preflight));

        let ambiguous = WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
            bytes_written: 1,
            source: io::Error::other("test ambiguous write failure"),
        };
        assert!(!frame_failure_precedes_possible_write(&ambiguous));
    }
}
