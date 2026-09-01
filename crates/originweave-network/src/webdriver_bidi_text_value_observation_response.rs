use std::{collections::HashSet, error::Error, fmt};

use originweave_core::{
    MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES, UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiJsonEnvelope, WebDriverBiDiJsonEnvelopeError, WebDriverBiDiJsonEnvelopeKind,
    WebDriverBiDiWebSocketTextMessage,
};

const MAX_SCRIPT_RESULT_OBJECT_MEMBERS: usize = 64;
const MAX_SCRIPT_RESULT_MEMBER_NAME_BYTES: usize = 128;
const MAX_SCRIPT_RESULT_NESTING_DEPTH: usize = 64;

/// Credential-minimal result of comparing one correlated text-value observation with the exact
/// already-authorized non-secret text that preceded it.
///
/// The observed page string is never retained. This value keeps only the matched command id, the
/// observed UTF-8 byte count, and whether the observed string exactly matched the caller-supplied
/// expected text. A mismatch is valid negative post-condition evidence rather than transport or
/// parser success for the preceding action.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WebDriverBiDiTextValueObservationResult {
    command_id: u64,
    observed_text_bytes: usize,
    matches_expected_text: bool,
}

impl fmt::Debug for WebDriverBiDiTextValueObservationResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiTextValueObservationResult")
            .field("command_id", &self.command_id)
            .field("observed_text_bytes", &self.observed_text_bytes)
            .field("matches_expected_text", &self.matches_expected_text)
            .finish()
    }
}

impl WebDriverBiDiTextValueObservationResult {
    /// Parse one bounded local-end response, consume its exact outstanding command, and compare
    /// the successful string RemoteValue with the already-authorized expected non-secret text.
    ///
    /// The expected text is revalidated against the same reviewed local byte and character policy
    /// used by node-bound text input before any response or correlation state is touched. Common
    /// WebDriver BiDi envelope validation and command-specific `script.EvaluateResult` projection
    /// likewise complete before correlation can be consumed. Malformed, duplicate, unsupported,
    /// or over-budget result shapes therefore leave unrelated outstanding command state intact.
    ///
    /// A correlatable top-level protocol error and a valid `script` exception consume their exact
    /// command id and return typed failures. A successful string result consumes the exact id and
    /// immediately drops the page-controlled string after computing byte count and equality. This
    /// boundary does not retry, grant browser or policy authority, retain a realm identifier, or
    /// claim success when the observed value differs from the expected text.
    pub fn parse_correlate_and_compare(
        message: &WebDriverBiDiWebSocketTextMessage,
        expected_text: &str,
        correlation: &mut WebDriverBiDiCommandCorrelation,
    ) -> Result<Self, WebDriverBiDiTextValueObservationResponseError> {
        validate_expected_text(expected_text)?;
        let envelope = WebDriverBiDiJsonEnvelope::parse(message).map_err(|source| {
            WebDriverBiDiTextValueObservationResponseError::Envelope { source }
        })?;

        match envelope.kind() {
            WebDriverBiDiJsonEnvelopeKind::Event => {
                Err(WebDriverBiDiTextValueObservationResponseError::UnexpectedEvent)
            }
            WebDriverBiDiJsonEnvelopeKind::Error => {
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| {
                        WebDriverBiDiTextValueObservationResponseError::Correlation { source }
                    })?;
                Err(
                    WebDriverBiDiTextValueObservationResponseError::RemoteProtocolError {
                        command_id: completed.command_id(),
                    },
                )
            }
            WebDriverBiDiJsonEnvelopeKind::Success => {
                let projection = project_script_result(message.as_str()).map_err(|source| {
                    WebDriverBiDiTextValueObservationResponseError::Projection { source }
                })?;
                let completed = correlation
                    .correlate_response(&envelope)
                    .map_err(|source| {
                        WebDriverBiDiTextValueObservationResponseError::Correlation { source }
                    })?;
                match projection {
                    ScriptResultProjection::Exception => Err(
                        WebDriverBiDiTextValueObservationResponseError::ScriptException {
                            command_id: completed.command_id(),
                        },
                    ),
                    ScriptResultProjection::String(observed_text) => {
                        let observed_text_bytes = observed_text.len();
                        let matches_expected_text = observed_text == expected_text;
                        Ok(Self {
                            command_id: completed.command_id(),
                            observed_text_bytes,
                            matches_expected_text,
                        })
                    }
                }
            }
        }
    }

    /// Return the exact local command identifier consumed by this observation response.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.command_id
    }

    /// Return the UTF-8 byte length of the observed string without retaining the string itself.
    #[must_use]
    pub const fn observed_text_bytes(&self) -> usize {
        self.observed_text_bytes
    }

    /// Return whether the observed page value exactly matched the expected non-secret text.
    #[must_use]
    pub const fn matches_expected_text(&self) -> bool {
        self.matches_expected_text
    }
}

/// Fail-closed failures while admitting and comparing one text-value post-condition response.
#[derive(Debug)]
pub enum WebDriverBiDiTextValueObservationResponseError {
    /// The caller supplied an empty expected text value.
    EmptyExpectedText,
    /// The caller supplied expected text above the reviewed text-input byte budget.
    ExpectedTextTooLong,
    /// The expected text contains a disallowed control, whitespace, or reviewed format character.
    InvalidExpectedText,
    /// Common bounded WebDriver BiDi local-end envelope validation failed.
    Envelope {
        /// Exact common-envelope validation failure.
        source: WebDriverBiDiJsonEnvelopeError,
    },
    /// A WebDriver BiDi event was supplied where this command-specific response boundary requires
    /// a correlated success or error response.
    UnexpectedEvent,
    /// The command-specific `script.EvaluateResult` shape was malformed or unsupported.
    Projection {
        /// Exact non-sensitive projection failure.
        source: WebDriverBiDiTextValueObservationProjectionError,
    },
    /// Exact command-response correlation failed without consuming unrelated state.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// The remote end returned a correlatable top-level WebDriver BiDi protocol error.
    RemoteProtocolError {
        /// Exact local command identifier consumed by the protocol-error response.
        command_id: u64,
    },
    /// The remote end completed `script.callFunction` with a typed script exception.
    ScriptException {
        /// Exact local command identifier consumed by the script-exception result.
        command_id: u64,
    },
}

impl fmt::Display for WebDriverBiDiTextValueObservationResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyExpectedText => "expected text-value postcondition must not be empty",
            Self::ExpectedTextTooLong => {
                "expected text-value postcondition exceeds the local byte budget"
            }
            Self::InvalidExpectedText => {
                "expected text-value postcondition contains a disallowed character"
            }
            Self::Envelope { .. } => "WebDriver BiDi text-value observation envelope is invalid",
            Self::UnexpectedEvent => {
                "WebDriver BiDi text-value observation received an event instead of a command response"
            }
            Self::Projection { .. } => "WebDriver BiDi text-value observation result is invalid",
            Self::Correlation { .. } => {
                "WebDriver BiDi text-value observation response correlation failed"
            }
            Self::RemoteProtocolError { .. } => {
                "WebDriver BiDi text-value observation returned a protocol error"
            }
            Self::ScriptException { .. } => {
                "WebDriver BiDi text-value observation returned a script exception"
            }
        })
    }
}

impl Error for WebDriverBiDiTextValueObservationResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Envelope { source } => Some(source),
            Self::Projection { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::EmptyExpectedText
            | Self::ExpectedTextTooLong
            | Self::InvalidExpectedText
            | Self::UnexpectedEvent
            | Self::RemoteProtocolError { .. }
            | Self::ScriptException { .. } => None,
        }
    }
}

/// Non-sensitive structural failures while projecting a successful `script.callFunction` result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiTextValueObservationProjectionError {
    /// An object required by the command-specific result contract was absent or malformed.
    InvalidObject {
        /// Stable non-sensitive protocol member path.
        member: &'static str,
    },
    /// A required command-specific member was absent.
    MissingMember {
        /// Stable non-sensitive protocol member path.
        member: &'static str,
    },
    /// A command-specific object repeated a member name.
    DuplicateMember,
    /// A command-specific object exceeded the reviewed member-count budget.
    TooManyMembers,
    /// A command-specific member name exceeded the reviewed byte budget.
    MemberNameTooLong,
    /// A JSON string could not be projected without violating the bounded decoder contract.
    InvalidString,
    /// The `script.EvaluateResult` discriminator was neither `success` nor `exception`.
    UnsupportedScriptResultType,
    /// A successful script RemoteValue was not a string value.
    UnsupportedRemoteValueType,
    /// The observed page string exceeded the reviewed text-input byte budget.
    ObservedTextTooLong,
    /// A nested value exceeded the reviewed command-specific projection depth budget.
    NestingTooDeep,
    /// A command-specific value ended unexpectedly despite prior common-envelope validation.
    InvalidValue,
}

impl fmt::Display for WebDriverBiDiTextValueObservationProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject { member } => write!(formatter, "invalid object at {member}"),
            Self::MissingMember { member } => write!(formatter, "missing member {member}"),
            Self::DuplicateMember => {
                formatter.write_str("duplicate command-specific result member")
            }
            Self::TooManyMembers => {
                formatter.write_str("command-specific result object has too many members")
            }
            Self::MemberNameTooLong => {
                formatter.write_str("command-specific result member name is too long")
            }
            Self::InvalidString => formatter.write_str("invalid command-specific JSON string"),
            Self::UnsupportedScriptResultType => {
                formatter.write_str("unsupported script result type")
            }
            Self::UnsupportedRemoteValueType => {
                formatter.write_str("text-value observation did not return a string RemoteValue")
            }
            Self::ObservedTextTooLong => formatter
                .write_str("observed text-value postcondition exceeds the local byte budget"),
            Self::NestingTooDeep => {
                formatter.write_str("command-specific result nesting is too deep")
            }
            Self::InvalidValue => formatter.write_str("invalid command-specific result value"),
        }
    }
}

impl Error for WebDriverBiDiTextValueObservationProjectionError {}

fn validate_expected_text(
    expected_text: &str,
) -> Result<(), WebDriverBiDiTextValueObservationResponseError> {
    if expected_text.is_empty() {
        return Err(WebDriverBiDiTextValueObservationResponseError::EmptyExpectedText);
    }
    if expected_text.len() > MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES {
        return Err(WebDriverBiDiTextValueObservationResponseError::ExpectedTextTooLong);
    }
    if expected_text.chars().any(|character| {
        (character.is_whitespace() && character != ' ')
            || character.is_control()
            || UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS.contains(&character)
    }) {
        return Err(WebDriverBiDiTextValueObservationResponseError::InvalidExpectedText);
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum ScriptResultProjection {
    String(String),
    Exception,
}

fn project_script_result(
    text: &str,
) -> Result<ScriptResultProjection, WebDriverBiDiTextValueObservationProjectionError> {
    let top_level = parse_object_members(text, "response")?;
    let script_result = required_object_member(&top_level, "result", "result")?;
    let script_members = parse_object_members(script_result, "result")?;
    let result_type = required_string_member(&script_members, "type", "result.type")?;
    let _realm = required_string_member(&script_members, "realm", "result.realm")?;

    match result_type.as_str() {
        "success" => {
            let remote = required_object_member(&script_members, "result", "result.result")?;
            let remote_members = parse_object_members(remote, "result.result")?;
            let remote_type =
                required_string_member(&remote_members, "type", "result.result.type")?;
            if remote_type != "string" {
                return Err(
                    WebDriverBiDiTextValueObservationProjectionError::UnsupportedRemoteValueType,
                );
            }
            let observed = required_string_member(&remote_members, "value", "result.result.value")?;
            if observed.len() > MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES {
                return Err(WebDriverBiDiTextValueObservationProjectionError::ObservedTextTooLong);
            }
            Ok(ScriptResultProjection::String(observed))
        }
        "exception" => {
            let _details = required_object_member(
                &script_members,
                "exceptionDetails",
                "result.exceptionDetails",
            )?;
            Ok(ScriptResultProjection::Exception)
        }
        _ => Err(WebDriverBiDiTextValueObservationProjectionError::UnsupportedScriptResultType),
    }
}

fn required_object_member<'a>(
    members: &'a [(String, &'a str)],
    name: &str,
    path: &'static str,
) -> Result<&'a str, WebDriverBiDiTextValueObservationProjectionError> {
    let value = members
        .iter()
        .find_map(|(member, value)| (member == name).then_some(*value))
        .ok_or(WebDriverBiDiTextValueObservationProjectionError::MissingMember { member: path })?;
    let trimmed = value.trim();
    if !trimmed.starts_with('{') {
        return Err(
            WebDriverBiDiTextValueObservationProjectionError::InvalidObject { member: path },
        );
    }
    Ok(trimmed)
}

fn required_string_member(
    members: &[(String, &str)],
    name: &str,
    path: &'static str,
) -> Result<String, WebDriverBiDiTextValueObservationProjectionError> {
    let value = members
        .iter()
        .find_map(|(member, value)| (member == name).then_some(*value))
        .ok_or(WebDriverBiDiTextValueObservationProjectionError::MissingMember { member: path })?;
    decode_json_string(value.trim())
}

fn parse_object_members<'a>(
    text: &'a str,
    path: &'static str,
) -> Result<Vec<(String, &'a str)>, WebDriverBiDiTextValueObservationProjectionError> {
    let bytes = text.as_bytes();
    let mut index = skip_whitespace(bytes, 0);
    if bytes.get(index) != Some(&b'{') {
        return Err(
            WebDriverBiDiTextValueObservationProjectionError::InvalidObject { member: path },
        );
    }
    index += 1;
    let mut members = Vec::new();
    let mut names = HashSet::new();

    loop {
        index = skip_whitespace(bytes, index);
        if bytes.get(index) == Some(&b'}') {
            index += 1;
            index = skip_whitespace(bytes, index);
            if index != bytes.len() {
                return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue);
            }
            return Ok(members);
        }
        if members.len() >= MAX_SCRIPT_RESULT_OBJECT_MEMBERS {
            return Err(WebDriverBiDiTextValueObservationProjectionError::TooManyMembers);
        }
        let key_end = scan_string_end(bytes, index)?;
        let key = decode_json_string(&text[index..key_end])?;
        if key.len() > MAX_SCRIPT_RESULT_MEMBER_NAME_BYTES {
            return Err(WebDriverBiDiTextValueObservationProjectionError::MemberNameTooLong);
        }
        if !names.insert(key.clone()) {
            return Err(WebDriverBiDiTextValueObservationProjectionError::DuplicateMember);
        }
        index = skip_whitespace(bytes, key_end);
        if bytes.get(index) != Some(&b':') {
            return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue);
        }
        index += 1;
        index = skip_whitespace(bytes, index);
        let value_start = index;
        let value_end = scan_value_end(bytes, value_start, 0)?;
        members.push((key, &text[value_start..value_end]));
        index = skip_whitespace(bytes, value_end);
        match bytes.get(index) {
            Some(b',') => index += 1,
            Some(b'}') => {}
            _ => return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue),
        }
    }
}

fn skip_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        index += 1;
    }
    index
}

fn scan_value_end(
    bytes: &[u8],
    index: usize,
    depth: usize,
) -> Result<usize, WebDriverBiDiTextValueObservationProjectionError> {
    if depth >= MAX_SCRIPT_RESULT_NESTING_DEPTH {
        return Err(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep);
    }
    match bytes.get(index) {
        Some(b'"') => scan_string_end(bytes, index),
        Some(b'{') => scan_container_end(bytes, index, b'{', b'}', depth + 1),
        Some(b'[') => scan_container_end(bytes, index, b'[', b']', depth + 1),
        Some(_) => {
            let mut end = index;
            while let Some(byte) = bytes.get(end) {
                if matches!(byte, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r') {
                    break;
                }
                end += 1;
            }
            if end == index {
                Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
            } else {
                Ok(end)
            }
        }
        None => Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue),
    }
}

fn scan_container_end(
    bytes: &[u8],
    start: usize,
    open: u8,
    close: u8,
    depth: usize,
) -> Result<usize, WebDriverBiDiTextValueObservationProjectionError> {
    if depth > MAX_SCRIPT_RESULT_NESTING_DEPTH {
        return Err(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep);
    }
    if bytes.get(start) != Some(&open) {
        return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue);
    }
    let mut stack = vec![close];
    let mut index = start + 1;
    while let Some(byte) = bytes.get(index).copied() {
        match byte {
            b'"' => index = scan_string_end(bytes, index)?,
            b'{' => {
                if stack.len() >= MAX_SCRIPT_RESULT_NESTING_DEPTH {
                    return Err(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep);
                }
                stack.push(b'}');
                index += 1;
            }
            b'[' => {
                if stack.len() >= MAX_SCRIPT_RESULT_NESTING_DEPTH {
                    return Err(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep);
                }
                stack.push(b']');
                index += 1;
            }
            b'}' | b']' => {
                if stack.pop() != Some(byte) {
                    return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue);
                }
                index += 1;
                if stack.is_empty() {
                    return Ok(index);
                }
            }
            _ => index += 1,
        }
    }
    Err(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
}

fn scan_string_end(
    bytes: &[u8],
    start: usize,
) -> Result<usize, WebDriverBiDiTextValueObservationProjectionError> {
    if bytes.get(start) != Some(&b'"') {
        return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
    }
    let mut index = start + 1;
    while let Some(byte) = bytes.get(index).copied() {
        match byte {
            b'"' => return Ok(index + 1),
            b'\\' => {
                index += 2;
                if index > bytes.len() {
                    return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
                }
            }
            0x00..=0x1f => {
                return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
            }
            _ => index += 1,
        }
    }
    Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
}

fn decode_json_string(
    value: &str,
) -> Result<String, WebDriverBiDiTextValueObservationProjectionError> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
    }
    let inner = &value[1..value.len() - 1];
    let mut characters = inner.chars();
    let mut output = String::new();
    while let Some(character) = characters.next() {
        if character != '\\' {
            if character.is_control() {
                return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
            }
            output.push(character);
            continue;
        }
        let escaped = characters
            .next()
            .ok_or(WebDriverBiDiTextValueObservationProjectionError::InvalidString)?;
        match escaped {
            '"' => output.push('"'),
            '\\' => output.push('\\'),
            '/' => output.push('/'),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000c}'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'u' => {
                let first = decode_hex_quad(&mut characters)?;
                if (0xd800..=0xdbff).contains(&first) {
                    if characters.next() != Some('\\') || characters.next() != Some('u') {
                        return Err(
                            WebDriverBiDiTextValueObservationProjectionError::InvalidString,
                        );
                    }
                    let second = decode_hex_quad(&mut characters)?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(
                            WebDriverBiDiTextValueObservationProjectionError::InvalidString,
                        );
                    }
                    let units = [first, second];
                    output.push_str(&String::from_utf16_lossy(&units));
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString);
                } else {
                    let units = [first];
                    output.push_str(&String::from_utf16_lossy(&units));
                }
            }
            _ => return Err(WebDriverBiDiTextValueObservationProjectionError::InvalidString),
        }
    }
    Ok(output)
}

fn decode_hex_quad(
    characters: &mut std::str::Chars<'_>,
) -> Result<u16, WebDriverBiDiTextValueObservationProjectionError> {
    let mut value = 0_u16;
    for _ in 0..4 {
        let digit = characters
            .next()
            .and_then(|character| character.to_digit(16))
            .ok_or(WebDriverBiDiTextValueObservationProjectionError::InvalidString)?;
        value = (value << 4) | digit as u16;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_accepts_extensions_and_decodes_string_escapes() {
        let response = r#"{"type":"success","id":7,"vendor":true,"result":{"realm":"realm-1","type":"success","vendor":{"nested":[1,true,null]},"result":{"value":"A\"B\\C\/D\b\f\n\r\t\u20ac\ud83d\ude00","type":"string","vendor":0}}}"#;
        assert_eq!(
            project_script_result(response),
            Ok(ScriptResultProjection::String(
                "A\"B\\C/D\u{0008}\u{000c}\n\r\t€😀".to_owned(),
            ))
        );
    }

    #[test]
    fn projection_accepts_typed_script_exception_without_retaining_details() {
        let response = r#"{"type":"success","id":8,"result":{"type":"exception","realm":"realm-1","exceptionDetails":{"text":"page-secret","columnNumber":1,"lineNumber":1,"stackTrace":{"callFrames":[]}}}}"#;
        assert_eq!(
            project_script_result(response),
            Ok(ScriptResultProjection::Exception)
        );
    }

    #[test]
    fn projection_rejects_unsupported_and_oversized_remote_values() {
        let wrong_type = r#"{"type":"success","id":9,"result":{"type":"success","realm":"realm-1","result":{"type":"number","value":1}}}"#;
        assert_eq!(
            project_script_result(wrong_type).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::UnsupportedRemoteValueType)
        );

        let oversized = "x".repeat(MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES + 1);
        let response = format!(
            "{{\"type\":\"success\",\"id\":10,\"result\":{{\"type\":\"success\",\"realm\":\"realm-1\",\"result\":{{\"type\":\"string\",\"value\":\"{oversized}\"}}}}}}"
        );
        assert_eq!(
            project_script_result(&response).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::ObservedTextTooLong)
        );
    }

    #[test]
    fn projection_rejects_missing_duplicate_and_invalid_members() {
        let missing = r#"{"type":"success","id":11,"result":{"type":"success","realm":"realm-1"}}"#;
        assert_eq!(
            project_script_result(missing).err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::MissingMember {
                    member: "result.result"
                }
            )
        );

        let duplicate = r#"{"type":"success","id":12,"result":{"type":"success","type":"success","realm":"realm-1","result":{"type":"string","value":"x"}}}"#;
        assert_eq!(
            project_script_result(duplicate).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::DuplicateMember)
        );

        let invalid_object = r#"{"type":"success","id":13,"result":false}"#;
        assert_eq!(
            project_script_result(invalid_object).err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::InvalidObject {
                    member: "result"
                }
            )
        );

        let unsupported =
            r#"{"type":"success","id":14,"result":{"type":"future","realm":"realm-1"}}"#;
        assert_eq!(
            project_script_result(unsupported).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::UnsupportedScriptResultType)
        );
    }

    #[test]
    fn projection_helpers_reject_invalid_strings_values_and_resource_exhaustion() {
        assert_eq!(
            decode_json_string("not-a-string").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\uDC00""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\uD800x""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\uD800\x""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\uD800\u12xz""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\uD800\u0041""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\q""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string(r#""\u12xz""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        assert_eq!(
            parse_object_members("[]", "root").err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::InvalidObject { member: "root" }
            )
        );
        assert_eq!(
            parse_object_members("{\"a\":1} trailing", "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            scan_value_end(b"", 0, 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            scan_value_end(b"x", 0, MAX_SCRIPT_RESULT_NESTING_DEPTH).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep)
        );
        assert_eq!(
            scan_container_end(b"[]", 0, b'{', b'}', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            scan_string_end(b"x", 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
    }

    #[test]
    fn expected_text_validation_covers_budget_and_injection_policy() {
        assert_eq!(
            validate_expected_text("")
                .err()
                .map(|error| error.to_string())
                .as_deref(),
            Some("expected text-value postcondition must not be empty")
        );
        let oversized = "x".repeat(MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES + 1);
        assert_eq!(
            validate_expected_text(&oversized)
                .err()
                .map(|error| error.to_string())
                .as_deref(),
            Some("expected text-value postcondition exceeds the local byte budget")
        );
        assert_eq!(validate_expected_text("ordinary space").is_ok(), true);
        for rejected in ["tab\tvalue", "control\u{0001}", "bidi\u{202e}override"] {
            assert_eq!(
                validate_expected_text(rejected)
                    .err()
                    .map(|error| error.to_string())
                    .as_deref(),
                Some("expected text-value postcondition contains a disallowed character")
            );
        }
    }

    #[test]
    fn projection_helpers_cover_structural_and_terminal_failures() {
        let missing_exception_details =
            r#"{"type":"success","id":15,"result":{"type":"exception","realm":"realm-1"}}"#;
        assert_eq!(
            project_script_result(missing_exception_details).err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::MissingMember {
                    member: "result.exceptionDetails"
                }
            )
        );
        let invalid_exception_details = r#"{"type":"success","id":16,"result":{"type":"exception","realm":"realm-1","exceptionDetails":false}}"#;
        assert_eq!(
            project_script_result(invalid_exception_details).err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::InvalidObject {
                    member: "result.exceptionDetails"
                }
            )
        );

        let many_members = (0..=MAX_SCRIPT_RESULT_OBJECT_MEMBERS)
            .map(|index| format!("\"k{index}\":0"))
            .collect::<Vec<_>>()
            .join(",");
        let too_many = format!("{{{many_members}}}");
        assert_eq!(
            parse_object_members(&too_many, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::TooManyMembers)
        );

        let long_name = "k".repeat(MAX_SCRIPT_RESULT_MEMBER_NAME_BYTES + 1);
        let long_member = format!("{{\"{long_name}\":0}}");
        assert_eq!(
            parse_object_members(&long_member, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::MemberNameTooLong)
        );
        assert_eq!(
            parse_object_members(r#"{"a" 1}"#, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            parse_object_members(r#"{"a":1]"#, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );

        assert_eq!(scan_value_end(b"[]", 0, 0), Ok(2));
        assert_eq!(scan_value_end(b"terminal", 0, 0), Ok(b"terminal".len()));
        assert_eq!(
            scan_value_end(b",", 0, 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            scan_container_end(b"{}", 0, b'{', b'}', MAX_SCRIPT_RESULT_NESTING_DEPTH + 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep)
        );

        let deep_objects = "{".repeat(MAX_SCRIPT_RESULT_NESTING_DEPTH + 1);
        assert_eq!(
            scan_container_end(deep_objects.as_bytes(), 0, b'{', b'}', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep)
        );
        let deep_arrays = "[".repeat(MAX_SCRIPT_RESULT_NESTING_DEPTH + 1);
        assert_eq!(
            scan_container_end(deep_arrays.as_bytes(), 0, b'[', b']', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep)
        );
        assert_eq!(
            scan_container_end(b"{]", 0, b'{', b'}', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );
        assert_eq!(
            scan_container_end(b"{", 0, b'{', b'}', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );

        assert_eq!(
            scan_string_end(b"\"\\", 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            scan_string_end(b"\"\x01\"", 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            scan_string_end(b"\"x", 0).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        assert_eq!(
            decode_json_string("\"").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            decode_json_string("\"x").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        let raw_control = format!("\"{}\"", '\u{0001}');
        assert_eq!(
            decode_json_string(&raw_control).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(decode_json_string(r#""\uD83D\uDE00""#).as_deref(), Ok("😀"));
        assert_eq!(decode_json_string(r#""\u20AC""#).as_deref(), Ok("€"));
        assert_eq!(
            decode_json_string(r#""\u12""#).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
    }

    #[test]
    fn projection_propagates_malformed_nested_json_without_consuming_detail() {
        assert_eq!(
            project_script_result("[]").err(),
            Some(
                WebDriverBiDiTextValueObservationProjectionError::InvalidObject {
                    member: "response"
                }
            )
        );

        let invalid_result_type =
            r#"{"type":"success","id":17,"result":{"type":"\q","realm":"realm-1"}}"#;
        assert_eq!(
            project_script_result(invalid_result_type).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        let malformed_remote = r#"{"type":"success","id":18,"result":{"type":"success","realm":"realm-1","result":{"type" 1}}}"#;
        assert_eq!(
            project_script_result(malformed_remote).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );

        let invalid_remote_type = r#"{"type":"success","id":19,"result":{"type":"success","realm":"realm-1","result":{"type":"\q","value":"x"}}}"#;
        assert_eq!(
            project_script_result(invalid_remote_type).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        let invalid_remote_value = r#"{"type":"success","id":20,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"\q"}}}"#;
        assert_eq!(
            project_script_result(invalid_remote_value).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        assert_eq!(
            parse_object_members(r#"{"unterminated}"#, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            parse_object_members(r#"{"\q":1}"#, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
        assert_eq!(
            parse_object_members(r#"{"a":"#, "root").err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidValue)
        );

        assert_eq!(
            scan_container_end(b"{\"unterminated", 0, b'{', b'}', 1).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );

        let dangling_escape_json_string = format!("{}{}{}", '"', '\\', '"');
        assert_eq!(
            decode_json_string(&dangling_escape_json_string).err(),
            Some(WebDriverBiDiTextValueObservationProjectionError::InvalidString)
        );
    }

    #[test]
    fn response_and_projection_errors_are_stable_and_non_sensitive() {
        let cases: Vec<WebDriverBiDiTextValueObservationResponseError> = vec![
            WebDriverBiDiTextValueObservationResponseError::EmptyExpectedText,
            WebDriverBiDiTextValueObservationResponseError::ExpectedTextTooLong,
            WebDriverBiDiTextValueObservationResponseError::InvalidExpectedText,
            WebDriverBiDiTextValueObservationResponseError::UnexpectedEvent,
            WebDriverBiDiTextValueObservationResponseError::RemoteProtocolError { command_id: 7 },
            WebDriverBiDiTextValueObservationResponseError::ScriptException { command_id: 8 },
        ];
        for error in cases {
            assert_eq!(error.source().is_none(), true);
            assert_eq!(error.to_string().is_empty(), false);
        }

        let envelope = WebDriverBiDiTextValueObservationResponseError::Envelope {
            source: WebDriverBiDiJsonEnvelopeError::InvalidJson,
        };
        assert_eq!(envelope.source().is_some(), true);
        let projection = WebDriverBiDiTextValueObservationResponseError::Projection {
            source: WebDriverBiDiTextValueObservationProjectionError::InvalidValue,
        };
        assert_eq!(projection.source().is_some(), true);
        let correlation = WebDriverBiDiTextValueObservationResponseError::Correlation {
            source: WebDriverBiDiCommandCorrelationError::CommandNotOutstanding,
        };
        assert_eq!(correlation.source().is_some(), true);

        let projection_errors = [
            WebDriverBiDiTextValueObservationProjectionError::InvalidObject { member: "result" },
            WebDriverBiDiTextValueObservationProjectionError::MissingMember { member: "result" },
            WebDriverBiDiTextValueObservationProjectionError::DuplicateMember,
            WebDriverBiDiTextValueObservationProjectionError::TooManyMembers,
            WebDriverBiDiTextValueObservationProjectionError::MemberNameTooLong,
            WebDriverBiDiTextValueObservationProjectionError::InvalidString,
            WebDriverBiDiTextValueObservationProjectionError::UnsupportedScriptResultType,
            WebDriverBiDiTextValueObservationProjectionError::UnsupportedRemoteValueType,
            WebDriverBiDiTextValueObservationProjectionError::ObservedTextTooLong,
            WebDriverBiDiTextValueObservationProjectionError::NestingTooDeep,
            WebDriverBiDiTextValueObservationProjectionError::InvalidValue,
        ];
        for error in projection_errors {
            assert_eq!(error.to_string().is_empty(), false);
        }
    }
}
