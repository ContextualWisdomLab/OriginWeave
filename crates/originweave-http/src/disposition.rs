//! Bounded content-disposition, safe filename, and redirect metadata parsing.

use std::collections::BTreeMap;

use originweave_core::Origin;
use sha2::{Digest, Sha256};

use crate::field::{FieldBlock, is_token_byte};
use crate::mime::{MimeType, ObservedMimeClassification};
use crate::{HttpError, MAX_SAFE_FILENAME_BYTES};

const MAX_REDIRECT_LOCATION_BYTES: usize = 8_192;
const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Supported disposition type for bounded response metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispositionKind {
    /// Content may be presented inline by a later separately authorized renderer.
    Inline,
    /// Content is proposed as an attachment for a later download authority.
    Attachment,
}

/// Relationship between a safe filename extension and observed MIME evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionMimeRelation {
    /// No filename or filename extension was present.
    Absent,
    /// A recognized extension agrees with the observed MIME essence.
    Match,
    /// A recognized extension disagrees with the observed MIME essence.
    Mismatch,
    /// The extension is syntactically safe but not recognized by the first map.
    Unknown,
}

/// Safe metadata extracted from one `Content-Disposition` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeContentDisposition {
    kind: DispositionKind,
    filename: Option<String>,
    extension_mime_relation: ExtensionMimeRelation,
}

impl SafeContentDisposition {
    /// Return the normalized disposition kind.
    #[must_use]
    pub const fn kind(&self) -> DispositionKind {
        self.kind
    }

    /// Return the validated filename metadata, when supplied.
    #[must_use]
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Return the extension-to-observed-MIME relation.
    #[must_use]
    pub const fn extension_mime_relation(&self) -> ExtensionMimeRelation {
        self.extension_mime_relation
    }
}

/// Credential-free redirect metadata that never follows the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectMetadata {
    location_hash: String,
    target_origin: Option<Origin>,
    is_relative: bool,
}

impl RedirectMetadata {
    /// Return a domain-separated SHA-256 identifier for the complete Location value.
    #[must_use]
    pub const fn location_hash(&self) -> &str {
        self.location_hash.as_str()
    }

    /// Return the canonical target origin when Location used absolute URI form.
    #[must_use]
    pub const fn target_origin(&self) -> Option<&Origin> {
        self.target_origin.as_ref()
    }

    /// Return whether the Location value was an origin-relative reference.
    #[must_use]
    pub const fn is_relative(&self) -> bool {
        self.is_relative
    }
}

pub(crate) fn parse_content_disposition(
    fields: &FieldBlock,
    observed: &ObservedMimeClassification,
) -> Result<Option<SafeContentDisposition>, HttpError> {
    let values = fields.values("content-disposition");
    let value = match values.as_slice() {
        [] => return Ok(None),
        [value] => *value,
        _multiple => return Err(HttpError::InvalidContentDisposition),
    };
    let segments = split_semicolon_segments(value)?;
    let disposition = trim_optional_whitespace(segments[0]);
    let kind = if disposition.eq_ignore_ascii_case(b"inline") {
        DispositionKind::Inline
    } else if disposition.eq_ignore_ascii_case(b"attachment") {
        DispositionKind::Attachment
    } else {
        return Err(HttpError::InvalidContentDisposition);
    };

    let mut parameters = BTreeMap::new();
    for segment in &segments[1..] {
        let segment = trim_optional_whitespace(segment);
        let equals = segment
            .iter()
            .position(|byte| *byte == b'=')
            .ok_or(HttpError::InvalidContentDisposition)?;
        let name = trim_optional_whitespace(&segment[..equals]);
        let value = trim_optional_whitespace(&segment[equals + 1..]);
        if name.is_empty() || !name.iter().copied().all(is_token_byte) || value.is_empty() {
            return Err(HttpError::InvalidContentDisposition);
        }
        // `is_token_byte` above guarantees ASCII, so lowercase normalization is infallible.
        let name = ascii_lowercase(name);
        if !matches!(name.as_str(), "filename" | "filename*")
            || parameters.insert(name, value.to_vec()).is_some()
        {
            return Err(HttpError::InvalidContentDisposition);
        }
    }

    let filename = if let Some(value) = parameters.get("filename*") {
        Some(parse_extended_filename(value)?)
    } else if let Some(value) = parameters.get("filename") {
        Some(parse_filename_value(value)?)
    } else {
        None
    };
    if let Some(filename) = filename.as_deref() {
        validate_safe_filename(filename)?;
    }
    let extension_mime_relation =
        extension_mime_relation(filename.as_deref(), observed.mime_type());
    Ok(Some(SafeContentDisposition {
        kind,
        filename,
        extension_mime_relation,
    }))
}

pub(crate) fn parse_redirect_metadata(
    status_code: u16,
    fields: &FieldBlock,
) -> Result<Option<RedirectMetadata>, HttpError> {
    if !matches!(status_code, 300 | 301 | 302 | 303 | 305 | 307 | 308) {
        return Ok(None);
    }
    let values = fields.values("location");
    let location = match values.as_slice() {
        [] => return Ok(None),
        [value] => trim_optional_whitespace(value),
        _multiple => return Err(HttpError::InvalidRedirectMetadata),
    };
    if location.is_empty()
        || location.len() > MAX_REDIRECT_LOCATION_BYTES
        || location
            .iter()
            .copied()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || byte == b'#')
    {
        return Err(HttpError::InvalidRedirectMetadata);
    }
    let location_text =
        std::str::from_utf8(location).map_err(|_error| HttpError::InvalidRedirectMetadata)?;
    let (target_origin, is_relative) = if location_text.starts_with("//") {
        // RFC 3986 §4.2 defines `//authority/path` as a network-path reference. This crate does
        // not carry the base URI needed to resolve that authority safely, so fail closed rather
        // than misrepresenting it as a same-origin absolute-path reference.
        return Err(HttpError::InvalidRedirectMetadata);
    } else if location_text.starts_with('/') {
        (None, true)
    } else {
        let origin_text = absolute_location_origin(location_text)?;
        let origin =
            Origin::parse(origin_text).map_err(|_error| HttpError::InvalidRedirectMetadata)?;
        (Some(origin), false)
    };
    Ok(Some(RedirectMetadata {
        location_hash: sha256_identifier(b"originweave-http-location-v1\0", location),
        target_origin,
        is_relative,
    }))
}

fn parse_filename_value(value: &[u8]) -> Result<String, HttpError> {
    if value.first() == Some(&b'"') {
        // `split_semicolon_segments` rejects an unterminated opening quote before this parser is
        // called. A quoted parameter therefore cannot have length one; only trailing material
        // after a balanced closing quote remains a local syntax error here.
        if value.last() != Some(&b'"') {
            return Err(HttpError::InvalidContentDisposition);
        }
        let mut decoded = Vec::with_capacity(value.len() - 2);
        let mut escaped = false;
        for byte in value[1..value.len() - 1].iter().copied() {
            if escaped {
                if byte.is_ascii_control() {
                    return Err(HttpError::InvalidContentDisposition);
                }
                decoded.push(byte);
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' || byte.is_ascii_control() {
                return Err(HttpError::InvalidContentDisposition);
            } else {
                decoded.push(byte);
            }
        }
        // The outer segment scanner rejects an escape that would consume the closing quote, so
        // `escaped` is necessarily false after iterating the quoted payload.
        String::from_utf8(decoded).map_err(|_error| HttpError::InvalidContentDisposition)
    } else {
        if !value.iter().copied().all(is_token_byte) {
            return Err(HttpError::InvalidContentDisposition);
        }
        std::str::from_utf8(value)
            .map(str::to_owned)
            .map_err(|_error| HttpError::InvalidContentDisposition)
    }
}

fn parse_extended_filename(value: &[u8]) -> Result<String, HttpError> {
    let value =
        std::str::from_utf8(value).map_err(|_error| HttpError::InvalidContentDisposition)?;
    let mut sections = value.splitn(3, '\'');
    let charset = sections
        .next()
        .ok_or(HttpError::InvalidContentDisposition)?;
    let _language = sections
        .next()
        .ok_or(HttpError::InvalidContentDisposition)?;
    let encoded = sections
        .next()
        .ok_or(HttpError::InvalidContentDisposition)?;
    if !charset.eq_ignore_ascii_case("utf-8") {
        return Err(HttpError::InvalidContentDisposition);
    }
    let bytes = percent_decode(encoded.as_bytes())?;
    String::from_utf8(bytes).map_err(|_error| HttpError::InvalidContentDisposition)
}

fn percent_decode(input: &[u8]) -> Result<Vec<u8>, HttpError> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0_usize;
    while index < input.len() {
        if input[index] == b'%' {
            if index + 2 >= input.len()
                || !input[index + 1].is_ascii_hexdigit()
                || !input[index + 2].is_ascii_hexdigit()
            {
                return Err(HttpError::InvalidContentDisposition);
            }
            output.push((hex_value(input[index + 1]) << 4) | hex_value(input[index + 2]));
            index += 3;
        } else {
            if !input[index].is_ascii() {
                return Err(HttpError::InvalidContentDisposition);
            }
            output.push(input[index]);
            index += 1;
        }
    }
    Ok(output)
}

fn validate_safe_filename(filename: &str) -> Result<(), HttpError> {
    if filename.is_empty()
        || filename.len() > MAX_SAFE_FILENAME_BYTES
        || filename.trim() != filename
        // A trailing ASCII space is already rejected by the trim equality above.
        || filename.ends_with('.')
        || matches!(filename, "." | "..")
        || filename.chars().any(is_forbidden_filename_character)
    {
        return Err(HttpError::InvalidContentDisposition);
    }
    let base = filename
        .split('.')
        .next()
        .unwrap_or(filename)
        .to_ascii_uppercase();
    if is_windows_device_name(&base) {
        return Err(HttpError::InvalidContentDisposition);
    }
    Ok(())
}

fn is_forbidden_filename_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0'
        )
        || matches!(
            character,
            '\u{202a}'
                | '\u{202b}'
                | '\u{202c}'
                | '\u{202d}'
                | '\u{202e}'
                | '\u{2066}'
                | '\u{2067}'
                | '\u{2068}'
                | '\u{2069}'
        )
}

fn is_windows_device_name(base: &str) -> bool {
    matches!(base, "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
        || (base.len() == 4
            && (base.starts_with("COM") || base.starts_with("LPT"))
            && matches!(base.as_bytes()[3], b'1'..=b'9'))
}

fn extension_mime_relation(filename: Option<&str>, observed: &MimeType) -> ExtensionMimeRelation {
    let Some(filename) = filename else {
        return ExtensionMimeRelation::Absent;
    };
    let Some(extension) = filename
        .rsplit_once('.')
        .map(|(_base, extension)| extension)
    else {
        return ExtensionMimeRelation::Absent;
    };
    // Safe filenames cannot end in '.', so a present extension is necessarily non-empty.
    let expected = match extension.to_ascii_lowercase().as_str() {
        "html" | "htm" => Some("text/html"),
        "xml" => Some("application/xml"),
        "svg" => Some("image/svg+xml"),
        "pdf" => Some("application/pdf"),
        "zip" => Some("application/zip"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "txt" => Some("text/plain"),
        "js" | "mjs" => Some("text/javascript"),
        _other => None,
    };
    match expected {
        Some(expected) if observed.essence() == expected => ExtensionMimeRelation::Match,
        Some(_expected) => ExtensionMimeRelation::Mismatch,
        None => ExtensionMimeRelation::Unknown,
    }
}

fn split_semicolon_segments(input: &[u8]) -> Result<Vec<&[u8]>, HttpError> {
    let mut segments = Vec::new();
    let mut start = 0_usize;
    let mut in_quote = false;
    let mut escaped = false;
    for (index, byte) in input.iter().copied().enumerate() {
        if escaped {
            escaped = false;
        } else if in_quote && byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            in_quote = !in_quote;
        } else if byte == b';' && !in_quote {
            segments.push(&input[start..index]);
            start = index + 1;
        }
    }
    // `escaped` can only be set while `in_quote` is true, so an unfinished escape is already an
    // unfinished quoted string. Keeping a second end-state condition would represent an
    // impossible state and obscure the parser invariant.
    if in_quote {
        return Err(HttpError::InvalidContentDisposition);
    }
    segments.push(&input[start..]);
    Ok(segments)
}

fn absolute_location_origin(location: &str) -> Result<&str, HttpError> {
    let scheme_end = location
        .find("://")
        .ok_or(HttpError::InvalidRedirectMetadata)?;
    let authority_start = scheme_end + 3;
    let authority_end = location[authority_start..]
        .find(['/', '?'])
        .map_or(location.len(), |offset| authority_start + offset);
    if authority_end == authority_start {
        return Err(HttpError::InvalidRedirectMetadata);
    }
    Ok(&location[..authority_end])
}

fn ascii_lowercase(value: &[u8]) -> String {
    value
        .iter()
        .map(|byte| char::from(byte.to_ascii_lowercase()))
        .collect()
}

fn hex_value(byte: u8) -> u8 {
    let lower = byte.to_ascii_lowercase();
    if lower.is_ascii_digit() {
        lower - b'0'
    } else {
        // `percent_decode` validates ASCII hex before calling this helper.
        lower - b'a' + 10
    }
}

fn sha256_identifier(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut identifier = String::with_capacity(71);
    identifier.push_str("sha256:");
    for byte in digest {
        identifier.push(char::from(HEX_LOWER[usize::from(byte >> 4)]));
        identifier.push(char::from(HEX_LOWER[usize::from(byte & 0x0f)]));
    }
    identifier
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use crate::field::FieldLine;
    use crate::mime::observe_mime_type;

    use super::*;

    fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
        FieldBlock::new(
            entries
                .iter()
                .map(|(name, value)| {
                    FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("field")
                })
                .collect(),
        )
    }

    fn observed(content: &[u8]) -> ObservedMimeClassification {
        observe_mime_type(content, None).expect("observed MIME")
    }

    #[test]
    fn inline_and_attachment_metadata_preserve_safe_names() {
        let inline = parse_content_disposition(
            &fields(&[("content-disposition", b"inline")]),
            &observed(b"plain text"),
        )
        .expect("inline metadata")
        .expect("present");
        assert_eq!(inline.kind(), DispositionKind::Inline);
        assert_eq!(inline.filename(), None);
        assert_eq!(
            inline.extension_mime_relation(),
            ExtensionMimeRelation::Absent
        );

        let attachment = parse_content_disposition(
            &fields(&[("content-disposition", b"attachment; filename=report.pdf")]),
            &observed(b"%PDF-1.7"),
        )
        .expect("attachment metadata")
        .expect("present");
        assert_eq!(attachment.kind(), DispositionKind::Attachment);
        assert_eq!(attachment.filename(), Some("report.pdf"));
        assert_eq!(
            attachment.extension_mime_relation(),
            ExtensionMimeRelation::Match
        );
    }

    #[test]
    fn utf8_extended_filename_takes_precedence() {
        let disposition = parse_content_disposition(
            &fields(&[(
                "content-disposition",
                b"attachment; filename=fallback.txt; filename*=UTF-8''%ED%95%9C%EA%B8%80.txt",
            )]),
            &observed(b"plain text"),
        )
        .expect("extended filename")
        .expect("present");
        assert_eq!(disposition.filename(), Some("한글.txt"));
        assert_eq!(
            disposition.extension_mime_relation(),
            ExtensionMimeRelation::Match
        );

        let lowercase_hex = parse_content_disposition(
            &fields(&[(
                "content-disposition",
                b"attachment; filename*=UTF-8''report%2etxt",
            )]),
            &observed(b"plain text"),
        )
        .expect("lowercase percent hex")
        .expect("present");
        assert_eq!(lowercase_hex.filename(), Some("report.txt"));
    }

    #[test]
    fn quoted_filename_escapes_are_decoded_without_path_authority() {
        let disposition = parse_content_disposition(
            &fields(&[(
                "content-disposition",
                b"attachment; filename=\"quarter\\ one.txt\"",
            )]),
            &observed(b"plain"),
        )
        .expect("quoted filename")
        .expect("present");
        assert_eq!(disposition.filename(), Some("quarter one.txt"));
    }

    #[test]
    fn hostile_filename_and_parameter_syntax_fail_closed() {
        let invalid_values = [
            b"form-data; filename=x".as_slice(),
            b"attachment; unknown=x",
            b"attachment; filename=a; FILENAME=b",
            b"attachment; filename*=ISO-8859-1''name.txt",
            b"attachment; filename*=UTF-8''bad%ZZ.txt",
            b"attachment; filename=\"unterminated",
            b"attachment; filename=../escape.txt",
            b"attachment; filename=C:drive.txt",
            b"attachment; filename=path\\escape.txt",
            b"attachment; filename=/absolute.txt",
            b"attachment; filename=CON.txt",
            b"attachment; filename=LPT9.log",
            b"attachment; filename=trailing.",
            b"attachment; filename=\" surrounding.txt \"",
        ];
        for value in invalid_values {
            assert!(matches!(
                parse_content_disposition(
                    &fields(&[("content-disposition", value)]),
                    &observed(b"plain"),
                ),
                Err(HttpError::InvalidContentDisposition)
            ));
        }
        let bidi = "attachment; filename=evil\u{202e}txt.exe";
        assert!(matches!(
            parse_content_disposition(
                &fields(&[("content-disposition", bidi.as_bytes())]),
                &observed(b"plain"),
            ),
            Err(HttpError::InvalidContentDisposition)
        ));
        let long = format!("attachment; filename={}", "a".repeat(256));
        assert!(matches!(
            parse_content_disposition(
                &fields(&[("content-disposition", long.as_bytes())]),
                &observed(b"plain"),
            ),
            Err(HttpError::InvalidContentDisposition)
        ));
    }

    #[test]
    fn extension_relations_distinguish_mismatch_unknown_and_absence() {
        for (value, expected) in [
            (
                b"attachment; filename=page.pdf".as_slice(),
                ExtensionMimeRelation::Mismatch,
            ),
            (
                b"attachment; filename=data.custom",
                ExtensionMimeRelation::Unknown,
            ),
            (
                b"attachment; filename=README",
                ExtensionMimeRelation::Absent,
            ),
        ] {
            let disposition = parse_content_disposition(
                &fields(&[("content-disposition", value)]),
                &observed(b"<html>"),
            )
            .expect("disposition")
            .expect("present");
            assert_eq!(disposition.extension_mime_relation(), expected);
        }
    }

    #[test]
    fn redirect_metadata_hashes_absolute_and_relative_locations_without_following() {
        let absolute = parse_redirect_metadata(
            302,
            &fields(&[(
                "location",
                b"https://example.net:8443/new/path?token=secret",
            )]),
        )
        .expect("absolute redirect")
        .expect("present");
        assert!(!absolute.is_relative());
        assert_eq!(
            absolute.target_origin(),
            Some(&Origin::parse("https://example.net:8443").expect("target origin"))
        );
        assert!(absolute.location_hash().starts_with("sha256:"));
        assert_eq!(absolute.location_hash().len(), 71);
        assert!(!format!("{absolute:?}").contains("secret"));

        let relative =
            parse_redirect_metadata(307, &fields(&[("location", b"/next/path?opaque=value")]))
                .expect("relative redirect")
                .expect("present");
        assert!(relative.is_relative());
        assert_eq!(relative.target_origin(), None);
    }

    #[test]
    fn non_redirect_status_and_absent_location_produce_no_metadata() {
        assert_eq!(
            parse_redirect_metadata(200, &fields(&[("location", b"/ignored")]))
                .expect("non-redirect"),
            None
        );
        assert_eq!(
            parse_redirect_metadata(302, &FieldBlock::default()).expect("absent Location"),
            None
        );
    }

    #[test]
    fn duplicate_or_hostile_redirect_metadata_fails_closed() {
        for entries in [
            vec![
                ("location", b"/one".as_slice()),
                ("location", b"/two".as_slice()),
            ],
            vec![("location", b"".as_slice())],
            vec![("location", b"relative/path".as_slice())],
            vec![("location", b"https://user@example.com/path".as_slice())],
            vec![("location", b"https://example.com/path#fragment".as_slice())],
            vec![("location", b"https://example.com/white space".as_slice())],
        ] {
            assert!(matches!(
                parse_redirect_metadata(302, &fields(&entries)),
                Err(HttpError::InvalidRedirectMetadata)
            ));
        }
    }
}
