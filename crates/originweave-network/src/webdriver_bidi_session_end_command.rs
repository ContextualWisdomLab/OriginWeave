use std::{error::Error, fmt, time::Duration};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketMaskKey,
};

const SESSION_END_METHOD: &str = "session.end";

/// One bounded WebDriver BiDi `session.end` command.
///
/// The command is deliberately concrete rather than a generic JSON or arbitrary-method escape
/// hatch. It carries only a WebDriver BiDi `js-uint` correlation identifier and always serializes
/// the standards-defined empty parameter map. Successfully writing the frame does not claim that
/// the remote session ended; callers must wait for a separately validated correlated response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebDriverBiDiSessionEndCommand {
    command_id: u64,
}

impl WebDriverBiDiSessionEndCommand {
    /// Construct one `session.end` command with a JavaScript-safe correlation identifier.
    pub fn new(command_id: u64) -> Result<Self, WebDriverBiDiSessionEndCommandError> {
        if command_id > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(WebDriverBiDiSessionEndCommandError::CommandIdOutOfRange {
                command_id,
                maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
            });
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
    /// Registration occurs before the first possible remote side effect. A correlation failure
    /// therefore writes nothing. Once registration succeeds, any frame-write failure consumes the
    /// transport and intentionally leaves the identifier outstanding: a partial or fully emitted
    /// frame is ambiguous, so silently retiring the id could allow unsafe reuse. A successful write
    /// also leaves the identifier outstanding until a later correlated response proves completion.
    pub fn send(
        self,
        established: WebDriverBiDiWebSocketEstablished,
        correlation: &mut WebDriverBiDiCommandCorrelation,
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiSessionEndCommandError> {
        correlation
            .register_command(self.command_id)
            .map_err(|source| WebDriverBiDiSessionEndCommandError::Correlation { source })?;
        let message = self.serialized();
        established
            .write_text_frame(&message, masking_key, frame_timeout)
            .map_err(|source| WebDriverBiDiSessionEndCommandError::FrameWrite { source })
    }

    fn serialized(self) -> String {
        format!(
            "{{\"id\":{},\"method\":\"{SESSION_END_METHOD}\",\"params\":{{}}}}",
            self.command_id
        )
    }
}

/// Fail-closed errors while constructing or sending one typed `session.end` command.
#[derive(Debug)]
pub enum WebDriverBiDiSessionEndCommandError {
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

impl fmt::Display for WebDriverBiDiSessionEndCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIdOutOfRange { .. } => formatter
                .write_str("WebDriver BiDi session.end command id is outside the js-uint range"),
            Self::Correlation { .. } => formatter
                .write_str("WebDriver BiDi session.end command correlation was rejected"),
            Self::FrameWrite { .. } => {
                formatter.write_str("WebDriver BiDi session.end command frame write failed")
            }
        }
    }
}

impl Error for WebDriverBiDiSessionEndCommandError {
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
        let accepted = WebDriverBiDiSessionEndCommand::new(MAX_WEBDRIVER_BIDI_JS_UINT);
        assert_eq!(
            accepted.ok().map(|command| command.command_id()),
            Some(MAX_WEBDRIVER_BIDI_JS_UINT)
        );

        let rejected = WebDriverBiDiSessionEndCommand::new(MAX_WEBDRIVER_BIDI_JS_UINT + 1);
        assert_eq!(
            rejected.err().map(|error| error.to_string()).as_deref(),
            Some("WebDriver BiDi session.end command id is outside the js-uint range")
        );
    }

    #[test]
    fn command_serialization_is_static_and_exact() {
        let command = WebDriverBiDiSessionEndCommand { command_id: 42 };
        assert_eq!(command.command_id(), 42);
        assert_eq!(
            command.serialized(),
            r#"{"id":42,"method":"session.end","params":{}}"#
        );
    }

    #[test]
    fn command_errors_have_stable_messages_and_typed_sources() {
        let range = WebDriverBiDiSessionEndCommandError::CommandIdOutOfRange {
            command_id: MAX_WEBDRIVER_BIDI_JS_UINT + 1,
            maximum_command_id: MAX_WEBDRIVER_BIDI_JS_UINT,
        };
        assert_eq!(
            range.to_string(),
            "WebDriver BiDi session.end command id is outside the js-uint range"
        );
        assert!(range.source().is_none());

        let correlation = WebDriverBiDiSessionEndCommandError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandAlreadyOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.end command correlation was rejected"
        );
        assert!(correlation.source().is_some());

        let frame = WebDriverBiDiSessionEndCommandError::FrameWrite {
            source: WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                bytes_written: 0,
                source: io::Error::other("test frame failure"),
            },
        };
        assert_eq!(
            frame.to_string(),
            "WebDriver BiDi session.end command frame write failed"
        );
        assert!(frame.source().is_some());
    }
}
