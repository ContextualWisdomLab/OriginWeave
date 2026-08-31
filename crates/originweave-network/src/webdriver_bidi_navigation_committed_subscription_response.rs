use std::{error::Error, fmt};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError, WebDriverBiDiJsonEnvelopeKind,
    WebDriverBiDiWebSocketTextMessage,
};

/// Maximum decoded UTF-8 bytes retained from a WebDriver BiDi `session.Subscription` identifier.
///
/// WebDriver BiDi defines the identifier as opaque text without a protocol size ceiling. OriginWeave
/// therefore applies a reviewed local retention bound while preserving the identifier byte-for-byte
/// for later typed subscription lifecycle work. The value is never included in `Debug` output.
pub const MAX_WEBDRIVER_BIDI_SUBSCRIPTION_IDENTIFIER_BYTES: usize = 4_096;

/// Typed, correlated successful result of one context-scoped WebDriver BiDi `session.subscribe`.
///
/// This value retains only the exact correlated command id and the bounded opaque subscription
/// identifier returned by the remote end. It does not expose a generic JSON result, grant event,
/// browser, policy, origin, secret, or Agent authority, or prove that any subscribed event has fired.
#[derive(Eq, PartialEq)]
pub struct WebDriverBiDiNavigationCommittedSubscriptionResult {
    command_id: u64,
    subscription_id: String,
}

impl fmt::Debug for WebDriverBiDiNavigationCommittedSubscriptionResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiNavigationCommittedSubscriptionResult")
            .field("command_id", &self.command_id)
            .field("subscription_id_len", &self.subscription_id.len())
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedSubscriptionResult {
    /// Parse one bounded local-end message and consume its exact outstanding command on success.
    ///
    /// Common WebDriver BiDi envelope validation runs first. A successful envelope then undergoes
    /// command-specific projection of the required `result.subscription` text before correlation is
    /// consumed, so malformed or ambiguous success bodies cannot silently retire a command id. A
    /// correlatable protocol-error response consumes its matching id and returns a typed remote
    /// failure retaining only the protocol error code. Events, null-id errors, malformed envelopes,
    /// and unknown ids fail closed without consuming unrelated outstanding correlation state.
    pub fn parse_and_correlate(
        message: &WebDriverBiDiWebSocketTextMessage,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedSubscriptionResponseError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message).map_err(|source| {
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::Envelope { source }
        })?;

        match envelope.kind() {
            WebDriverBiDiJsonEnvelopeKind::Success => {
                let projected = SubscriptionProjection::parse(message.as_str())?;
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| {
                        WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                            source,
                        }
                    })?;
                Ok(Self {
                    command_id: completed.command_id(),
                    subscription_id: projected.subscription_id,
                })
            }
            WebDriverBiDiJsonEnvelopeKind::Error => {
                let error_code = retain_validated_error_code(envelope.error_code())?;
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| {
                        WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                            source,
                        }
                    })?;
                Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::RemoteProtocolError {
                        command_id: completed.command_id(),
                        error_code,
                    },
                )
            }
            WebDriverBiDiJsonEnvelopeKind::Event => Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
                    source: WebDriverBiDiCommandCorrelationError::EventIsNotResponse,
                },
            ),
        }
    }

    /// Return the exact local command identifier consumed by this result.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Borrow the bounded opaque subscription identifier returned by the remote end.
    #[must_use]
    pub fn subscription_id(&self) -> &str {
        &self.subscription_id
    }
}

/// Fail-closed failures while admitting one typed WebDriver BiDi `session.subscribe` response.
#[derive(Debug, Eq, PartialEq)]
pub enum WebDriverBiDiNavigationCommittedSubscriptionResponseError {
    /// Common local-end JSON envelope validation failed.
    Envelope {
        /// Exact common-envelope validation failure.
        source: WebDriverBiDiJsonEnvelopeError,
    },
    /// The successful result object omits the required `subscription` member.
    MissingSubscription,
    /// The successful result object's `subscription` member is not JSON text.
    InvalidSubscription,
    /// The successful result repeats the `subscription` member and is ambiguous.
    DuplicateSubscription,
    /// The decoded subscription identifier exceeds the reviewed local retention bound.
    SubscriptionTooLarge {
        /// Maximum decoded identifier length admitted in bytes.
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
        /// Protocol error code retained from the already validated common envelope.
        error_code: String,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedSubscriptionResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => {
                formatter.write_str("WebDriver BiDi session.subscribe envelope is invalid")
            }
            Self::MissingSubscription => formatter
                .write_str("WebDriver BiDi session.subscribe result is missing subscription"),
            Self::InvalidSubscription => formatter
                .write_str("WebDriver BiDi session.subscribe result subscription is invalid"),
            Self::DuplicateSubscription => formatter.write_str(
                "WebDriver BiDi session.subscribe result contains duplicate subscription",
            ),
            Self::SubscriptionTooLarge { .. } => formatter.write_str(
                "WebDriver BiDi session.subscribe result subscription exceeds the size bound",
            ),
            Self::InvalidResultProjection => {
                formatter.write_str("WebDriver BiDi session.subscribe result projection is invalid")
            }
            Self::Correlation { .. } => {
                formatter.write_str("WebDriver BiDi session.subscribe response correlation failed")
            }
            Self::RemoteProtocolError { .. } => {
                formatter.write_str("WebDriver BiDi session.subscribe returned a protocol error")
            }
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedSubscriptionResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::MissingSubscription
            | Self::InvalidSubscription
            | Self::DuplicateSubscription
            | Self::SubscriptionTooLarge { .. }
            | Self::InvalidResultProjection
            | Self::RemoteProtocolError { .. } => None,
        }
    }
}

fn retain_validated_error_code(
    error_code: Option<&str>,
) -> Result<String, WebDriverBiDiNavigationCommittedSubscriptionResponseError> {
    error_code.map(str::to_owned).ok_or(
        WebDriverBiDiNavigationCommittedSubscriptionResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "error" },
        },
    )
}

struct SubscriptionProjection {
    subscription_id: String,
}

impl SubscriptionProjection {
    fn parse(
        text: &str,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedSubscriptionResponseError> {
        let mut cursor = ProjectionCursor::new(text);
        cursor.skip_whitespace();
        if !cursor.consume_byte(b'{') {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            );
        }
        cursor.skip_whitespace();
        if cursor.consume_byte(b'}') {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            );
        }

        loop {
            cursor.skip_whitespace();
            let key = cursor.parse_string().ok_or(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            )?;
            cursor.skip_whitespace();
            if !cursor.consume_byte(b':') {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
            cursor.skip_whitespace();
            if key == "result" {
                return cursor.parse_result_object();
            }
            if !cursor.skip_value() {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
            cursor.skip_whitespace();
            if cursor.consume_byte(b'}') {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
            if !cursor.consume_byte(b',') {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
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
    ) -> Result<SubscriptionProjection, WebDriverBiDiNavigationCommittedSubscriptionResponseError>
    {
        if !self.consume_byte(b'{') {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            );
        }
        self.skip_whitespace();
        let mut subscription_id = None;
        if self.consume_byte(b'}') {
            return Err(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::MissingSubscription,
            );
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string().ok_or(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            )?;
            self.skip_whitespace();
            if !self.consume_byte(b':') {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
            self.skip_whitespace();
            if key == "subscription" {
                if subscription_id.is_some() {
                    return Err(
                        WebDriverBiDiNavigationCommittedSubscriptionResponseError::DuplicateSubscription,
                    );
                }
                let parsed = self.parse_string().ok_or(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidSubscription,
                )?;
                if parsed.len() > MAX_WEBDRIVER_BIDI_SUBSCRIPTION_IDENTIFIER_BYTES {
                    return Err(
                        WebDriverBiDiNavigationCommittedSubscriptionResponseError::SubscriptionTooLarge {
                            maximum_bytes: MAX_WEBDRIVER_BIDI_SUBSCRIPTION_IDENTIFIER_BYTES,
                        },
                    );
                }
                subscription_id = Some(parsed);
            } else if !self.skip_value() {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                break;
            }
            if !self.consume_byte(b',') {
                return Err(
                    WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
                );
            }
        }

        Ok(SubscriptionProjection {
            subscription_id: subscription_id.ok_or(
                WebDriverBiDiNavigationCommittedSubscriptionResponseError::MissingSubscription,
            )?,
        })
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
                    let end = self.index + width;
                    output.push_str(&self.input[self.index..end]);
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
        if (0xd800..=0xdbff).contains(&first) {
            if !self.consume_byte(b'\\') || !self.consume_byte(b'u') {
                return false;
            }
            let Some(second) = self.parse_hex_u16() else {
                return false;
            };
            if !(0xdc00..=0xdfff).contains(&second) {
                return false;
            }
            output.push_str(&String::from_utf16_lossy(&[first, second]));
            true
        } else if (0xdc00..=0xdfff).contains(&first) {
            false
        } else {
            output.push_str(&String::from_utf16_lossy(&[first]));
            true
        }
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
    fn projection_accepts_subscription_unknown_metadata_and_escaped_keys() {
        let projected = SubscriptionProjection::parse(
            r#"{"meta":[null,true,false,1,-2.5e+3,{"nested":"value"}],"re\u0073ult":{"extra":{},"sub\u0073cription":"sub-\ud83d\ude80"}}"#,
        );
        assert!(projected.is_ok());
        assert_eq!(
            projected.ok().map(|value| value.subscription_id),
            Some("sub-🚀".to_owned())
        );

        let direct_utf8 = SubscriptionProjection::parse(
            "\n\t { \r\n \"result\" : { \"subscription\" : \"구독-a\" } }",
        );
        assert_eq!(
            direct_utf8.ok().map(|value| value.subscription_id),
            Some("구독-a".to_owned())
        );
    }

    #[test]
    fn projection_rejects_missing_invalid_duplicate_and_oversized_subscription() {
        let cases = [
            (r#"{"result":{}}"#.to_owned(), "missing"),
            (r#"{"result":{"subscription":false}}"#.to_owned(), "invalid"),
            (
                r#"{"result":{"subscription":"a","subscription":"b"}}"#.to_owned(),
                "duplicate",
            ),
            (
                format!(
                    "{{\"result\":{{\"subscription\":\"{}\"}}}}",
                    "x".repeat(MAX_WEBDRIVER_BIDI_SUBSCRIPTION_IDENTIFIER_BYTES + 1)
                ),
                "oversized",
            ),
            (r#"{"x":1}"#.to_owned(), "missing result"),
        ];

        for (document, label) in cases {
            assert!(SubscriptionProjection::parse(&document).is_err(), "{label}");
        }
    }

    #[test]
    fn projection_cursor_rejects_malformed_private_inputs_without_panicking() {
        let malformed = [
            "",
            "[]",
            "{}",
            r#"{"x":}"#,
            r#"{"x" 1}"#,
            r#"{"x":1 ?}"#,
            r#"{?}"#,
            r#"{"result":[]}"#,
            r#"{"result":{?}}"#,
            r#"{"result":{"subscription" "x"}}"#,
            r#"{"result":{"subscription":"x" "extra":1}}"#,
            r#"{"result":{"subscription":"x","extra":?}}"#,
            r#"{"result":{"subscription":"\uD800"}}"#,
            r#"{"result":{"subscription":"\q"}}"#,
        ];
        for document in malformed {
            assert!(
                SubscriptionProjection::parse(document).is_err(),
                "{document}"
            );
        }
    }

    #[test]
    fn projection_cursor_defensive_helpers_cover_hostile_dispatch_edges() {
        let mut object = ProjectionCursor::new("[]");
        assert!(!object.skip_object());
        let mut object = ProjectionCursor::new("{}");
        assert!(object.skip_object());
        let mut object = ProjectionCursor::new("{?}");
        assert!(!object.skip_object());
        let mut object = ProjectionCursor::new(r#"{"x" 1}"#);
        assert!(!object.skip_object());
        let mut object = ProjectionCursor::new(r#"{"x":?}"#);
        assert!(!object.skip_object());
        let mut object = ProjectionCursor::new(r#"{"x":1 ?}"#);
        assert!(!object.skip_object());
        let mut object = ProjectionCursor::new(r#"{"x":1,"y":2}"#);
        assert!(object.skip_object());

        let mut array = ProjectionCursor::new("{}");
        assert!(!array.skip_array());
        let mut array = ProjectionCursor::new("[]");
        assert!(array.skip_array());
        let mut array = ProjectionCursor::new("[?]");
        assert!(!array.skip_array());
        let mut array = ProjectionCursor::new("[1 ?]");
        assert!(!array.skip_array());

        for document in [r#""x""#, "{}", "[]", "true", "false", "null", "-2.5e+3"] {
            let mut value = ProjectionCursor::new(document);
            assert!(value.skip_value(), "{document}");
        }
        let mut value = ProjectionCursor::new("?");
        assert!(!value.skip_value());

        let mut number = ProjectionCursor::new("x");
        assert!(!number.skip_number());
        let mut number = ProjectionCursor::new("+1");
        assert!(number.skip_number());

        let mut string = ProjectionCursor::new("x");
        assert!(string.parse_string().is_none());
        let mut string = ProjectionCursor::new("\"unterminated");
        assert!(string.parse_string().is_none());
        let mut string = ProjectionCursor::new("\"\u{0001}\"");
        assert!(string.parse_string().is_none());
        let mut string = ProjectionCursor::new("\"é\"");
        assert_eq!(string.parse_string().as_deref(), Some("é"));

        let mut output = String::new();
        let mut escape = ProjectionCursor::new("");
        assert!(!escape.parse_escape(&mut output));
        for sequence in ["\"", "\\", "/", "b", "f", "n", "r", "t"] {
            let mut output = String::new();
            let mut escape = ProjectionCursor::new(sequence);
            assert!(escape.parse_escape(&mut output), "{sequence:?}");
        }
        let mut output = String::new();
        let mut escape = ProjectionCursor::new("q");
        assert!(!escape.parse_escape(&mut output));

        for sequence in ["0000", "aBcD", "Ff09"] {
            let mut hex = ProjectionCursor::new(sequence);
            assert!(hex.parse_hex_u16().is_some());
        }
        let mut hex = ProjectionCursor::new("xyz1");
        assert!(hex.parse_hex_u16().is_none());
        let mut hex = ProjectionCursor::new("0");
        assert!(hex.parse_hex_u16().is_none());

        let unicode_cases = [
            ("0041", true),
            ("d83d\\ude80", true),
            ("d83d", false),
            ("d83d\\u0041", false),
            ("dc00", false),
            ("zzzz", false),
        ];
        for (sequence, expected) in unicode_cases {
            let mut output = String::new();
            let mut unicode = ProjectionCursor::new(sequence);
            assert_eq!(unicode.parse_unicode_escape(&mut output), expected);
        }
    }

    #[test]
    fn response_errors_have_stable_messages_and_typed_sources() {
        let envelope = WebDriverBiDiNavigationCommittedSubscriptionResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(
            envelope.to_string(),
            "WebDriver BiDi session.subscribe envelope is invalid"
        );
        assert!(envelope.source().is_some());

        let correlation = WebDriverBiDiNavigationCommittedSubscriptionResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(
            correlation.to_string(),
            "WebDriver BiDi session.subscribe response correlation failed"
        );
        assert!(correlation.source().is_some());

        let source_free = [
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::MissingSubscription,
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidSubscription,
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::DuplicateSubscription,
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::SubscriptionTooLarge {
                maximum_bytes: MAX_WEBDRIVER_BIDI_SUBSCRIPTION_IDENTIFIER_BYTES,
            },
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::InvalidResultProjection,
            WebDriverBiDiNavigationCommittedSubscriptionResponseError::RemoteProtocolError {
                command_id: 7,
                error_code: "invalid argument".to_owned(),
            },
        ];
        for error in source_free {
            assert!(!error.to_string().is_empty());
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn result_debug_redacts_opaque_subscription_identifier() {
        let result = WebDriverBiDiNavigationCommittedSubscriptionResult {
            command_id: 7,
            subscription_id: "sensitive-subscription".to_owned(),
        };
        let debug = format!("{result:?}");
        assert!(debug.contains("command_id"));
        assert!(debug.contains("subscription_id_len"));
        assert!(!debug.contains("sensitive-subscription"));
        assert_eq!(result.command_id(), 7);
        assert_eq!(result.subscription_id(), "sensitive-subscription");
    }

    #[test]
    fn retain_error_code_fails_closed_when_common_invariant_is_absent() {
        assert_eq!(
            retain_validated_error_code(Some("invalid argument")).as_deref(),
            Ok("invalid argument")
        );
        assert!(retain_validated_error_code(None).is_err());
    }
}
