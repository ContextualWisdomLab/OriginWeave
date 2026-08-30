use std::{error::Error, fmt};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserRegistryError, BrowserSessionId, BrowsingContextId,
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
};

use crate::{
    MAX_WEBDRIVER_BIDI_JS_UINT, WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError,
    WebDriverBiDiJsonEnvelopeKind, WebDriverBiDiWebSocketTextMessage,
};

/// WebDriver BiDi event method that reports a committed browsing-context navigation.
pub const WEBDRIVER_BIDI_NAVIGATION_COMMITTED_METHOD: &str = "browsingContext.navigationCommitted";

/// Maximum UTF-8 bytes retained for one opaque WebDriver BiDi navigation identifier.
pub const MAX_WEBDRIVER_BIDI_NAVIGATION_IDENTIFIER_BYTES: usize =
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES;

/// Maximum UTF-8 bytes retained for one observed serialized navigation URL.
pub const MAX_WEBDRIVER_BIDI_NAVIGATION_URL_BYTES: usize = 16 * 1024;

/// A typed local-end observation that one exact registered browser context committed the expected URL.
///
/// This value is evidence of the WebDriver BiDi event only. It does not advance an OriginWeave
/// document epoch, bind an origin, prove which action caused the navigation, or grant browser,
/// policy, node, destination, credential, process, or reusable Agent authority.
pub struct WebDriverBiDiNavigationCommittedObservation {
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    navigation_id: Option<String>,
    timestamp: u64,
    url: String,
}

impl fmt::Debug for WebDriverBiDiNavigationCommittedObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiNavigationCommittedObservation")
            .field("browser_session", &self.browser_session.value())
            .field("browsing_context", &self.browsing_context.value())
            .field("has_navigation_id", &self.navigation_id.is_some())
            .field("timestamp", &self.timestamp)
            .field("url_bytes", &self.url.len())
            .finish()
    }
}

impl WebDriverBiDiNavigationCommittedObservation {
    /// Parse one complete event, bind its external context to the exact registered context, and
    /// require the observed URL to equal the caller's declared post-condition URL exactly.
    pub fn parse_and_match(
        message: &WebDriverBiDiWebSocketTextMessage,
        registry: &BrowserAuthorityRegistry,
        browser_session: BrowserSessionId,
        browsing_context: BrowsingContextId,
        expected_url: &str,
    ) -> Result<Self, WebDriverBiDiNavigationCommittedObservationError> {
        let envelope = WebDriverBiDiJsonEnvelope::parse(message).map_err(|source| {
            WebDriverBiDiNavigationCommittedObservationError::Envelope { source }
        })?;
        if envelope.kind() != WebDriverBiDiJsonEnvelopeKind::Event
            || envelope.method() != Some(WEBDRIVER_BIDI_NAVIGATION_COMMITTED_METHOD)
        {
            return Err(WebDriverBiDiNavigationCommittedObservationError::UnexpectedEvent);
        }

        let projected =
            NavigationCommittedProjection::parse(message.as_str()).map_err(|source| {
                WebDriverBiDiNavigationCommittedObservationError::Projection { source }
            })?;
        registry
            .require_registered_context_external_identifier(
                browser_session,
                browsing_context,
                &projected.context,
            )
            .map_err(
                |source| WebDriverBiDiNavigationCommittedObservationError::ContextBinding {
                    source,
                },
            )?;
        if projected.url != expected_url {
            return Err(WebDriverBiDiNavigationCommittedObservationError::UnexpectedUrl);
        }

        Ok(Self {
            browser_session,
            browsing_context,
            navigation_id: projected.navigation_id,
            timestamp: projected.timestamp,
            url: projected.url,
        })
    }

    /// Return the exact OriginWeave browser session whose registered context matched the event.
    #[must_use]
    pub const fn browser_session(&self) -> BrowserSessionId {
        self.browser_session
    }

    /// Return the exact OriginWeave browsing context whose external identifier matched the event.
    #[must_use]
    pub const fn browsing_context(&self) -> BrowsingContextId {
        self.browsing_context
    }

    /// Borrow the optional opaque WebDriver BiDi navigation identifier.
    #[must_use]
    pub fn navigation_id(&self) -> Option<&str> {
        self.navigation_id.as_deref()
    }

    /// Return the WebDriver BiDi monotonic event timestamp value admitted as a JavaScript uint.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Borrow the exact bounded serialized URL observed in the committed-navigation event.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// Fail-closed failures while admitting one committed-navigation post-condition observation.
#[derive(Debug)]
pub enum WebDriverBiDiNavigationCommittedObservationError {
    /// The complete local-end WebDriver BiDi JSON envelope was malformed.
    Envelope {
        /// Underlying complete-envelope validation failure.
        source: WebDriverBiDiJsonEnvelopeError,
    },
    /// The message was not the exact `browsingContext.navigationCommitted` event.
    UnexpectedEvent,
    /// Required navigation-info fields could not be projected safely.
    Projection {
        /// Underlying typed navigation-info projection failure.
        source: WebDriverBiDiNavigationCommittedProjectionError,
    },
    /// The event's external context did not map to the exact registered OriginWeave context.
    ContextBinding {
        /// Underlying browser-registry authority-binding failure.
        source: BrowserRegistryError,
    },
    /// The committed URL did not equal the caller's declared post-condition URL exactly.
    UnexpectedUrl,
}

impl fmt::Display for WebDriverBiDiNavigationCommittedObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Envelope { .. } => formatter
                .write_str("WebDriver BiDi navigation-committed envelope is invalid"),
            Self::UnexpectedEvent => formatter.write_str(
                "WebDriver BiDi message is not the expected navigation-committed event",
            ),
            Self::Projection { .. } => formatter
                .write_str("WebDriver BiDi navigation-committed params are invalid"),
            Self::ContextBinding { .. } => formatter.write_str(
                "WebDriver BiDi navigation-committed context does not match registered authority",
            ),
            Self::UnexpectedUrl => formatter.write_str(
                "WebDriver BiDi navigation-committed URL does not match the declared post-condition",
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedObservationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Projection { source } => Some(source),
            Self::ContextBinding { source } => Some(source),
            Self::UnexpectedEvent | Self::UnexpectedUrl => None,
        }
    }
}

/// Fail-closed failures while projecting W3C `browsingContext.NavigationInfo` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiNavigationCommittedProjectionError {
    /// The projection cursor encountered an impossible structure after common JSON validation.
    InvalidStructure,
    /// A required `NavigationInfo` member was absent.
    MissingRequiredMember {
        /// Missing member name.
        member: &'static str,
    },
    /// A required `NavigationInfo` member appeared more than once.
    DuplicateRequiredMember {
        /// Duplicated member name.
        member: &'static str,
    },
    /// The external browsing-context identifier was malformed or exceeded the reviewed bound.
    InvalidContextIdentifier,
    /// The optional navigation identifier was malformed or exceeded the reviewed bound.
    InvalidNavigationIdentifier,
    /// The navigation timestamp was not a canonical WebDriver BiDi JavaScript uint.
    InvalidTimestamp,
    /// The serialized URL exceeded the reviewed observation resource bound.
    UrlTooLarge {
        /// Maximum admitted UTF-8 URL bytes.
        maximum_bytes: usize,
    },
}

impl fmt::Display for WebDriverBiDiNavigationCommittedProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStructure => formatter
                .write_str("navigation-committed projection encountered invalid JSON structure"),
            Self::MissingRequiredMember { member } => {
                write!(
                    formatter,
                    "navigation-committed params are missing {member}"
                )
            }
            Self::DuplicateRequiredMember { member } => write!(
                formatter,
                "navigation-committed params contain duplicate {member}"
            ),
            Self::InvalidContextIdentifier => {
                formatter.write_str("navigation-committed context identifier is invalid")
            }
            Self::InvalidNavigationIdentifier => {
                formatter.write_str("navigation-committed navigation identifier is invalid")
            }
            Self::InvalidTimestamp => {
                formatter.write_str("navigation-committed timestamp is not a JavaScript uint")
            }
            Self::UrlTooLarge { maximum_bytes } => write!(
                formatter,
                "navigation-committed URL exceeds the {maximum_bytes}-byte observation limit"
            ),
        }
    }
}

impl Error for WebDriverBiDiNavigationCommittedProjectionError {}

struct NavigationCommittedProjection {
    context: String,
    navigation_id: Option<String>,
    timestamp: u64,
    url: String,
}

impl NavigationCommittedProjection {
    fn parse(input: &str) -> Result<Self, WebDriverBiDiNavigationCommittedProjectionError> {
        let mut cursor = ProjectionCursor::new(input);
        cursor.skip_whitespace();
        if !cursor.consume_byte(b'{') {
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
        }
        cursor.skip_whitespace();
        if cursor.consume_byte(b'}') {
            return Err(
                WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                    member: "params",
                },
            );
        }

        let mut projection = None;
        loop {
            cursor.skip_whitespace();
            let key = cursor
                .parse_string()
                .ok_or(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure)?;
            cursor.skip_whitespace();
            if !cursor.consume_byte(b':') {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
            cursor.skip_whitespace();
            if key == "params" {
                if projection.is_some() {
                    return Err(
                        WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                            member: "params",
                        },
                    );
                }
                projection = Some(cursor.parse_params_object()?);
            } else if !cursor.skip_value() {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
            cursor.skip_whitespace();
            if cursor.consume_byte(b'}') {
                break;
            }
            if !cursor.consume_byte(b',') {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
        }
        cursor.skip_whitespace();
        if cursor.current_byte().is_some() {
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
        }
        projection.ok_or(
            WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                member: "params",
            },
        )
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

    fn parse_params_object(
        &mut self,
    ) -> Result<NavigationCommittedProjection, WebDriverBiDiNavigationCommittedProjectionError>
    {
        if !self.consume_byte(b'{') {
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
        }
        self.skip_whitespace();
        if self.consume_byte(b'}') {
            return Err(
                WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                    member: "context",
                },
            );
        }

        let mut context = None;
        let mut navigation_seen = false;
        let mut navigation_id = None;
        let mut timestamp = None;
        let mut url = None;

        loop {
            self.skip_whitespace();
            let key = self
                .parse_string()
                .ok_or(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure)?;
            self.skip_whitespace();
            if !self.consume_byte(b':') {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
            self.skip_whitespace();
            match key.as_str() {
                "context" => {
                    if context.is_some() {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                                member: "context",
                            },
                        );
                    }
                    let parsed = self.parse_string().ok_or(
                        WebDriverBiDiNavigationCommittedProjectionError::InvalidContextIdentifier,
                    )?;
                    if !protocol_identifier_is_valid(&parsed, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES)
                    {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::InvalidContextIdentifier,
                        );
                    }
                    context = Some(parsed);
                }
                "navigation" => {
                    if navigation_seen {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                                member: "navigation",
                            },
                        );
                    }
                    navigation_seen = true;
                    if self.consume_literal(b"null") {
                        navigation_id = None;
                    } else {
                        let parsed = self.parse_string().ok_or(
                            WebDriverBiDiNavigationCommittedProjectionError::InvalidNavigationIdentifier,
                        )?;
                        if !protocol_identifier_is_valid(
                            &parsed,
                            MAX_WEBDRIVER_BIDI_NAVIGATION_IDENTIFIER_BYTES,
                        ) {
                            return Err(
                                WebDriverBiDiNavigationCommittedProjectionError::InvalidNavigationIdentifier,
                            );
                        }
                        navigation_id = Some(parsed);
                    }
                }
                "timestamp" => {
                    if timestamp.is_some() {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                                member: "timestamp",
                            },
                        );
                    }
                    timestamp = Some(self.parse_js_uint()?);
                }
                "url" => {
                    if url.is_some() {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                                member: "url",
                            },
                        );
                    }
                    let parsed = self
                        .parse_string()
                        .ok_or(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure)?;
                    if parsed.len() > MAX_WEBDRIVER_BIDI_NAVIGATION_URL_BYTES {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::UrlTooLarge {
                                maximum_bytes: MAX_WEBDRIVER_BIDI_NAVIGATION_URL_BYTES,
                            },
                        );
                    }
                    url = Some(parsed);
                }
                _ => {
                    if !self.skip_value() {
                        return Err(
                            WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure,
                        );
                    }
                }
            }

            self.skip_whitespace();
            if self.consume_byte(b'}') {
                break;
            }
            if !self.consume_byte(b',') {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
        }

        Ok(NavigationCommittedProjection {
            context: context.ok_or(
                WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                    member: "context",
                },
            )?,
            navigation_id: if navigation_seen {
                navigation_id
            } else {
                return Err(
                    WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                        member: "navigation",
                    },
                );
            },
            timestamp: timestamp.ok_or(
                WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                    member: "timestamp",
                },
            )?,
            url: url.ok_or(
                WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                    member: "url",
                },
            )?,
        })
    }

    fn parse_js_uint(&mut self) -> Result<u64, WebDriverBiDiNavigationCommittedProjectionError> {
        let start = self.index;
        if !self.skip_number() {
            if !self.skip_value() {
                return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure);
            }
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidTimestamp);
        }
        let raw = &self.input[start..self.index];
        if !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidTimestamp);
        }
        let value = raw
            .parse::<u64>()
            .map_err(|_error| WebDriverBiDiNavigationCommittedProjectionError::InvalidTimestamp)?;
        if value > MAX_WEBDRIVER_BIDI_JS_UINT {
            return Err(WebDriverBiDiNavigationCommittedProjectionError::InvalidTimestamp);
        }
        Ok(value)
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

fn protocol_identifier_is_valid(value: &str, maximum_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum_bytes
        && !value.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS.contains(&character)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_accepts_extensible_navigation_info_and_null_navigation() {
        let projected = NavigationCommittedProjection::parse(
            r#"{"type":"event","method":"browsingContext.navigationCommitted","meta":[null,true,false,-2.5e+3,{"nested":"value"}],"params":{"url":"https://example.test/\u0061fter","timestamp":42,"navigation":"nav-\ud83d\ude80","context":"context-a","vendor":{"ignored":true}}}"#,
        );
        assert!(projected.is_ok());
        let projected = projected.ok();
        assert_eq!(
            projected.as_ref().map(|value| value.context.as_str()),
            Some("context-a")
        );
        assert_eq!(
            projected
                .as_ref()
                .and_then(|value| value.navigation_id.as_deref()),
            Some("nav-🚀")
        );
        assert_eq!(projected.as_ref().map(|value| value.timestamp), Some(42));
        assert_eq!(
            projected.as_ref().map(|value| value.url.as_str()),
            Some("https://example.test/after")
        );

        let projected = NavigationCommittedProjection::parse(
            r#"{"params":{"context":"context-a","navigation":null,"timestamp":0,"url":"about:blank"}}"#,
        );
        assert!(projected.is_ok());
        assert_eq!(projected.ok().and_then(|value| value.navigation_id), None);
    }

    #[test]
    fn projection_rejects_missing_duplicate_invalid_and_oversized_required_fields() {
        let oversized_url = "x".repeat(MAX_WEBDRIVER_BIDI_NAVIGATION_URL_BYTES + 1);
        let oversized_navigation = "n".repeat(MAX_WEBDRIVER_BIDI_NAVIGATION_IDENTIFIER_BYTES + 1);
        let oversized_context = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1);
        let cases = [
            "{}".to_owned(),
            r#"{"params":{}}"#.to_owned(),
            r#"{"params":{"navigation":null,"timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"context-a","timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"context-a","navigation":null,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"context-a","navigation":null,"timestamp":1}}"#.to_owned(),
            r#"{"params":{},"params":{}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":1,"url":"x"},"params":{"context":"b","navigation":null,"timestamp":2,"url":"y"}}"#.to_owned(),
            r#"{"params":{"context":"a","context":"b","navigation":null,"timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"navigation":"b","timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":1,"timestamp":2,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":1,"url":"x","url":"y"}}"#.to_owned(),
            r#"{"params":{"context":"bad context","navigation":null,"timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":"bad nav","timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":false,"timestamp":1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":-1,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":1.5,"url":"x"}}"#.to_owned(),
            r#"{"params":{"context":"a","navigation":null,"timestamp":1,"url":"x"}}?"#.to_owned(),
            r#"{"params":{"context":"a","vendor":?,"navigation":null,"timestamp":1,"url":"x"}}"#.to_owned(),
            format!(
                "{{\"params\":{{\"context\":\"a\",\"navigation\":null,\"timestamp\":{},\"url\":\"x\"}}}}",
                MAX_WEBDRIVER_BIDI_JS_UINT + 1
            ),
            format!(
                "{{\"params\":{{\"context\":\"{}\",\"navigation\":null,\"timestamp\":1,\"url\":\"x\"}}}}",
                oversized_context
            ),
            format!(
                "{{\"params\":{{\"context\":\"a\",\"navigation\":\"{}\",\"timestamp\":1,\"url\":\"x\"}}}}",
                oversized_navigation
            ),
            format!(
                "{{\"params\":{{\"context\":\"a\",\"navigation\":null,\"timestamp\":1,\"url\":\"{}\"}}}}",
                oversized_url
            ),
        ];
        for document in cases {
            assert!(NavigationCommittedProjection::parse(&document).is_err());
        }
    }

    #[test]
    fn projection_cursor_defensive_helpers_cover_private_hostile_edges() {
        for document in [
            "",
            "[]",
            r#"{"x" 1}"#,
            r#"{"x":?}"#,
            r#"{"x":1 ?}"#,
            r#"{"params":[]}"#,
            r#"{"params":{?}}"#,
            r#"{"params":{"context" "a"}}"#,
            r#"{"params":{"context":"a" ?}}"#,
            r#"{"params":{"context":"\uD800","navigation":null,"timestamp":1,"url":"x"}}"#,
            r#"{"params":{"context":"\q","navigation":null,"timestamp":1,"url":"x"}}"#,
            r#"{"params":{"context":"a","navigation":null,"timestamp":?,"url":"x"}}"#,
        ] {
            assert!(NavigationCommittedProjection::parse(document).is_err());
        }

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

        for invalid in [
            "\"abc\\",
            r#""\uD800""#,
            r#""\uDC00""#,
            r#""\uD800\x""#,
            r#""\uD800\uZZZZ""#,
            r#""\uD800\u0041""#,
            r#""\uZZZZ""#,
            r#""\q""#,
        ] {
            let mut string = ProjectionCursor::new(invalid);
            assert!(string.parse_string().is_none());
        }

        assert!(!protocol_identifier_is_valid(
            "",
            MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES
        ));
        assert!(!protocol_identifier_is_valid(
            "\u{0000}",
            MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES
        ));
    }

    #[test]
    fn projection_error_display_is_specific_and_source_free() {
        let errors = [
            WebDriverBiDiNavigationCommittedProjectionError::InvalidStructure,
            WebDriverBiDiNavigationCommittedProjectionError::MissingRequiredMember {
                member: "url",
            },
            WebDriverBiDiNavigationCommittedProjectionError::DuplicateRequiredMember {
                member: "url",
            },
            WebDriverBiDiNavigationCommittedProjectionError::InvalidContextIdentifier,
            WebDriverBiDiNavigationCommittedProjectionError::InvalidNavigationIdentifier,
            WebDriverBiDiNavigationCommittedProjectionError::InvalidTimestamp,
            WebDriverBiDiNavigationCommittedProjectionError::UrlTooLarge {
                maximum_bytes: MAX_WEBDRIVER_BIDI_NAVIGATION_URL_BYTES,
            },
        ];
        for error in errors {
            assert!(!error.to_string().is_empty());
            assert!(error.source().is_none());
        }
    }
}
