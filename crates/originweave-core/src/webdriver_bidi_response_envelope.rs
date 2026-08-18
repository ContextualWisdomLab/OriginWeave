use std::{error::Error, fmt};

use crate::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_COMMAND_ID,
    WebDriverBiDiCommandResponseKind,
};

/// Maximum accepted JSON container nesting depth for one WebDriver BiDi response document.
///
/// The top-level response object is depth 1. The limit is an OriginWeave resource-safety
/// budget, not a WebDriver BiDi protocol maximum.
pub const MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH: usize = 64;

/// Maximum accepted number of fields in one top-level WebDriver BiDi response object.
///
/// The limit is an OriginWeave resource-safety budget, not a WebDriver BiDi protocol maximum.
pub const MAX_WEBDRIVER_BIDI_RESPONSE_TOP_LEVEL_FIELDS: usize = 64;

/// Fail-closed reasons a bounded WebDriver BiDi response document cannot become typed envelope
/// evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiResponseEnvelopeParseError {
    /// The document is not syntactically valid JSON with one complete top-level object.
    InvalidJson,
    /// JSON object/array nesting exceeded the configured parser safety budget.
    JsonDepthExceeded,
    /// The top-level response object contains more fields than the configured safety budget.
    TopLevelFieldCountExceeded,
    /// The top-level response object repeats a field after JSON string escape decoding.
    DuplicateTopLevelField,
    /// The response object omits the required `type` discriminator.
    MissingResponseType,
    /// The response `type` is not exactly `success` or `error`.
    UnexpectedResponseType,
    /// The response object omits the required `id` field.
    MissingResponseId,
    /// The response `id` is not a protocol-range JSON integer, or is `null` where forbidden.
    InvalidResponseId,
    /// The selected response kind omits one of its required payload fields.
    MissingRequiredPayload,
    /// A required response payload field has the wrong JSON value type.
    InvalidRequiredPayloadType,
}

impl fmt::Display for WebDriverBiDiResponseEnvelopeParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidJson => "WebDriver BiDi response document is not valid JSON",
            Self::JsonDepthExceeded => {
                "WebDriver BiDi response JSON depth exceeds the safety budget"
            }
            Self::TopLevelFieldCountExceeded => {
                "WebDriver BiDi response top-level field count exceeds the safety budget"
            }
            Self::DuplicateTopLevelField => {
                "WebDriver BiDi response contains a duplicate top-level field"
            }
            Self::MissingResponseType => "WebDriver BiDi response is missing its type field",
            Self::UnexpectedResponseType => "WebDriver BiDi response type is not success or error",
            Self::MissingResponseId => "WebDriver BiDi response is missing its id field",
            Self::InvalidResponseId => "WebDriver BiDi response id is invalid",
            Self::MissingRequiredPayload => {
                "WebDriver BiDi response is missing a required payload field"
            }
            Self::InvalidRequiredPayloadType => {
                "WebDriver BiDi response payload field has an invalid JSON type"
            }
        })
    }
}

impl Error for WebDriverBiDiResponseEnvelopeParseError {}

/// Typed evidence that one bounded raw document is a syntactically valid WebDriver BiDi command
/// response envelope.
///
/// The value retains the exact admitted wire text and exposes only the command-response kind and
/// parsed response identifier needed by later correlation. Parsing does not authenticate a browser
/// or transport and does not grant browser, node, policy, or Agent authority.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedWebDriverBiDiCommandResponseEnvelope {
    document: BoundedWebDriverBiDiResponseDocument,
    kind: WebDriverBiDiCommandResponseKind,
    response_id: Option<u64>,
}

impl ParsedWebDriverBiDiCommandResponseEnvelope {
    /// Returns whether the parsed command response is a success or error envelope.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiCommandResponseKind {
        self.kind
    }

    /// Returns the parsed command identifier, or `None` only for an error response whose required
    /// `id` field was explicitly JSON `null`.
    #[must_use]
    pub const fn response_id(&self) -> Option<u64> {
        self.response_id
    }

    /// Returns the exact bounded wire text from which this envelope evidence was parsed.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.document.as_str()
    }
}

impl BoundedWebDriverBiDiResponseDocument {
    /// Parses this already-bounded raw document into typed command-response envelope evidence.
    ///
    /// Complete JSON syntax, decoded top-level field uniqueness, response-kind requirements,
    /// protocol-range response identifiers, and explicit parser resource budgets are enforced
    /// before the value can be used by a later correlation boundary.
    pub fn parse_command_response(
        self,
    ) -> Result<ParsedWebDriverBiDiCommandResponseEnvelope, WebDriverBiDiResponseEnvelopeParseError>
    {
        let parsed = ResponseEnvelopeParser::new(self.as_str()).parse()?;
        Ok(ParsedWebDriverBiDiCommandResponseEnvelope {
            document: self,
            kind: parsed.kind,
            response_id: parsed.response_id,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParsedJsonValue {
    Object,
    Array,
    String(Vec<u8>),
    Number(String),
    Boolean,
    Null,
}

struct ParsedEnvelopeFields {
    kind: WebDriverBiDiCommandResponseKind,
    response_id: Option<u64>,
}

struct ResponseEnvelopeParser<'input> {
    input: &'input str,
    position: usize,
}

impl<'input> ResponseEnvelopeParser<'input> {
    const fn new(input: &'input str) -> Self {
        Self { input, position: 0 }
    }

    fn parse(mut self) -> Result<ParsedEnvelopeFields, WebDriverBiDiResponseEnvelopeParseError> {
        self.skip_whitespace();
        self.expect_byte(b'{')?;
        self.skip_whitespace();

        let mut seen_fields: Vec<Vec<u8>> = Vec::new();
        let mut response_type = None;
        let mut response_id = None;
        let mut result = None;
        let mut error_code = None;
        let mut message = None;
        let mut stacktrace = None;

        if self.peek_byte() != Some(b'}') {
            loop {
                if seen_fields.len() >= MAX_WEBDRIVER_BIDI_RESPONSE_TOP_LEVEL_FIELDS {
                    return Err(
                        WebDriverBiDiResponseEnvelopeParseError::TopLevelFieldCountExceeded,
                    );
                }

                let field_name = self.parse_string()?;
                if seen_fields.contains(&field_name) {
                    return Err(WebDriverBiDiResponseEnvelopeParseError::DuplicateTopLevelField);
                }
                seen_fields.push(field_name.clone());
                self.skip_whitespace();
                self.expect_byte(b':')?;
                self.skip_whitespace();
                let value = self.parse_value(2)?;

                match field_name.as_slice() {
                    b"type" => response_type = Some(value),
                    b"id" => response_id = Some(value),
                    b"result" => result = Some(value),
                    b"error" => error_code = Some(value),
                    b"message" => message = Some(value),
                    b"stacktrace" => stacktrace = Some(value),
                    _ => {}
                }

                self.skip_whitespace();
                match self.peek_byte() {
                    Some(b',') => {
                        self.position += 1;
                        self.skip_whitespace();
                    }
                    Some(b'}') => break,
                    _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
                }
            }
        }

        self.expect_byte(b'}')?;
        self.skip_whitespace();
        if self.position != self.input.len() {
            return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
        }

        let kind = Self::parse_response_type(response_type)?;
        let response_id = Self::parse_response_id(response_id, kind)?;
        Self::validate_required_payload(kind, result, error_code, message, stacktrace)?;

        Ok(ParsedEnvelopeFields { kind, response_id })
    }

    fn parse_response_type(
        value: Option<ParsedJsonValue>,
    ) -> Result<WebDriverBiDiCommandResponseKind, WebDriverBiDiResponseEnvelopeParseError> {
        let value = value.ok_or(WebDriverBiDiResponseEnvelopeParseError::MissingResponseType)?;
        match value {
            ParsedJsonValue::String(value) if value == b"success" => {
                Ok(WebDriverBiDiCommandResponseKind::Success)
            }
            ParsedJsonValue::String(value) if value == b"error" => {
                Ok(WebDriverBiDiCommandResponseKind::Error)
            }
            _ => Err(WebDriverBiDiResponseEnvelopeParseError::UnexpectedResponseType),
        }
    }

    fn parse_response_id(
        value: Option<ParsedJsonValue>,
        kind: WebDriverBiDiCommandResponseKind,
    ) -> Result<Option<u64>, WebDriverBiDiResponseEnvelopeParseError> {
        let value = value.ok_or(WebDriverBiDiResponseEnvelopeParseError::MissingResponseId)?;
        let raw = match value {
            ParsedJsonValue::Null if kind == WebDriverBiDiCommandResponseKind::Error => {
                return Ok(None);
            }
            ParsedJsonValue::Number(raw) => raw,
            _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId),
        };
        if !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId);
        }
        let parsed = raw
            .parse::<u64>()
            .map_err(|_| WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId)?;
        if parsed > MAX_WEBDRIVER_BIDI_COMMAND_ID {
            return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId);
        }
        Ok(Some(parsed))
    }

    fn validate_required_payload(
        kind: WebDriverBiDiCommandResponseKind,
        result: Option<ParsedJsonValue>,
        error_code: Option<ParsedJsonValue>,
        message: Option<ParsedJsonValue>,
        stacktrace: Option<ParsedJsonValue>,
    ) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        match kind {
            WebDriverBiDiCommandResponseKind::Success => {
                let result = result
                    .ok_or(WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload)?;
                if !matches!(result, ParsedJsonValue::Object) {
                    return Err(
                        WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
                    );
                }
            }
            WebDriverBiDiCommandResponseKind::Error => {
                let error_code = error_code
                    .ok_or(WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload)?;
                let message = message
                    .ok_or(WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload)?;
                if !matches!(error_code, ParsedJsonValue::String(_))
                    || !matches!(message, ParsedJsonValue::String(_))
                {
                    return Err(
                        WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
                    );
                }
                if let Some(stacktrace) = stacktrace {
                    if !matches!(stacktrace, ParsedJsonValue::String(_)) {
                        return Err(
                            WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_value(
        &mut self,
        container_depth: usize,
    ) -> Result<ParsedJsonValue, WebDriverBiDiResponseEnvelopeParseError> {
        match self.peek_byte() {
            Some(b'{') => {
                self.parse_object(container_depth)?;
                Ok(ParsedJsonValue::Object)
            }
            Some(b'[') => {
                self.parse_array(container_depth)?;
                Ok(ParsedJsonValue::Array)
            }
            Some(b'"') => Ok(ParsedJsonValue::String(self.parse_string()?)),
            Some(b'-' | b'0'..=b'9') => Ok(ParsedJsonValue::Number(self.parse_number()?)),
            Some(b't') => {
                self.parse_literal(b"true")?;
                Ok(ParsedJsonValue::Boolean)
            }
            Some(b'f') => {
                self.parse_literal(b"false")?;
                Ok(ParsedJsonValue::Boolean)
            }
            Some(b'n') => {
                self.parse_literal(b"null")?;
                Ok(ParsedJsonValue::Null)
            }
            _ => Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
        }
    }

    fn parse_object(
        &mut self,
        depth: usize,
    ) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        Self::require_depth(depth)?;
        self.expect_byte(b'{')?;
        self.skip_whitespace();
        if self.peek_byte() == Some(b'}') {
            self.position += 1;
            return Ok(());
        }

        loop {
            self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_whitespace();
            self.parse_value(depth + 1)?;
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        Self::require_depth(depth)?;
        self.expect_byte(b'[')?;
        self.skip_whitespace();
        if self.peek_byte() == Some(b']') {
            self.position += 1;
            return Ok(());
        }

        loop {
            self.parse_value(depth + 1)?;
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.position += 1;
                    return Ok(());
                }
                _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
            }
        }
    }

    fn require_depth(depth: usize) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        if depth > MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH {
            Err(WebDriverBiDiResponseEnvelopeParseError::JsonDepthExceeded)
        } else {
            Ok(())
        }
    }

    fn parse_string(&mut self) -> Result<Vec<u8>, WebDriverBiDiResponseEnvelopeParseError> {
        self.expect_byte(b'"')?;
        let mut decoded = Vec::new();
        loop {
            let byte = self
                .peek_byte()
                .ok_or(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(decoded);
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut decoded)?;
                }
                0x00..=0x1f => {
                    return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
                }
                _ => {
                    decoded.push(byte);
                    self.position += 1;
                }
            }
        }
    }

    fn parse_escape(
        &mut self,
        decoded: &mut Vec<u8>,
    ) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        let escaped = self
            .peek_byte()
            .ok_or(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)?;
        self.position += 1;
        match escaped {
            b'"' => decoded.push(b'"'),
            b'\\' => decoded.push(b'\\'),
            b'/' => decoded.push(b'/'),
            b'b' => decoded.push(0x08),
            b'f' => decoded.push(0x0c),
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b't' => decoded.push(b'\t'),
            b'u' => {
                let scalar = self.parse_unicode_escape()?;
                Self::push_utf8(decoded, scalar);
            }
            _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, WebDriverBiDiResponseEnvelopeParseError> {
        let first = self.parse_hex_code_unit()?;
        if (0xd800..=0xdbff).contains(&first) {
            if self.peek_byte() != Some(b'\\') {
                return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
            }
            self.position += 1;
            if self.peek_byte() != Some(b'u') {
                return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
            }
            self.position += 1;
            let second = self.parse_hex_code_unit()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
            }
            return Ok(0x1_0000
                + ((u32::from(first) - 0xd800) << 10)
                + (u32::from(second) - 0xdc00));
        }
        if (0xdc00..=0xdfff).contains(&first) {
            return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
        }
        Ok(u32::from(first))
    }

    fn push_utf8(decoded: &mut Vec<u8>, scalar: u32) {
        if scalar <= 0x7f {
            decoded.push(scalar as u8);
        } else if scalar <= 0x7ff {
            decoded.push((0xc0 | (scalar >> 6)) as u8);
            decoded.push((0x80 | (scalar & 0x3f)) as u8);
        } else if scalar <= 0xffff {
            decoded.push((0xe0 | (scalar >> 12)) as u8);
            decoded.push((0x80 | ((scalar >> 6) & 0x3f)) as u8);
            decoded.push((0x80 | (scalar & 0x3f)) as u8);
        } else {
            decoded.push((0xf0 | (scalar >> 18)) as u8);
            decoded.push((0x80 | ((scalar >> 12) & 0x3f)) as u8);
            decoded.push((0x80 | ((scalar >> 6) & 0x3f)) as u8);
            decoded.push((0x80 | (scalar & 0x3f)) as u8);
        }
    }

    fn parse_hex_code_unit(&mut self) -> Result<u16, WebDriverBiDiResponseEnvelopeParseError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self
                .peek_byte()
                .ok_or(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)?;
            let digit = Self::hex_value(byte)
                .ok_or(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)?;
            value = (value << 4) | u16::from(digit);
            self.position += 1;
        }
        Ok(value)
    }

    const fn hex_value(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Result<String, WebDriverBiDiResponseEnvelopeParseError> {
        let start = self.position;
        if self.peek_byte() == Some(b'-') {
            self.position += 1;
        }

        match self.peek_byte() {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
                }
            }
            Some(b'1'..=b'9') => self.consume_digits(),
            _ => return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson),
        }

        if self.peek_byte() == Some(b'.') {
            self.position += 1;
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
            }
            self.consume_digits();
        }

        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
            }
            self.consume_digits();
        }

        Ok(self.input[start..self.position].to_owned())
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
    }

    fn parse_literal(
        &mut self,
        literal: &[u8],
    ) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        let end = self.position + literal.len();
        if self.input.as_bytes().get(self.position..end) != Some(literal) {
            return Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson);
        }
        self.position = end;
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.position += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), WebDriverBiDiResponseEnvelopeParseError> {
        if self.peek_byte() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }
}
