//! Bounded parsing for one Chrome native-messaging host manifest document.
//!
//! This module bounds untrusted document bytes before allocation, validates complete JSON syntax
//! for the reviewed native-host schema, and then delegates authority-bearing field validation to
//! [`NativeMessagingHostManifest`]. Parsing a document does not prove that the manifest is
//! installed by Chrome, authenticated by the operating system, or safe to use as process or Agent
//! authority.

use std::fmt;

use crate::{
    NativeMessagingHostManifest, NativeMessagingHostManifestError, NativeMessagingHostName,
    NativeMessagingHostNameError, NativeMessagingHostPlatform,
};

/// Maximum UTF-8 byte length accepted for one native-messaging host manifest document.
///
/// Chrome does not define this OriginWeave-specific 64 KiB safety budget. The limit exists to
/// bound allocation and parser input before any JSON or authority-bearing field processing.
pub const MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES: usize = 64 * 1024;

/// A bounded UTF-8 native-messaging host manifest document awaiting structured parsing.
///
/// Possessing this value proves only that the original byte document was non-empty, within the
/// OriginWeave ingress budget, valid UTF-8, and wrapped by one outer object boundary after JSON
/// whitespace is ignored. Call [`Self::parse_host_manifest`] to establish complete JSON/schema
/// validity and the existing host-manifest authority contract. This value alone carries no
/// installation, origin, executable, process, or Agent authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeMessagingManifestDocument {
    text: String,
}

impl NativeMessagingManifestDocument {
    /// Admit one untrusted manifest document before structured JSON parsing.
    ///
    /// The byte-size check runs before UTF-8 decoding or allocation of the stored `String` so
    /// oversized input cannot force unbounded parser or text-storage work. Empty input, invalid
    /// UTF-8, and documents whose first and last non-JSON-whitespace characters are not `{` and
    /// `}` fail closed. The object-envelope check is only a cheap ingress guard;
    /// [`Self::parse_host_manifest`] must still prove complete JSON syntax and field semantics.
    pub fn parse(bytes: &[u8]) -> Result<Self, NativeMessagingManifestDocumentError> {
        if bytes.is_empty() {
            return Err(NativeMessagingManifestDocumentError::EmptyDocument);
        }
        if bytes.len() > MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES {
            return Err(NativeMessagingManifestDocumentError::DocumentTooLarge);
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_error| NativeMessagingManifestDocumentError::InvalidUtf8)?;
        let trimmed = text.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n'));
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary);
        }
        Ok(Self {
            text: text.to_owned(),
        })
    }

    /// Parse the complete reviewed Chrome native-host manifest schema and validate its authority.
    ///
    /// The parser accepts exactly the required `name`, `description`, `path`, `type`, and
    /// `allowed_origins` members plus the optional boolean
    /// `supports_native_initiated_connections`. Duplicate decoded member names, unknown members,
    /// missing required members, wrong JSON types, malformed escapes, malformed arrays, trailing
    /// commas, and trailing JSON data all fail closed. JSON strings are decoded before the
    /// existing host-name, path, interface, and extension-origin validators run.
    ///
    /// `description` is required and type-checked because Chrome's manifest schema requires it,
    /// but it is intentionally not retained as authority. The optional native-initiated field
    /// defaults to `false` when absent. A successful result still does not prove installation,
    /// filesystem ownership, executable identity, process provenance, feature/policy enablement,
    /// message provenance, or Agent authority.
    pub fn parse_host_manifest(
        &self,
        platform: NativeMessagingHostPlatform,
    ) -> Result<NativeMessagingHostManifest, NativeMessagingManifestParseError> {
        let fields = ManifestJsonParser::new(&self.text).parse_manifest()?;
        let host_name = NativeMessagingHostName::parse(&fields.name)
            .map_err(NativeMessagingManifestParseError::HostName)?;
        let allowed_origins: Vec<&str> = fields.allowed_origins.iter().map(String::as_str).collect();
        NativeMessagingHostManifest::parse_with_native_initiated_connections(
            host_name,
            platform,
            &fields.executable_path,
            &fields.interface_type,
            fields.supports_native_initiated_connections,
            &allowed_origins,
        )
        .map_err(NativeMessagingManifestParseError::Manifest)
    }

    /// Return the exact validated UTF-8 text without interpreting JSON fields.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ManifestFields {
    name: String,
    executable_path: String,
    interface_type: String,
    allowed_origins: Vec<String>,
    supports_native_initiated_connections: bool,
}

#[derive(Debug, Default)]
struct PartialManifestFields {
    name: Option<String>,
    description: Option<String>,
    executable_path: Option<String>,
    interface_type: Option<String>,
    allowed_origins: Option<Vec<String>>,
    supports_native_initiated_connections: Option<bool>,
}

impl PartialManifestFields {
    fn finish(self) -> Result<ManifestFields, NativeMessagingManifestParseError> {
        let (Some(name), Some(_description), Some(executable_path), Some(interface_type), Some(allowed_origins)) = (
            self.name,
            self.description,
            self.executable_path,
            self.interface_type,
            self.allowed_origins,
        ) else {
            return Err(NativeMessagingManifestParseError::MissingRequiredField);
        };
        Ok(ManifestFields {
            name,
            executable_path,
            interface_type,
            allowed_origins,
            supports_native_initiated_connections: self
                .supports_native_initiated_connections
                .unwrap_or(false),
        })
    }
}

struct ManifestJsonParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> ManifestJsonParser<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse_manifest(mut self) -> Result<ManifestFields, NativeMessagingManifestParseError> {
        self.skip_whitespace();
        self.expect_byte(b'{')?;
        self.skip_whitespace();
        let mut fields = PartialManifestFields::default();
        if self.peek_byte() == Some(b'}') {
            self.position += 1;
        } else {
            loop {
                let key = self.parse_string()?;
                self.skip_whitespace();
                self.expect_byte(b':')?;
                self.skip_whitespace();
                self.parse_field(&key, &mut fields)?;
                self.skip_whitespace();
                match self.peek_byte() {
                    Some(b',') => {
                        self.position += 1;
                        self.skip_whitespace();
                        if self.peek_byte() == Some(b'}') {
                            return Err(NativeMessagingManifestParseError::InvalidJson);
                        }
                    }
                    Some(b'}') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(NativeMessagingManifestParseError::InvalidJson),
                }
            }
        }
        self.skip_whitespace();
        if self.position != self.input.len() {
            return Err(NativeMessagingManifestParseError::InvalidJson);
        }
        fields.finish()
    }

    fn parse_field(
        &mut self,
        key: &str,
        fields: &mut PartialManifestFields,
    ) -> Result<(), NativeMessagingManifestParseError> {
        match key {
            "name" => {
                if fields.name.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.name = Some(self.parse_typed_string()?);
            }
            "description" => {
                if fields.description.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.description = Some(self.parse_typed_string()?);
            }
            "path" => {
                if fields.executable_path.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.executable_path = Some(self.parse_typed_string()?);
            }
            "type" => {
                if fields.interface_type.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.interface_type = Some(self.parse_typed_string()?);
            }
            "allowed_origins" => {
                if fields.allowed_origins.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.allowed_origins = Some(self.parse_string_array()?);
            }
            "supports_native_initiated_connections" => {
                if fields.supports_native_initiated_connections.is_some() {
                    return Err(NativeMessagingManifestParseError::DuplicateField);
                }
                fields.supports_native_initiated_connections = Some(self.parse_boolean()?);
            }
            _ => return Err(NativeMessagingManifestParseError::UnknownField),
        }
        Ok(())
    }

    fn parse_typed_string(&mut self) -> Result<String, NativeMessagingManifestParseError> {
        if self.peek_byte() != Some(b'"') {
            return Err(NativeMessagingManifestParseError::InvalidFieldType);
        }
        self.parse_string()
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, NativeMessagingManifestParseError> {
        if self.peek_byte() != Some(b'[') {
            return Err(NativeMessagingManifestParseError::InvalidFieldType);
        }
        self.position += 1;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.peek_byte() == Some(b']') {
            self.position += 1;
            return Ok(values);
        }
        loop {
            if self.peek_byte() != Some(b'"') {
                return Err(NativeMessagingManifestParseError::InvalidFieldType);
            }
            values.push(self.parse_string()?);
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                    if self.peek_byte() == Some(b']') {
                        return Err(NativeMessagingManifestParseError::InvalidJson);
                    }
                }
                Some(b']') => {
                    self.position += 1;
                    return Ok(values);
                }
                _ => return Err(NativeMessagingManifestParseError::InvalidJson),
            }
        }
    }

    fn parse_boolean(&mut self) -> Result<bool, NativeMessagingManifestParseError> {
        if self.input[self.position..].starts_with("true") {
            self.position += 4;
            return Ok(true);
        }
        if self.input[self.position..].starts_with("false") {
            self.position += 5;
            return Ok(false);
        }
        Err(NativeMessagingManifestParseError::InvalidFieldType)
    }

    fn parse_string(&mut self) -> Result<String, NativeMessagingManifestParseError> {
        self.expect_byte(b'"')?;
        let mut output = String::new();
        loop {
            let Some(byte) = self.peek_byte() else {
                return Err(NativeMessagingManifestParseError::InvalidJson);
            };
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.position += 1;
                    self.parse_escape(&mut output)?;
                }
                0x00..=0x1f => return Err(NativeMessagingManifestParseError::InvalidJson),
                _ => {
                    let Some(character) = self.input[self.position..].chars().next() else {
                        return Err(NativeMessagingManifestParseError::InvalidJson);
                    };
                    output.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
    }

    fn parse_escape(
        &mut self,
        output: &mut String,
    ) -> Result<(), NativeMessagingManifestParseError> {
        let Some(escape) = self.take_byte() else {
            return Err(NativeMessagingManifestParseError::InvalidJson);
        };
        match escape {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{0008}'),
            b'f' => output.push('\u{000c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => output.push(self.parse_unicode_escape()?),
            _ => return Err(NativeMessagingManifestParseError::InvalidJson),
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self) -> Result<char, NativeMessagingManifestParseError> {
        let first = self.parse_hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if self.take_byte() != Some(b'\\') || self.take_byte() != Some(b'u') {
                return Err(NativeMessagingManifestParseError::InvalidJson);
            }
            let second = self.parse_hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(NativeMessagingManifestParseError::InvalidJson);
            }
            0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else {
            if (0xdc00..=0xdfff).contains(&first) {
                return Err(NativeMessagingManifestParseError::InvalidJson);
            }
            u32::from(first)
        };
        char::from_u32(scalar).ok_or(NativeMessagingManifestParseError::InvalidJson)
    }

    fn parse_hex_quad(&mut self) -> Result<u16, NativeMessagingManifestParseError> {
        if self.position + 4 > self.input.len() {
            return Err(NativeMessagingManifestParseError::InvalidJson);
        }
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self.input.as_bytes()[self.position];
            let Some(digit) = (byte as char).to_digit(16) else {
                return Err(NativeMessagingManifestParseError::InvalidJson);
            };
            value = (value << 4) | digit as u16;
            self.position += 1;
        }
        Ok(value)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.position += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), NativeMessagingManifestParseError> {
        if self.take_byte() == Some(expected) {
            Ok(())
        } else {
            Err(NativeMessagingManifestParseError::InvalidJson)
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }

    fn take_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.position += 1;
        Some(byte)
    }
}

/// Failure to admit a native-messaging host manifest document at the pre-parser boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingManifestDocumentError {
    /// The manifest document contained zero bytes.
    EmptyDocument,
    /// The manifest document exceeded the OriginWeave pre-parser safety budget.
    DocumentTooLarge,
    /// The manifest document was not valid UTF-8.
    InvalidUtf8,
    /// The document did not have one outer object boundary after JSON whitespace was removed.
    InvalidObjectBoundary,
}

impl fmt::Display for NativeMessagingManifestDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => {
                formatter.write_str("native messaging host manifest document is empty")
            }
            Self::DocumentTooLarge => formatter.write_str(
                "native messaging host manifest document exceeds the OriginWeave safety budget",
            ),
            Self::InvalidUtf8 => {
                formatter.write_str("native messaging host manifest document is not valid UTF-8")
            }
            Self::InvalidObjectBoundary => formatter.write_str(
                "native messaging host manifest document must have one outer JSON object boundary",
            ),
        }
    }
}

impl std::error::Error for NativeMessagingManifestDocumentError {}

/// Failure to parse or validate a complete bounded native-messaging host manifest document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeMessagingManifestParseError {
    /// The document was not one complete valid JSON object in the reviewed schema.
    InvalidJson,
    /// A decoded manifest member appeared more than once.
    DuplicateField,
    /// The manifest contained a member outside the reviewed Chrome native-host schema.
    UnknownField,
    /// One or more Chrome-required manifest members were absent.
    MissingRequiredField,
    /// A reviewed member used a JSON type different from the Chrome manifest contract.
    InvalidFieldType,
    /// The decoded host-name string violated the existing exact host-identity contract.
    HostName(NativeMessagingHostNameError),
    /// The decoded authority-bearing fields failed the existing host-manifest validator.
    Manifest(NativeMessagingHostManifestError),
}

impl fmt::Display for NativeMessagingManifestParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => formatter.write_str("native messaging host manifest JSON is invalid"),
            Self::DuplicateField => {
                formatter.write_str("native messaging host manifest contains a duplicate field")
            }
            Self::UnknownField => {
                formatter.write_str("native messaging host manifest contains an unknown field")
            }
            Self::MissingRequiredField => formatter
                .write_str("native messaging host manifest is missing a required field"),
            Self::InvalidFieldType => {
                formatter.write_str("native messaging host manifest field has an invalid JSON type")
            }
            Self::HostName(error) => write!(formatter, "invalid native messaging host name: {error}"),
            Self::Manifest(error) => write!(formatter, "invalid native messaging host manifest: {error}"),
        }
    }
}

impl std::error::Error for NativeMessagingManifestParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HostName(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::InvalidJson
            | Self::DuplicateField
            | Self::UnknownField
            | Self::MissingRequiredField
            | Self::InvalidFieldType => None,
        }
    }
}
