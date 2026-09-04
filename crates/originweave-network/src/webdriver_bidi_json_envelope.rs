use std::{collections::HashSet, error::Error, fmt};

use crate::WebDriverBiDiWebSocketTextMessage;

/// Largest nesting depth accepted while validating one BiDi JSON envelope.
///
/// The WebSocket text-message boundary already caps the aggregate document at 1 MiB. This
/// independent depth budget prevents a syntactically valid but pathologically nested document
/// from exhausting the Rust call stack while no browser or Agent authority has been granted.
pub const MAX_WEBDRIVER_BIDI_JSON_DEPTH: usize = 64;

/// Largest integer admitted by WebDriver BiDi's `js-uint` production.
pub const MAX_WEBDRIVER_BIDI_JS_UINT: u64 = 9_007_199_254_740_991;

const WEBDRIVER_BIDI_ERROR_CODES: [&str; 30] = [
    "invalid argument",
    "invalid selector",
    "invalid session id",
    "invalid web extension",
    "move target out of bounds",
    "no such alert",
    "no such network collector",
    "no such element",
    "no such frame",
    "no such handle",
    "no such history entry",
    "no such intercept",
    "no such network data",
    "no such node",
    "no such request",
    "no such screencast",
    "no such script",
    "no such storage partition",
    "no such user context",
    "no such web extension",
    "session not created",
    "unable to capture screen",
    "unable to close browser",
    "unable to set cookie",
    "unable to set file input",
    "unavailable network data",
    "underspecified storage partition",
    "unknown command",
    "unknown error",
    "unsupported operation",
];

/// Local-end WebDriver BiDi envelope kind after complete JSON syntax validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiJsonEnvelopeKind {
    /// A command completed successfully.
    Success,
    /// A command completed with a protocol error.
    Error,
    /// The remote end emitted an event.
    Event,
}

/// Credential-minimal classification of one complete WebDriver BiDi local-end JSON envelope.
///
/// Result and parameter bodies are deliberately validated and discarded at this boundary. They
/// remain untrusted protocol data for later command- or event-specific parsers and are not exposed
/// as generic JSON values that could become ambient browser or Agent authority.
#[derive(Eq, PartialEq)]
pub struct WebDriverBiDiJsonEnvelope {
    kind: WebDriverBiDiJsonEnvelopeKind,
    command_id: Option<u64>,
    method: Option<String>,
    error_code: Option<String>,
}

impl fmt::Debug for WebDriverBiDiJsonEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiJsonEnvelope")
            .field("kind", &self.kind)
            .field("command_id", &self.command_id)
            .field("has_method", &self.method.is_some())
            .field("has_error_code", &self.error_code.is_some())
            .finish()
    }
}

impl WebDriverBiDiJsonEnvelope {
    /// Parse and classify one already bounded, validated UTF-8 WebSocket text message.
    ///
    /// This validates the complete RFC 8259 JSON grammar, rejects duplicate top-level member
    /// names and excessive nesting, then enforces only the common local-end envelope shape from
    /// WebDriver BiDi. Extensible result/parameter bodies remain opaque and are discarded.
    pub fn parse(
        message: &WebDriverBiDiWebSocketTextMessage,
    ) -> Result<Self, WebDriverBiDiJsonEnvelopeError> {
        Self::parse_str(message.as_str())
    }

    fn parse_str(text: &str) -> Result<Self, WebDriverBiDiJsonEnvelopeError> {
        let mut cursor = JsonCursor::new(text);
        let fields = cursor.parse_top_level_object()?;
        cursor.skip_whitespace();
        if !cursor.is_finished() {
            return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
        }
        fields.into_envelope()
    }

    /// Return the classified local-end envelope kind.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiJsonEnvelopeKind {
        self.kind
    }

    /// Return the command identifier for success and correlatable error responses.
    ///
    /// Events and error responses whose protocol `id` is `null` return `None`.
    #[must_use]
    pub const fn command_id(&self) -> Option<u64> {
        self.command_id
    }

    /// Borrow the event method when this is an event envelope.
    #[must_use]
    pub fn method(&self) -> Option<&str> {
        self.method.as_deref()
    }

    /// Borrow the protocol error code when this is an error envelope.
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }
}

/// Fail-closed JSON syntax and common-envelope failures for local-end WebDriver BiDi messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiJsonEnvelopeError {
    /// The document violates RFC 8259 JSON syntax or contains trailing non-whitespace bytes.
    InvalidJson,
    /// The top-level JSON value is not an object.
    RootMustBeObject,
    /// A top-level object member name appears more than once.
    DuplicateTopLevelMember,
    /// JSON nesting exceeded the reviewed parser safety budget.
    NestingTooDeep {
        /// Maximum nesting depth admitted by this parser.
        maximum_depth: usize,
    },
    /// A required common-envelope member is absent.
    MissingRequiredMember {
        /// Stable non-sensitive member name.
        member: &'static str,
    },
    /// A common-envelope member has the wrong JSON type or value range.
    InvalidMember {
        /// Stable non-sensitive member name.
        member: &'static str,
    },
    /// The `type` discriminator is not one of the three local-end envelope kinds.
    UnsupportedEnvelopeType,
}

impl fmt::Display for WebDriverBiDiJsonEnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => formatter.write_str("invalid WebDriver BiDi JSON document"),
            Self::RootMustBeObject => {
                formatter.write_str("WebDriver BiDi local-end message must be a JSON object")
            }
            Self::DuplicateTopLevelMember => formatter
                .write_str("WebDriver BiDi JSON object contains a duplicate top-level member"),
            Self::NestingTooDeep { maximum_depth } => write!(
                formatter,
                "WebDriver BiDi JSON nesting exceeds maximum depth {maximum_depth}"
            ),
            Self::MissingRequiredMember { member } => {
                write!(
                    formatter,
                    "WebDriver BiDi envelope is missing required member {member}"
                )
            }
            Self::InvalidMember { member } => {
                write!(
                    formatter,
                    "WebDriver BiDi envelope member {member} is invalid"
                )
            }
            Self::UnsupportedEnvelopeType => {
                formatter.write_str("unsupported WebDriver BiDi local-end envelope type")
            }
        }
    }
}

impl Error for WebDriverBiDiJsonEnvelopeError {}

#[derive(Default)]
struct TopLevelFields {
    envelope_type: Option<JsonValue>,
    id: Option<JsonValue>,
    result: Option<JsonValue>,
    method: Option<JsonValue>,
    params: Option<JsonValue>,
    error: Option<JsonValue>,
    message: Option<JsonValue>,
    stacktrace: Option<JsonValue>,
}

impl TopLevelFields {
    fn record(&mut self, key: &str, value: JsonValue) {
        match key {
            "type" => self.envelope_type = Some(value),
            "id" => self.id = Some(value),
            "result" => self.result = Some(value),
            "method" => self.method = Some(value),
            "params" => self.params = Some(value),
            "error" => self.error = Some(value),
            "message" => self.message = Some(value),
            "stacktrace" => self.stacktrace = Some(value),
            _ => {}
        }
    }

    fn into_envelope(self) -> Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError> {
        let kind = match self.envelope_type.as_ref() {
            None => return Err(missing("type")),
            Some(JsonValue::Text(envelope_type)) => match envelope_type.as_str() {
                "success" => WebDriverBiDiJsonEnvelopeKind::Success,
                "error" => WebDriverBiDiJsonEnvelopeKind::Error,
                "event" => WebDriverBiDiJsonEnvelopeKind::Event,
                _ => return Err(WebDriverBiDiJsonEnvelopeError::UnsupportedEnvelopeType),
            },
            Some(_) => return Err(invalid("type")),
        };
        match kind {
            WebDriverBiDiJsonEnvelopeKind::Success => self.into_success(),
            WebDriverBiDiJsonEnvelopeKind::Error => self.into_error(),
            WebDriverBiDiJsonEnvelopeKind::Event => self.into_event(),
        }
    }

    fn into_success(self) -> Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError> {
        let command_id = required_js_uint(self.id, "id")?;
        require_object(self.result, "result")?;
        Ok(WebDriverBiDiJsonEnvelope {
            kind: WebDriverBiDiJsonEnvelopeKind::Success,
            command_id: Some(command_id),
            method: None,
            error_code: None,
        })
    }

    fn into_error(self) -> Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError> {
        let command_id = nullable_js_uint(self.id, "id")?;
        let error_code = required_text(self.error, "error")?;
        let _message = required_text(self.message, "message")?;
        if let Some(stacktrace) = self.stacktrace {
            require_text_value(stacktrace, "stacktrace")?;
        }
        if !is_webdriver_bidi_error_code(&error_code) {
            return Err(invalid("error"));
        }
        Ok(WebDriverBiDiJsonEnvelope {
            kind: WebDriverBiDiJsonEnvelopeKind::Error,
            command_id,
            method: None,
            error_code: Some(error_code),
        })
    }

    fn into_event(self) -> Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError> {
        let method = required_text(self.method, "method")?;
        require_object(self.params, "params")?;
        if !matches!(
            method.split_once('.'),
            Some((module, event)) if !module.is_empty() && !event.is_empty()
        ) {
            return Err(invalid("method"));
        }
        Ok(WebDriverBiDiJsonEnvelope {
            kind: WebDriverBiDiJsonEnvelopeKind::Event,
            command_id: None,
            method: Some(method),
            error_code: None,
        })
    }
}

fn missing(member: &'static str) -> WebDriverBiDiJsonEnvelopeError {
    WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member }
}

fn invalid(member: &'static str) -> WebDriverBiDiJsonEnvelopeError {
    WebDriverBiDiJsonEnvelopeError::InvalidMember { member }
}

fn is_webdriver_bidi_error_code(value: &str) -> bool {
    WEBDRIVER_BIDI_ERROR_CODES.contains(&value)
}

fn required_text(
    value: Option<JsonValue>,
    member: &'static str,
) -> Result<String, WebDriverBiDiJsonEnvelopeError> {
    let value = value.ok_or_else(|| missing(member))?;
    match value {
        JsonValue::Text(text) => Ok(text),
        _ => Err(invalid(member)),
    }
}

fn require_text_value(
    value: JsonValue,
    member: &'static str,
) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
    if matches!(value, JsonValue::Text(_)) {
        Ok(())
    } else {
        Err(invalid(member))
    }
}

fn required_js_uint(
    value: Option<JsonValue>,
    member: &'static str,
) -> Result<u64, WebDriverBiDiJsonEnvelopeError> {
    let value = value.ok_or_else(|| missing(member))?;
    match value {
        JsonValue::Number(Some(number)) => Ok(number),
        _ => Err(invalid(member)),
    }
}

fn nullable_js_uint(
    value: Option<JsonValue>,
    member: &'static str,
) -> Result<Option<u64>, WebDriverBiDiJsonEnvelopeError> {
    let value = value.ok_or_else(|| missing(member))?;
    match value {
        JsonValue::Null => Ok(None),
        JsonValue::Number(Some(number)) => Ok(Some(number)),
        _ => Err(invalid(member)),
    }
}

fn require_object(
    value: Option<JsonValue>,
    member: &'static str,
) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
    let value = value.ok_or_else(|| missing(member))?;
    if matches!(value, JsonValue::Object) {
        Ok(())
    } else {
        Err(invalid(member))
    }
}

enum JsonValue {
    Null,
    Text(String),
    Number(Option<u64>),
    Object,
    Other,
}

struct JsonCursor<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> JsonCursor<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn is_finished(&self) -> bool {
        self.index == self.input.len()
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

    fn expect_byte(&mut self, expected: u8) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(WebDriverBiDiJsonEnvelopeError::InvalidJson)
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current_byte(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }

    fn parse_top_level_object(&mut self) -> Result<TopLevelFields, WebDriverBiDiJsonEnvelopeError> {
        self.skip_whitespace();
        if !self.consume_byte(b'{') {
            return Err(WebDriverBiDiJsonEnvelopeError::RootMustBeObject);
        }
        self.skip_whitespace();
        let mut fields = TopLevelFields::default();
        let mut seen = HashSet::new();
        if self.consume_byte(b'}') {
            return Ok(fields);
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            if !seen.insert(key.clone()) {
                return Err(WebDriverBiDiJsonEnvelopeError::DuplicateTopLevelMember);
            }
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            let value = self.parse_value(1)?;
            fields.record(&key, value);
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                return Ok(fields);
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, WebDriverBiDiJsonEnvelopeError> {
        if depth > MAX_WEBDRIVER_BIDI_JSON_DEPTH {
            return Err(WebDriverBiDiJsonEnvelopeError::NestingTooDeep {
                maximum_depth: MAX_WEBDRIVER_BIDI_JSON_DEPTH,
            });
        }
        self.skip_whitespace();
        match self.current_byte() {
            Some(b'"') => self.parse_string().map(JsonValue::Text),
            Some(b'{') => {
                self.parse_object(depth)?;
                Ok(JsonValue::Object)
            }
            Some(b'[') => {
                self.parse_array(depth)?;
                Ok(JsonValue::Other)
            }
            Some(b'n') => {
                self.parse_literal(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.parse_literal(b"true")?;
                Ok(JsonValue::Other)
            }
            Some(b'f') => {
                self.parse_literal(b"false")?;
                Ok(JsonValue::Other)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            _ => Err(WebDriverBiDiJsonEnvelopeError::InvalidJson),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        // `parse_value` dispatches here only after observing `{`; consume that proven delimiter
        // directly so an impossible second validation branch does not masquerade as parser evidence.
        self.index += 1;
        self.skip_whitespace();
        if self.consume_byte(b'}') {
            return Ok(());
        }
        loop {
            self.skip_whitespace();
            let _key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            let _value = self.parse_value(depth + 1)?;
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                return Ok(());
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        // `parse_value` dispatches here only after observing `[`; consume that proven delimiter
        // directly so an impossible second validation branch does not masquerade as parser evidence.
        self.index += 1;
        self.skip_whitespace();
        if self.consume_byte(b']') {
            return Ok(());
        }
        loop {
            let _value = self.parse_value(depth + 1)?;
            self.skip_whitespace();
            if self.consume_byte(b']') {
                return Ok(());
            }
            self.expect_byte(b',')?;
            self.skip_whitespace();
        }
    }

    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        let end = self.index.saturating_add(literal.len());
        if self.input.as_bytes().get(self.index..end) == Some(literal) {
            self.index = end;
            Ok(())
        } else {
            Err(WebDriverBiDiJsonEnvelopeError::InvalidJson)
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, WebDriverBiDiJsonEnvelopeError> {
        let start = self.index;
        let negative = self.consume_byte(b'-');
        if self.consume_byte(b'0') {
            if matches!(self.current_byte(), Some(b'0'..=b'9')) {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            }
        } else {
            match self.current_byte() {
                Some(b'1'..=b'9') => {
                    self.index += 1;
                    self.consume_digits();
                }
                _ => return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson),
            }
        }

        let mut is_integer = true;
        if self.consume_byte(b'.') {
            is_integer = false;
            if !matches!(self.current_byte(), Some(b'0'..=b'9')) {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            }
            self.consume_digits();
        }
        if matches!(self.current_byte(), Some(b'e' | b'E')) {
            is_integer = false;
            self.index += 1;
            if matches!(self.current_byte(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            if !matches!(self.current_byte(), Some(b'0'..=b'9')) {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            }
            self.consume_digits();
        }

        let js_uint = if !negative && is_integer {
            self.input[start..self.index]
                .parse::<u64>()
                .ok()
                .filter(|number| *number <= MAX_WEBDRIVER_BIDI_JS_UINT)
        } else {
            None
        };
        Ok(JsonValue::Number(js_uint))
    }

    fn consume_digits(&mut self) {
        while matches!(self.current_byte(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
    }

    fn parse_string(&mut self) -> Result<String, WebDriverBiDiJsonEnvelopeError> {
        self.expect_byte(b'"')?;
        let mut output = String::new();
        loop {
            let Some(byte) = self.current_byte() else {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            };
            match byte {
                b'"' => {
                    self.index += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.index += 1;
                    self.parse_escape(&mut output)?;
                }
                0x00..=0x1f => return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson),
                _ if byte.is_ascii() => {
                    output.push(char::from(byte));
                    self.index += 1;
                }
                _ => {
                    // `input` is valid UTF-8 and `index` advances only on character boundaries.
                    // For a non-ASCII lead byte, leading_ones therefore yields the exact width.
                    let character_byte_count = byte.leading_ones() as usize;
                    let end = self.index + character_byte_count;
                    output.push_str(&self.input[self.index..end]);
                    self.index = end;
                }
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        let Some(escape) = self.current_byte() else {
            return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
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
            b'u' => self.parse_unicode_escape(output)?,
            _ => return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson),
        }
        Ok(())
    }

    fn parse_unicode_escape(
        &mut self,
        output: &mut String,
    ) -> Result<(), WebDriverBiDiJsonEnvelopeError> {
        let first = self.parse_hex_u16()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            self.expect_byte(b'\\')?;
            self.expect_byte(b'u')?;
            let second = self.parse_hex_u16()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            }
            0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
        } else {
            u32::from(first)
        };
        // `scalar` is either a non-surrogate `u16` or the scalar constructed from a validated
        // high/low surrogate pair, so `char::from_u32` is always `Some` under this parser invariant.
        output.extend(char::from_u32(scalar));
        Ok(())
    }

    fn parse_hex_u16(&mut self) -> Result<u16, WebDriverBiDiJsonEnvelopeError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let Some(byte) = self.current_byte() else {
                return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson);
            };
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(WebDriverBiDiJsonEnvelopeError::InvalidJson),
            };
            value = (value << 4) | digit;
            self.index += 1;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> Result<WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError> {
        WebDriverBiDiJsonEnvelope::parse_str(value)
    }

    #[test]
    fn classifies_all_local_end_envelope_kinds_and_redacts_debug() {
        let success = parse(
            r#"{"type":"success","id":9007199254740991,"result":{"ready":true},"ext":[null,false,1.5,-2e3,"\u20ac","\ud83d\ude00"]}"#,
        );
        assert_eq!(
            success.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
            Ok(WebDriverBiDiJsonEnvelopeKind::Success)
        );
        assert_eq!(
            success.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
            Ok(Some(MAX_WEBDRIVER_BIDI_JS_UINT))
        );
        assert_eq!(
            success.as_ref().map(WebDriverBiDiJsonEnvelope::method),
            Ok(None)
        );
        assert_eq!(
            success.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
            Ok(None)
        );
        let debug = format!("{success:?}");
        assert!(debug.contains("Success"));
        assert!(!debug.contains("ready"));

        let error = parse(
            r#"{"type":"error","id":null,"error":"invalid argument","message":"secret detail","stacktrace":"hidden","vendor":{"x":[]}}"#,
        );
        assert_eq!(
            error.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
            Ok(WebDriverBiDiJsonEnvelopeKind::Error)
        );
        assert_eq!(
            error.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
            Ok(None)
        );
        assert_eq!(
            error.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
            Ok(Some("invalid argument"))
        );
        assert_eq!(
            error.as_ref().map(WebDriverBiDiJsonEnvelope::method),
            Ok(None)
        );
        let debug = format!("{error:?}");
        assert!(!debug.contains("invalid argument"));
        assert!(!debug.contains("secret detail"));

        let event =
            parse(r#"{"type":"event","method":"browsingContext.load","params":{},"vendor":true}"#);
        assert_eq!(
            event.as_ref().map(WebDriverBiDiJsonEnvelope::kind),
            Ok(WebDriverBiDiJsonEnvelopeKind::Event)
        );
        assert_eq!(
            event.as_ref().map(WebDriverBiDiJsonEnvelope::command_id),
            Ok(None)
        );
        assert_eq!(
            event.as_ref().map(WebDriverBiDiJsonEnvelope::method),
            Ok(Some("browsingContext.load"))
        );
        assert_eq!(
            event.as_ref().map(WebDriverBiDiJsonEnvelope::error_code),
            Ok(None)
        );
    }

    #[test]
    fn accepts_current_protocol_error_code_vocabulary() {
        for error_code in WEBDRIVER_BIDI_ERROR_CODES {
            let document =
                format!(r#"{{"type":"error","id":7,"error":"{error_code}","message":""}}"#);
            assert_eq!(
                parse(&document)
                    .as_ref()
                    .ok()
                    .and_then(WebDriverBiDiJsonEnvelope::error_code),
                Some(error_code)
            );
        }
        assert_eq!(
            parse(
                r#"{"type":"error","id":7,"error":"attacker-defined-code","message":"bad code"}"#
            ),
            Err(WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "error" })
        );
    }

    #[test]
    fn accepts_correlatable_error_and_extensible_success_members() {
        let error = parse(r#"{"type":"error","id":7,"error":"unknown error","message":"","x":0}"#);
        assert_eq!(
            error
                .as_ref()
                .ok()
                .and_then(WebDriverBiDiJsonEnvelope::command_id),
            Some(7)
        );

        let success = parse(
            "{\n \"\\u0074ype\":\"success\", \"id\":0, \"result\":{\"escaped\":\"\\\\/\\b\\f\\n\\r\\t\\\"\",\"unicode\":\"é\"}, \"method\":123 } \r\n",
        );
        assert!(success.is_ok());
    }

    #[test]
    fn envelope_shape_failures_are_typed() {
        let cases = [
            (
                "{}",
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "type" },
            ),
            (
                r#"{"type":1}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "type" },
            ),
            (
                r#"{"type":"other"}"#,
                WebDriverBiDiJsonEnvelopeError::UnsupportedEnvelopeType,
            ),
            (
                r#"{"type":"success","result":{}}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "id" },
            ),
            (
                r#"{"type":"success","id":null,"result":{}}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "id" },
            ),
            (
                r#"{"type":"success","id":9007199254740992,"result":{}}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "id" },
            ),
            (
                r#"{"type":"success","id":1}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "result" },
            ),
            (
                r#"{"type":"success","id":1,"result":[]}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "result" },
            ),
            (
                r#"{"type":"error","error":"x","message":"m"}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "id" },
            ),
            (
                r#"{"type":"error","id":-1,"error":"x","message":"m"}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "id" },
            ),
            (
                r#"{"type":"error","id":null,"message":"m"}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "error" },
            ),
            (
                r#"{"type":"error","id":null,"error":false,"message":"m"}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "error" },
            ),
            (
                r#"{"type":"error","id":null,"error":"x"}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "message" },
            ),
            (
                r#"{"type":"error","id":null,"error":"x","message":{}}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "message" },
            ),
            (
                r#"{"type":"error","id":null,"error":"x","message":"m","stacktrace":0}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember {
                    member: "stacktrace",
                },
            ),
            (
                r#"{"type":"event","params":{}}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "method" },
            ),
            (
                r#"{"type":"event","method":false,"params":{}}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "method" },
            ),
            (
                r#"{"type":"event","method":"x"}"#,
                WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "params" },
            ),
            (
                r#"{"type":"event","method":"x","params":null}"#,
                WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "params" },
            ),
        ];
        for (document, expected) in cases {
            assert_eq!(parse(document), Err(expected));
        }
    }

    #[test]
    fn rejects_ambiguous_or_malformed_json() {
        let cases = [
            "[]",
            "null",
            r#"{"type":"success","type":"event","id":1,"result":{}}"#,
            r#"{"type":"success","id":1,"result":{}} trailing"#,
            r#"{"type":"success","id":01,"result":{}}"#,
            r#"{"type":"success","id":1.,"result":{}}"#,
            r#"{"type":"success","id":1e,"result":{}}"#,
            r#"{"type":"success","id":1e+,"result":{}}"#,
            r#"{"type":"success","id":-,"result":{}}"#,
            r#"{"type":"success","id":18446744073709551616,"result":{}}"#,
            "{\"type\":\"success\",\"id\":1,\"result\":{\"bad\":\"line\nbreak\"}}",
            r#"{"type":"success","id":1,"result":{"bad":"\x"}}"#,
            r#"{"type":"success","id":1,"result":{"bad":"\u12xz"}}"#,
            r#"{"type":"success","id":1,"result":{"bad":"\ud800x"}}"#,
            r#"{"type":"success","id":1,"result":{"bad":"\ud800\u0041"}}"#,
            r#"{"type":"success","id":1,"result":{"bad":"\udc00"}}"#,
            r#"{"type":"success","id":1,"result":{"a":true "b":false}}"#,
            r#"{"type":"success","id":1,"result":[1,]}"#,
            r#"{"type":"success","id":1,"result":{"a":tru}}"#,
            r#"{"type":"success","id":1,"result":{"a":fal}}"#,
            r#"{"type":"success","id":1,"result":{"a":nul}}"#,
            r#"{"type":"success","id":1,"result":{"a":}}"#,
            r#"{"type":"success","id":1,"result":{"#,
        ];
        for document in cases {
            assert!(parse(document).is_err(), "unexpectedly admitted {document}");
        }
        assert_eq!(
            parse("[]"),
            Err(WebDriverBiDiJsonEnvelopeError::RootMustBeObject)
        );
        assert_eq!(
            parse(r#"{"type":"success","type":"event","id":1,"result":{}}"#),
            Err(WebDriverBiDiJsonEnvelopeError::DuplicateTopLevelMember)
        );
    }

    #[test]
    fn rejects_excessive_json_nesting_and_formats_errors_without_payloads() {
        let mut document = String::from(r#"{"type":"success","id":1,"result":"#);
        for _ in 0..=MAX_WEBDRIVER_BIDI_JSON_DEPTH {
            document.push('[');
        }
        document.push_str("null");
        for _ in 0..=MAX_WEBDRIVER_BIDI_JSON_DEPTH {
            document.push(']');
        }
        document.push('}');
        assert_eq!(
            parse(&document),
            Err(WebDriverBiDiJsonEnvelopeError::NestingTooDeep {
                maximum_depth: MAX_WEBDRIVER_BIDI_JSON_DEPTH,
            })
        );

        let errors = [
            WebDriverBiDiJsonEnvelopeError::InvalidJson,
            WebDriverBiDiJsonEnvelopeError::RootMustBeObject,
            WebDriverBiDiJsonEnvelopeError::DuplicateTopLevelMember,
            WebDriverBiDiJsonEnvelopeError::NestingTooDeep {
                maximum_depth: MAX_WEBDRIVER_BIDI_JSON_DEPTH,
            },
            WebDriverBiDiJsonEnvelopeError::MissingRequiredMember { member: "id" },
            WebDriverBiDiJsonEnvelopeError::InvalidMember { member: "result" },
            WebDriverBiDiJsonEnvelopeError::UnsupportedEnvelopeType,
        ];
        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
            let source: &dyn Error = &error;
            assert!(source.source().is_none());
        }
    }
}
