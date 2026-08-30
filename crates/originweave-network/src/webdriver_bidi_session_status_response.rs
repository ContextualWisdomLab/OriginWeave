use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError, WebDriverBiDiJsonEnvelopeKind,
    WebDriverBiDiWebSocketTextMessage,
};

/// Maximum decoded byte length retained from WebDriver BiDi `session.status` implementation text.
///
/// The protocol requires an implementation-defined status message but does not define a size
/// ceiling. OriginWeave therefore keeps this operator-facing field within a smaller reviewed bound
/// than the surrounding WebSocket message and never includes its contents in `Debug` output.
pub const MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE: usize = 4_096;

/// Typed, correlated successful result of one WebDriver BiDi `session.status` command.
///
/// This value retains only the exact correlated command id, the standards-defined readiness bit,
/// and one bounded implementation-defined status message. It carries no generic JSON value,
/// browser capability, secret, origin grant, or Agent authority.
#[derive(Eq, PartialEq)]
pub struct WebDriverBiDiSessionStatusResult {
    command_id: u64,
    ready: bool,
    message: String,
}

impl fmt::Debug for WebDriverBiDiSessionStatusResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiSessionStatusResult")
            .field("command_id", &self.command_id)
            .field("ready", &self.ready)
            .field("message_len", &self.message.len())
            .finish()
    }
}

impl WebDriverBiDiSessionStatusResult {
    /// Parse one bounded local-end message and consume its exact outstanding command on success.
    ///
    /// Common WebDriver BiDi envelope validation runs first. A successful envelope then undergoes
    /// command-specific projection of `result.ready` and `result.message`; correlation is consumed
    /// only after that result is valid, so malformed success bodies cannot silently retire an id.
    /// A correlatable protocol-error response consumes its matching id and returns a typed remote
    /// protocol failure. Events, null-id errors, and unknown ids fail closed through the existing
    /// correlation boundary.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiWebSocketTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiSessionStatusResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message)
            .map_err(|source| WebDriverBiDiSessionStatusResponseError::Envelope { source })?;

        match envelope.kind() {
            WebDriverBiDiJsonEnvelopeKind::Success => {
                let projected = StatusProjection::parse(message.as_str())?;
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| WebDriverBiDiSessionStatusResponseError::Correlation {
                        source,
                    })?;
                Ok(Self {
                    command_id: completed.command_id(),
                    ready: projected.ready,
                    message: projected.message,
                })
            }
            WebDriverBiDiJsonEnvelopeKind::Error => {
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| WebDriverBiDiSessionStatusResponseError::Correlation {
                        source,
                    })?;
                Err(WebDriverBiDiSessionStatusResponseError::RemoteProtocolError {
                    command_id: completed.command_id(),
                })
            }
            WebDriverBiDiJsonEnvelopeKind::Event => correlation
                .correlate_response(&envelope)
                .map(|completed| Self {
                    command_id: completed.command_id(),
                    ready: false,
                    message: String::new(),
                })
                .map_err(|source| WebDriverBiDiSessionStatusResponseError::Correlation { source }),
        }
    }

    /// Return the exact local command identifier consumed by this result.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return whether the remote end reports readiness to create a new session.
    #[must_use]
    pub const fn ready(&self) -> bool {
        self.ready
    }

    /// Borrow the bounded implementation-defined readiness message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Fail-closed failures while admitting one typed WebDriver BiDi `session.status` response.
#[derive(Debug)]
pub enum WebDriverBiDiSessionStatusResponseError {
    /// Common local-end JSON envelope validation failed.
    Envelope {
        /// Exact common-envelope validation failure.
        source: WebDriverBiDiJsonEnvelopeError,
    },
    /// The successful result object omits the required `ready` member.
    MissingReady,
    /// The successful result object's `ready` member is not a JSON boolean.
    InvalidReady,
    /// The successful result object omits the required `message` member.
    MissingMessage,
    /// The successful result object's `message` member is not JSON text.
    InvalidMessage,
    /// The successful result repeats one command-specific member and is ambiguous.
    DuplicateResultMember {
        /// Stable command-specific member name that was repeated.
        member: &'static str,
    },
    /// The decoded implementation-defined status message exceeds the reviewed bound.
    MessageTooLarge {
        /// Maximum decoded message length admitted in bytes.
        maximum_bytes: usize,
    },
    /// A validated success envelope could not be projected through the command-specific parser.
    InvalidResultProjection,
    /// Exact command-response correlation failed without consuming unrelated state.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// The remote end returned a correlatable WebDriver BiDi protocol error for this command.
    RemoteProtocolError {
        /// Exact local command identifier consumed by the protocol-error response.
        command_id: u64,
    },
}

impl fmt::Display for WebDriverBiDiSessionStatusResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi session.status envelope is invalid")
            }
            Self::MissingReady => {
                formatter.write_str("WebDriver BiDi session.status result is missing ready")
            }
            Self::InvalidReady => {
                formatter.write_str("WebDriver BiDi session.status result ready is invalid")
            }
            Self::MissingMessage => {
                formatter.write_str("WebDriver BiDi session.status result is missing message")
            }
            Self::InvalidMessage => {
                formatter.write_str("WebDriver BiDi session.status result message is invalid")
            }
            Self::DuplicateResultMember { .. } => formatter
                .write_str("WebDriver BiDi session.status result contains a duplicate member"),
            Self::MessageTooLarge { .. } => formatter
                .write_str("WebDriver BiDi session.status result message exceeds the size bound"),
            Self::InvalidResultProjection => {
                formatter.write_str("WebDriver BiDi session.status result projection is invalid")
            }
            Self::Correlation { .. } => {
                formatter.write_str("WebDriver BiDi session.status response correlation failed")
            }
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi session.status returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiSessionStatusResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::MissingReady
            | Self::InvalidReady
            | Self::MissingMessage
            | Self::InvalidMessage
            | Self::DuplicateResultMember { .. }
            | Self::MessageTooLarge { .. }
            | Self::InvalidResultProjection
            | Self::RemoteProtocolError { .. } => None,
        }
    }
}

struct StatusProjection {
    ready: bool,
    message: String,
}

impl StatusProjection {
    fn parse(text: &str) -> Result<Self, WebDriverBiDiSessionStatusResponseError> {
        let mut cursor = ProjectionCursor::new(text);
        cursor.skip_whitespace();
        if !cursor.consume_byte(b'{') {
            return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
        }
        cursor.skip_whitespace();
        if cursor.consume_byte(b'}') {
            return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
        }

        loop {
            cursor.skip_whitespace();
            let key = cursor
                .parse_string()
                .ok_or(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection)?;
            cursor.skip_whitespace();
            if !cursor.consume_byte(b':') {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
            cursor.skip_whitespace();
            if key == "result" {
                return cursor.parse_result_object();
            }
            if !cursor.skip_value() {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
            cursor.skip_whitespace();
            if cursor.consume_byte(b'}') {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
            if !cursor.consume_byte(b',') {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
        }
    }
}

struct ProjectionCursor<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> ProjectionCursor<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn current_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.index).copied()
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.current_byte() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current_byte(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }

    fn parse_result_object(
        &mut self,
    ) -> Result<StatusProjection, WebDriverBiDiSessionStatusResponseError> {
        if !self.consume_byte(b'{') {
            return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
        }
        self.skip_whitespace();
        let mut ready = None;
        let mut message = None;
        if self.consume_byte(b'}') {
            return Err(WebDriverBiDiSessionStatusResponseError::MissingReady);
        }

        loop {
            self.skip_whitespace();
            let key = self
                .parse_string()
                .ok_or(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection)?;
            self.skip_whitespace();
            if !self.consume_byte(b':') {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
            self.skip_whitespace();
            match key.as_str() {
                "ready" => {
                    if ready.is_some() {
                        return Err(
                            WebDriverBiDiSessionStatusResponseError::DuplicateResultMember {
                                member: "ready",
                            },
                        );
                    }
                    ready = Some(self.parse_ready()?);
                }
                "message" => {
                    if message.is_some() {
                        return Err(
                            WebDriverBiDiSessionStatusResponseError::DuplicateResultMember {
                                member: "message",
                            },
                        );
                    }
                    let parsed = self
                        .parse_string()
                        .ok_or(WebDriverBiDiSessionStatusResponseError::InvalidMessage)?;
                    if parsed.len() > MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE {
                        return Err(WebDriverBiDiSessionStatusResponseError::MessageTooLarge {
                            maximum_bytes: MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE,
                        });
                    }
                    message = Some(parsed);
                }
                _ => {
                    if !self.skip_value() {
                        return Err(
                            WebDriverBiDiSessionStatusResponseError::InvalidResultProjection,
                        );
                    }
                }
            }
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                break;
            }
            if !self.consume_byte(b',') {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
        }

        Ok(StatusProjection {
            ready: ready.ok_or(WebDriverBiDiSessionStatusResponseError::MissingReady)?,
            message: message.ok_or(WebDriverBiDiSessionStatusResponseError::MissingMessage)?,
        })
    }

    fn parse_ready(&mut self) -> Result<bool, WebDriverBiDiSessionStatusResponseError> {
        if self.consume_literal(b"true") {
            Ok(true)
        } else if self.consume_literal(b"false") {
            Ok(false)
        } else {
            if !self.skip_value() {
                return Err(WebDriverBiDiSessionStatusResponseError::InvalidResultProjection);
            }
            Err(WebDriverBiDiSessionStatusResponseError::InvalidReady)
        }
    }

    fn consume_literal(&mut self, literal: &[u8]) -> bool {
        let end = self.index.saturating_add(literal.len());
        if self.input.as_bytes().get(self.index..end) == Some(literal) {
            self.index = end;
            true
        } else {
            false
        }
    }

    fn skip_value(&mut self) -> bool {
        self.skip_whitespace();
        match self.current_byte() {
            Some(b'"') => self.parse_string().is_some(),
            Some(b'{') => self.skip_object(),
            Some(b'[') => self.skip_array(),
            Some(b't') => self.consume_literal(b"true"),
            Some(b'f') => self.consume_literal(b"false"),
            Some(b'n') => self.consume_literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.skip_number(),
            _ => false,
        }
    }

    fn skip_object(&mut self) -> bool {
        if !self.consume_byte(b'{') {
            return false;
        }
        self.skip_whitespace();
        if self.consume_byte(b'}') {
            return true;
        }
        loop {
            self.skip_whitespace();
            if self.parse_string().is_none() {
                return false;
            }
            self.skip_whitespace();
            if !self.consume_byte(b':') {
                return false;
            }
            if !self.skip_value() {
                return false;
            }
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                return true;
            }
            if !self.consume_byte(b',') {
                return false;
            }
        }
    }

    fn skip_array(&mut self) -> bool {
        if !self.consume_byte(b'[') {
            return false;
        }
        self.skip_whitespace();
        if self.consume_byte(b']') {
            return true;
        }
        loop {
            if !self.skip_value() {
                return false;
            }
            self.skip_whitespace();
            if self.consume_byte(b']') {
                return true;
            }
            if !self.consume_byte(b',') {
                return false;
            }
        }
    }

    fn skip_number(&mut self) -> bool {
        let start = self.index;
        while matches!(
            self.current_byte(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.index += 1;
        }
        self.index > start
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.consume_byte(b'"') {
            return None;
        }
        let mut output = String::new();
        loop {
            let byte = self.current_byte()?;
            match byte {
                b'"' => {
                    self.index += 1;
                    return Some(output);
                }
                b'\\' => {
                    self.index += 1;
                    if !self.parse_escape(&mut output) {
                        return None;
                    }
                }
                0x00..=0x1f => return None,
                _ if byte.is_ascii() => {
                    output.push(char::from(byte));
                    self.index += 1;
                }
                _ => {
                    let width = byte.leading_ones() as usize;
                    let end = self.index.checked_add(width)?;
                    let character = self.input.get(self.index..end)?;
                    output.push_str(character);
                    self.index = end;
                }
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> bool {
        let Some(escape) = self.current_byte() else {
            return false;
        };
        self.index += 1;
        match escape {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{0008}'),
            b'f' => output.push('\u{000c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => return self.parse_unicode_escape(output),
            _ => return false,
        }
        true
    }

    fn parse_unicode_escape(&mut self, output: &mut String) -> bool {
        let Some(first) = self.parse_hex_u16() else {
            return false;
        };
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if !self.consume_byte(b'\\') || !self.consume_byte(b'u') {
                return false;
            }
            let Some(second) = self.parse_hex_u16() else {
                return false;
            };
            if !(0xdc00..=0xdfff).contains(&second) {
                return false;
            }
            0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            return false;
        } else {
            u32::from(first)
        };
        let Some(character) = char::from_u32(scalar) else {
            return false;
        };
        output.push(character);
        true
    }

    fn parse_hex_u16(&mut self) -> Option<u16> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self.current_byte()?;
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return None,
            };
            value = (value << 4) | digit;
            self.index += 1;
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_accepts_required_fields_unknown_metadata_and_escaped_keys() {
        let projected = StatusProjection::parse(
            r#"{"meta":[null,true,false,1,-2.5e+3,{"nested":"value"}],"re\u0073ult":{"message":"re\u0061dy \ud83d\ude80","extra":{},"ready":false}}"#,
        );
        assert!(projected.is_ok());
        let projected = projected.ok();
        assert_eq!(projected.as_ref().map(|value| value.ready), Some(false));
        assert_eq!(
            projected.as_ref().map(|value| value.message.as_str()),
            Some("ready 🚀")
        );
    }

    #[test]
    fn projection_rejects_missing_invalid_duplicate_and_oversized_required_fields() {
        let cases = [
            (r#"{"result":{"message":"x"}}"#.to_owned(), "missing ready"),
            (r#"{"result":{"ready":0,"message":"x"}}"#.to_owned(), "invalid ready"),
            (r#"{"result":{"ready":true}}"#.to_owned(), "missing message"),
            (r#"{"result":{"ready":true,"message":false}}"#.to_owned(), "invalid message"),
            (
                r#"{"result":{"ready":true,"ready":false,"message":"x"}}"#.to_owned(),
                "duplicate ready",
            ),
            (
                r#"{"result":{"ready":true,"message":"x","message":"y"}}"#.to_owned(),
                "duplicate message",
            ),
            (
                format!(
                    "{{\"result\":{{\"ready\":true,\"message\":\"{}\"}}}}",
                    "x".repeat(MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE + 1)
                ),
                "oversized message",
            ),
        ];

        for (document, label) in cases {
            assert!(StatusProjection::parse(&document).is_err(), "{label}");
        }
    }

    #[test]
    fn projection_cursor_rejects_malformed_private_inputs_without_panicking() {
        let malformed = [
            "",
            "[]",
            "{}",
            r#"{"x":}"#,
            r#"{"x":1}"#,
            r#"{"result":[]}"#,
            r#"{"result":{"ready":true,"message":"\uD800"}}"#,
            r#"{"result":{"ready":true,"message":"\q"}}"#,
        ];
        for document in malformed {
            assert!(StatusProjection::parse(document).is_err());
        }
    }

    #[test]
    fn response_errors_have_stable_redacted_messages_and_sources() {
        let envelope = WebDriverBiDiSessionStatusResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert!(envelope.source().is_some());
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi session.status envelope is invalid"
        );

        let correlation = WebDriverBiDiSessionStatusResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert!(correlation.source().is_some());
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.status response correlation failed"
        );

        let leaf_errors = [
            WebDriverBiDiSessionStatusResponseError::MissingReady,
            WebDriverBiDiSessionStatusResponseError::InvalidReady,
            WebDriverBiDiSessionStatusResponseError::MissingMessage,
            WebDriverBiDiSessionStatusResponseError::InvalidMessage,
            WebDriverBiDiSessionStatusResponseError::DuplicateResultMember { member: "ready" },
            WebDriverBiDiSessionStatusResponseError::MessageTooLarge {
                maximum_bytes: MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE,
            },
            WebDriverBiDiSessionStatusResponseError::InvalidResultProjection,
            WebDriverBiDiSessionStatusResponseError::RemoteProtocolError { command_id: 7 },
        ];
        for error in leaf_errors {
            assert!(error.source().is_none());
            assert!(!error.to_string().is_empty());
        }
    }
}
