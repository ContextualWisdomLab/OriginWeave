//! Conservative supplied and observed MIME classification without execution.

use std::collections::BTreeSet;

use crate::field::{FieldBlock, is_token_byte};
use crate::{HttpError, MAX_MIME_SNIFF_BYTES};

/// Version of the conservative byte-signature table used by OriginWeave.
pub const MIME_CLASSIFIER_VERSION: &str = "originweave-mime-signatures-1";

/// One syntactically validated MIME type and bounded parameter set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    type_name: String,
    subtype_name: String,
    parameters: Vec<(String, String)>,
}

impl MimeType {
    /// Parse one strict ASCII MIME type value.
    pub fn parse(input: &[u8]) -> Result<Self, HttpError> {
        let segments = split_semicolon_segments(input)?;
        let essence = trim_optional_whitespace(segments[0]);
        let slash = essence
            .iter()
            .position(|byte| *byte == b'/')
            .ok_or(HttpError::InvalidMimeType)?;
        let type_name = &essence[..slash];
        let subtype_name = &essence[slash + 1..];
        // A second slash is already rejected by `is_token_byte`, because `/` is not a MIME
        // token byte. Keeping a separate `contains('/')` condition would duplicate the same
        // grammar invariant and create an impossible coverage branch.
        if type_name.is_empty()
            || subtype_name.is_empty()
            || !type_name.iter().copied().all(is_token_byte)
            || !subtype_name.iter().copied().all(is_token_byte)
        {
            return Err(HttpError::InvalidMimeType);
        }
        let mut seen = BTreeSet::new();
        let mut parameters = Vec::new();
        for segment in &segments[1..] {
            let segment = trim_optional_whitespace(segment);
            let equals = segment
                .iter()
                .position(|byte| *byte == b'=')
                .ok_or(HttpError::InvalidMimeType)?;
            let name = trim_optional_whitespace(&segment[..equals]);
            let value = trim_optional_whitespace(&segment[equals + 1..]);
            if name.is_empty() || !name.iter().copied().all(is_token_byte) || value.is_empty() {
                return Err(HttpError::InvalidMimeType);
            }
            // `is_token_byte` guarantees ASCII, so lowercase normalization cannot fail.
            let name = ascii_lowercase(name);
            if !seen.insert(name.clone()) {
                return Err(HttpError::InvalidMimeType);
            }
            let value = parse_parameter_value(value)?;
            parameters.push((name, value));
        }
        Ok(Self {
            type_name: ascii_lowercase(type_name),
            subtype_name: ascii_lowercase(subtype_name),
            parameters,
        })
    }

    /// Construct one parameter-free MIME type from reviewed ASCII tokens.
    pub fn from_essence(type_name: &str, subtype_name: &str) -> Result<Self, HttpError> {
        let parsed = Self::parse(format!("{type_name}/{subtype_name}").as_bytes())?;
        if parsed.parameters.is_empty() {
            Ok(parsed)
        } else {
            Err(HttpError::InvalidMimeType)
        }
    }

    /// Return the lowercase top-level type token.
    #[must_use]
    pub const fn type_name(&self) -> &str {
        self.type_name.as_str()
    }

    /// Return the lowercase subtype token.
    #[must_use]
    pub const fn subtype_name(&self) -> &str {
        self.subtype_name.as_str()
    }

    /// Return the validated ordered parameter list.
    #[must_use]
    pub const fn parameters(&self) -> &[(String, String)] {
        self.parameters.as_slice()
    }

    /// Return whether two MIME values have the same type/subtype essence.
    #[must_use]
    pub fn same_essence(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.subtype_name == other.subtype_name
    }

    /// Return the lowercase `type/subtype` essence.
    #[must_use]
    pub fn essence(&self) -> String {
        format!("{}/{}", self.type_name, self.subtype_name)
    }
}

/// Broad downstream risk class inferred without rendering or executing content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentRiskClass {
    /// Passive text or image content under the reviewed signature table.
    Passive,
    /// HTML, XML, SVG, JavaScript, or PDF requiring a separate active-content policy.
    ActiveOrScriptable,
    /// An archive or container requiring a separate extraction authority.
    ArchiveOrContainer,
    /// Binary content that could not be safely classified.
    UnknownBinary,
}

/// Relationship between supplied MIME metadata and the observed byte classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimeMismatch {
    /// Supplied and observed type/subtype essences agree.
    Match,
    /// Supplied and observed type/subtype essences disagree.
    Mismatch,
    /// Only supplied metadata was informative; observed bytes fell back to binary.
    SuppliedOnly,
    /// No supplied metadata existed and the observed bytes were informative.
    ObservedOnly,
    /// Neither channel provided a more specific classification than binary fallback.
    BinaryFallback,
}

/// Explicit `X-Content-Type-Options` result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoSniffStatus {
    /// No no-sniff instruction was supplied.
    Absent,
    /// Every supplied value was the `nosniff` token.
    Enabled,
}

/// One versioned conservative classification from a bounded content prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedMimeClassification {
    mime_type: MimeType,
    risk_class: ContentRiskClass,
    classifier_version: &'static str,
}

impl ObservedMimeClassification {
    /// Return the observed MIME type.
    #[must_use]
    pub const fn mime_type(&self) -> &MimeType {
        &self.mime_type
    }

    /// Return the downstream risk class.
    #[must_use]
    pub const fn risk_class(&self) -> ContentRiskClass {
        self.risk_class
    }

    /// Return the immutable signature-table version.
    #[must_use]
    pub const fn classifier_version(&self) -> &'static str {
        self.classifier_version
    }
}

pub(crate) fn supplied_mime_type(fields: &FieldBlock) -> Result<Option<MimeType>, HttpError> {
    let values = fields.values("content-type");
    match values.as_slice() {
        [] => Ok(None),
        [value] => MimeType::parse(value).map(Some),
        _multiple => Err(HttpError::InvalidMimeType),
    }
}

pub(crate) fn no_sniff_status(fields: &FieldBlock) -> Result<NoSniffStatus, HttpError> {
    let values = fields.values("x-content-type-options");
    if values.is_empty() {
        return Ok(NoSniffStatus::Absent);
    }
    if values
        .iter()
        .all(|value| trim_optional_whitespace(value).eq_ignore_ascii_case(b"nosniff"))
    {
        Ok(NoSniffStatus::Enabled)
    } else {
        Err(HttpError::InvalidMimeType)
    }
}

pub(crate) fn classify_observed_mime(
    content: &[u8],
    supplied: Option<&MimeType>,
) -> ObservedMimeClassification {
    let prefix = &content[..content.len().min(MAX_MIME_SNIFF_BYTES)];
    let trimmed = trim_text_prefix(prefix);
    // Every classifier result below is an internal reviewed ASCII token pair. Constructing those
    // literals through the public fallible parser would add impossible error regions without
    // increasing validation: untrusted supplied metadata still uses `MimeType::parse` above.
    let mime_type = if prefix.starts_with(b"%PDF-") {
        internal_mime("application", "pdf")
    } else if is_png(prefix) {
        internal_mime("image", "png")
    } else if prefix.starts_with(&[0xff, 0xd8, 0xff]) {
        internal_mime("image", "jpeg")
    } else if prefix.starts_with(b"GIF87a") || prefix.starts_with(b"GIF89a") {
        internal_mime("image", "gif")
    } else if is_webp(prefix) {
        internal_mime("image", "webp")
    } else if is_zip(prefix) {
        internal_mime("application", "zip")
    } else if starts_ascii_case_insensitive(trimmed, b"<svg") {
        internal_mime("image", "svg+xml")
    } else if starts_ascii_case_insensitive(trimmed, b"<?xml") {
        internal_mime("application", "xml")
    } else if looks_like_html(trimmed) {
        internal_mime("text", "html")
    } else if supplied.is_some_and(is_javascript_mime) && is_plain_text(prefix) {
        internal_mime("text", "javascript")
    } else if is_plain_text(prefix) {
        internal_mime("text", "plain")
    } else {
        internal_mime("application", "octet-stream")
    };
    let risk_class = risk_class(&mime_type);
    ObservedMimeClassification {
        mime_type,
        risk_class,
        classifier_version: MIME_CLASSIFIER_VERSION,
    }
}

pub(crate) fn classify_mismatch(
    supplied: Option<&MimeType>,
    observed: &ObservedMimeClassification,
) -> MimeMismatch {
    let observed_is_binary = observed.mime_type.type_name() == "application"
        && observed.mime_type.subtype_name() == "octet-stream";
    match supplied {
        Some(supplied) if supplied.same_essence(observed.mime_type()) => MimeMismatch::Match,
        Some(_supplied) if observed_is_binary => MimeMismatch::SuppliedOnly,
        Some(_supplied) => MimeMismatch::Mismatch,
        None if observed_is_binary => MimeMismatch::BinaryFallback,
        None => MimeMismatch::ObservedOnly,
    }
}

fn split_semicolon_segments(input: &[u8]) -> Result<Vec<&[u8]>, HttpError> {
    let mut segments = Vec::new();
    let mut segment_start = 0_usize;
    let mut in_quote = false;
    let mut escaped = false;
    for (index, byte) in input.iter().copied().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_quote && byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == b'"' {
            in_quote = !in_quote;
            continue;
        }
        if byte == b';' && !in_quote {
            segments.push(&input[segment_start..index]);
            segment_start = index + 1;
        }
    }
    // `escaped` is only set while `in_quote` is true; therefore an unfinished escape is already
    // represented by the unfinished quoted-string state and does not need a second condition.
    if in_quote {
        return Err(HttpError::InvalidMimeType);
    }
    // This final segment exists even for empty input, so callers can reject an empty essence
    // without maintaining an unreachable `segments.is_empty()` branch here.
    segments.push(&input[segment_start..]);
    Ok(segments)
}

fn parse_parameter_value(value: &[u8]) -> Result<String, HttpError> {
    if value.first() == Some(&b'"') {
        // `split_semicolon_segments` has already rejected any unterminated opening quote, so a
        // quoted parameter cannot be a one-byte value at this point.
        if value.last() != Some(&b'"') {
            return Err(HttpError::InvalidMimeType);
        }
        let mut decoded = Vec::with_capacity(value.len() - 2);
        let mut escaped = false;
        for byte in value[1..value.len() - 1].iter().copied() {
            if escaped {
                if !matches!(byte, b'\t' | 0x20..=0x7e | 0x80..=0xff) {
                    return Err(HttpError::InvalidMimeType);
                }
                decoded.push(byte);
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if matches!(byte, b'\t' | 0x20..=0x7e | 0x80..=0xff) && byte != b'"' {
                decoded.push(byte);
            } else {
                return Err(HttpError::InvalidMimeType);
            }
        }
        // The outer scanner rejects an escape that would consume the closing quote, so the loop
        // cannot finish with `escaped == true` for a value admitted here.
        String::from_utf8(decoded).map_err(|_error| HttpError::InvalidMimeType)
    } else {
        if !value.iter().copied().all(is_token_byte) {
            return Err(HttpError::InvalidMimeType);
        }
        // An admitted token is ASCII by construction, so direct character collection avoids an
        // impossible UTF-8 conversion error branch while preserving the exact bytes.
        Ok(value.iter().map(|byte| char::from(*byte)).collect())
    }
}

fn ascii_lowercase(value: &[u8]) -> String {
    value
        .iter()
        .map(|byte| char::from(byte.to_ascii_lowercase()))
        .collect()
}

fn internal_mime(type_name: &str, subtype_name: &str) -> MimeType {
    MimeType {
        type_name: type_name.to_owned(),
        subtype_name: subtype_name.to_owned(),
        parameters: Vec::new(),
    }
}

pub(crate) fn risk_class(mime_type: &MimeType) -> ContentRiskClass {
    const ACTIVE_ESSENCES: &[&str] = &[
        "text/html",
        "application/xml",
        "text/xml",
        "image/svg+xml",
        "text/javascript",
        "application/javascript",
        "application/pdf",
    ];

    let essence = mime_type.essence();
    if ACTIVE_ESSENCES.contains(&essence.as_str()) {
        ContentRiskClass::ActiveOrScriptable
    } else if essence == "application/zip" {
        ContentRiskClass::ArchiveOrContainer
    } else if essence == "application/octet-stream" {
        ContentRiskClass::UnknownBinary
    } else {
        ContentRiskClass::Passive
    }
}

fn is_javascript_mime(mime_type: &MimeType) -> bool {
    const JAVASCRIPT_ESSENCES: &[&str] = &["text/javascript", "application/javascript"];
    let essence = mime_type.essence();
    JAVASCRIPT_ESSENCES.contains(&essence.as_str())
}

fn is_png(input: &[u8]) -> bool {
    input.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
}

fn is_webp(input: &[u8]) -> bool {
    input.len() >= 12 && input.starts_with(b"RIFF") && &input[8..12] == b"WEBP"
}

fn is_zip(input: &[u8]) -> bool {
    input.starts_with(b"PK\x03\x04")
        || input.starts_with(b"PK\x05\x06")
        || input.starts_with(b"PK\x07\x08")
}

fn trim_text_prefix(input: &[u8]) -> &[u8] {
    let input = input.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(input);
    let start = input
        .iter()
        .position(|byte| !matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
        .unwrap_or(input.len());
    &input[start..]
}

fn looks_like_html(input: &[u8]) -> bool {
    const SIGNATURES: &[&[u8]] = &[
        b"<!doctype html",
        b"<html",
        b"<head",
        b"<script",
        b"<iframe",
        b"<h1",
        b"<div",
        b"<font",
        b"<table",
        b"<a",
        b"<style",
        b"<title",
        b"<b",
        b"<body",
        b"<br",
        b"<p",
        b"<!--",
    ];
    SIGNATURES
        .iter()
        .any(|signature| starts_ascii_case_insensitive(input, signature))
}

fn starts_ascii_case_insensitive(input: &[u8], prefix: &[u8]) -> bool {
    input
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn is_plain_text(input: &[u8]) -> bool {
    let valid = match std::str::from_utf8(input) {
        Ok(_text) => input.len(),
        Err(error) if error.error_len().is_none() => error.valid_up_to(),
        Err(_error) => return false,
    };
    input[..valid]
        .iter()
        .copied()
        .all(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | 0x20..=0x7e | 0x80..=0xff))
}

fn trim_optional_whitespace(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t'))
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !matches!(byte, b' ' | b'\t'))
        .map_or(start, |index| index + 1);
    &value[start..end]
}
