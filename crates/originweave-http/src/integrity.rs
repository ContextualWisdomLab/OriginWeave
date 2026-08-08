//! RFC 9530 digest-field parsing and bounded integrity validation.

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, STANDARD_PAD_INDIFFERENT};
use sha2::{Digest, Sha256, Sha512};

use crate::field::{FieldBlock, is_token_byte};
use crate::{HttpError, IntegrityRequirement};

/// One digest algorithm supported by the first HTTP integrity slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegrityAlgorithm {
    /// SHA-256 as registered by RFC 9530.
    Sha256,
    /// SHA-512 as registered by RFC 9530.
    Sha512,
}

impl IntegrityAlgorithm {
    /// Return the registered Structured Fields dictionary key.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Sha256 => "sha-256",
            Self::Sha512 => "sha-512",
        }
    }
}

/// Result of evaluating one RFC 9530 digest field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// No applicable digest field was supplied and policy permitted absence.
    Absent,
    /// Every supported digest value present matched the applicable bytes.
    Verified(Vec<IntegrityAlgorithm>),
    /// A syntactically valid field contained no supported algorithm.
    UnsupportedAlgorithm,
    /// The field is valid, but the first slice cannot define the representation context.
    UnsupportedContext,
}

pub(crate) fn validate_content_digest(
    fields: &FieldBlock,
    trailers: &FieldBlock,
    content_bytes: &[u8],
    requirement: IntegrityRequirement,
) -> Result<IntegrityStatus, HttpError> {
    validate_digest_values(
        resolve_digest_dictionary(
            &fields.values("content-digest"),
            &trailers.values("content-digest"),
        )?,
        content_bytes,
        requirement,
    )
}

pub(crate) fn validate_representation_digest(
    fields: &FieldBlock,
    trailers: &FieldBlock,
    representation_bytes: &[u8],
    status_code: u16,
    has_content_range: bool,
    requirement: IntegrityRequirement,
) -> Result<IntegrityStatus, HttpError> {
    let dictionary = resolve_digest_dictionary(
        &fields.values("repr-digest"),
        &trailers.values("repr-digest"),
    )?;
    if dictionary.is_some() && (status_code != 200 || has_content_range) {
        return Ok(IntegrityStatus::UnsupportedContext);
    }
    validate_digest_values(dictionary, representation_bytes, requirement)
}

fn validate_digest_values(
    dictionary: Option<BTreeMap<String, Vec<u8>>>,
    bytes: &[u8],
    requirement: IntegrityRequirement,
) -> Result<IntegrityStatus, HttpError> {
    let Some(dictionary) = dictionary else {
        return match requirement {
            IntegrityRequirement::Optional => Ok(IntegrityStatus::Absent),
            IntegrityRequirement::RequireSupportedDigest => Err(HttpError::SupportedDigestRequired),
        };
    };
    let mut verified = Vec::new();
    for algorithm in [IntegrityAlgorithm::Sha256, IntegrityAlgorithm::Sha512] {
        if let Some(observed) = dictionary.get(algorithm.key()) {
            let expected = digest_bytes(algorithm, bytes);
            if observed.as_slice() != expected.as_slice() {
                return Err(HttpError::DigestMismatch {
                    algorithm: algorithm.key(),
                });
            }
            verified.push(algorithm);
        }
    }
    if verified.is_empty() {
        return match requirement {
            IntegrityRequirement::Optional => Ok(IntegrityStatus::UnsupportedAlgorithm),
            IntegrityRequirement::RequireSupportedDigest => Err(HttpError::SupportedDigestRequired),
        };
    }
    Ok(IntegrityStatus::Verified(verified))
}

fn resolve_digest_dictionary(
    header_values: &[&[u8]],
    trailer_values: &[&[u8]],
) -> Result<Option<BTreeMap<String, Vec<u8>>>, HttpError> {
    if header_values.is_empty() && trailer_values.is_empty() {
        return Ok(None);
    }
    let mut dictionary = BTreeMap::new();
    parse_digest_dictionary_into(header_values, &mut dictionary)?;
    parse_digest_dictionary_into(trailer_values, &mut dictionary)?;
    Ok(Some(dictionary))
}

fn parse_digest_dictionary_into(
    values: &[&[u8]],
    dictionary: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), HttpError> {
    for value in values {
        let mut cursor = 0_usize;
        skip_spaces(value, &mut cursor);
        loop {
            let key = parse_key(value, &mut cursor)?;
            if value.get(cursor) != Some(&b'=') {
                return Err(HttpError::InvalidDigestField);
            }
            cursor += 1;
            let bytes = parse_byte_sequence(value, &mut cursor)?;
            parse_parameters(value, &mut cursor)?;
            skip_ows(value, &mut cursor);

            // RFC 8941 dictionary parsing is last-occurrence-wins, including across repeated
            // field lines. RFC 9530 explicitly permits digest trailers to be merged into the
            // corresponding header field, so trailer members are parsed after header members.
            let key: String = key.iter().map(|byte| char::from(*byte)).collect();
            dictionary.insert(key, bytes);

            if cursor == value.len() {
                break;
            }
            if value[cursor] != b',' {
                return Err(HttpError::InvalidDigestField);
            }
            cursor += 1;
            skip_ows(value, &mut cursor);
            if cursor == value.len() {
                return Err(HttpError::InvalidDigestField);
            }
        }
    }
    Ok(())
}

fn parse_key<'a>(input: &'a [u8], cursor: &mut usize) -> Result<&'a [u8], HttpError> {
    let start = *cursor;
    let Some(first) = input.get(*cursor) else {
        return Err(HttpError::InvalidDigestField);
    };
    if !(first.is_ascii_lowercase() || *first == b'*') {
        return Err(HttpError::InvalidDigestField);
    }
    *cursor += 1;
    while input.get(*cursor).is_some_and(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'_' | b'-' | b'.' | b'*')
    }) {
        *cursor += 1;
    }
    Ok(&input[start..*cursor])
}

fn parse_byte_sequence(input: &[u8], cursor: &mut usize) -> Result<Vec<u8>, HttpError> {
    if input.get(*cursor) != Some(&b':') {
        return Err(HttpError::InvalidDigestField);
    }
    *cursor += 1;
    let start = *cursor;
    while input.get(*cursor).is_some_and(|byte| *byte != b':') {
        *cursor += 1;
    }
    if input.get(*cursor) != Some(&b':') {
        return Err(HttpError::InvalidDigestField);
    }
    let decoded = STANDARD_PAD_INDIFFERENT
        .decode(&input[start..*cursor])
        .map_err(|_error| HttpError::InvalidDigestField)?;
    *cursor += 1;
    Ok(decoded)
}

fn parse_parameters(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    while input.get(*cursor) == Some(&b';') {
        *cursor += 1;
        skip_spaces(input, cursor);
        let _parameter_key = parse_key(input, cursor)?;
        if input.get(*cursor) == Some(&b'=') {
            *cursor += 1;
            parse_bare_item(input, cursor)?;
        }
    }
    Ok(())
}

fn parse_bare_item(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    match input.get(*cursor).copied() {
        Some(b'-' | b'0'..=b'9') => parse_number(input, cursor),
        Some(b'"') => parse_string(input, cursor),
        Some(byte) if byte.is_ascii_alphabetic() || byte == b'*' => parse_token(input, cursor),
        Some(b':') => parse_byte_sequence(input, cursor).map(|_bytes| ()),
        Some(b'?') => parse_boolean(input, cursor),
        _other => Err(HttpError::InvalidDigestField),
    }
}

fn parse_number(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    if input.get(*cursor) == Some(&b'-') {
        *cursor += 1;
    }
    let integer_start = *cursor;
    while input.get(*cursor).is_some_and(u8::is_ascii_digit) {
        *cursor += 1;
    }
    let integer_digits = *cursor - integer_start;
    if integer_digits == 0 {
        return Err(HttpError::InvalidDigestField);
    }
    if input.get(*cursor) == Some(&b'.') {
        if integer_digits > 12 {
            return Err(HttpError::InvalidDigestField);
        }
        *cursor += 1;
        let fraction_start = *cursor;
        while input.get(*cursor).is_some_and(u8::is_ascii_digit) {
            *cursor += 1;
        }
        let fraction_digits = *cursor - fraction_start;
        if !(1..=3).contains(&fraction_digits) {
            return Err(HttpError::InvalidDigestField);
        }
    } else if integer_digits > 15 {
        return Err(HttpError::InvalidDigestField);
    }
    Ok(())
}

fn parse_string(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    *cursor += 1;
    loop {
        let Some(byte) = input.get(*cursor).copied() else {
            return Err(HttpError::InvalidDigestField);
        };
        *cursor += 1;
        match byte {
            b'"' => return Ok(()),
            b'\\' => {
                let Some(escaped) = input.get(*cursor).copied() else {
                    return Err(HttpError::InvalidDigestField);
                };
                if !matches!(escaped, b'"' | b'\\') {
                    return Err(HttpError::InvalidDigestField);
                }
                *cursor += 1;
            }
            0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7e => {}
            _other => return Err(HttpError::InvalidDigestField),
        }
    }
}

fn parse_token(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    *cursor += 1;
    while input
        .get(*cursor)
        .is_some_and(|byte| is_token_byte(*byte) || matches!(byte, b':' | b'/'))
    {
        *cursor += 1;
    }
    Ok(())
}

fn parse_boolean(input: &[u8], cursor: &mut usize) -> Result<(), HttpError> {
    match input.get(*cursor + 1) {
        Some(b'0' | b'1') => {
            *cursor += 2;
            Ok(())
        }
        _other => Err(HttpError::InvalidDigestField),
    }
}

fn skip_spaces(input: &[u8], cursor: &mut usize) {
    while input.get(*cursor) == Some(&b' ') {
        *cursor += 1;
    }
}

fn skip_ows(input: &[u8], cursor: &mut usize) {
    while input
        .get(*cursor)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        *cursor += 1;
    }
}

fn digest_bytes(algorithm: IntegrityAlgorithm, bytes: &[u8]) -> Vec<u8> {
    match algorithm {
        IntegrityAlgorithm::Sha256 => Sha256::digest(bytes).to_vec(),
        IntegrityAlgorithm::Sha512 => Sha512::digest(bytes).to_vec(),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use crate::field::FieldLine;

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

    fn digest_member(algorithm: IntegrityAlgorithm, bytes: &[u8]) -> String {
        let encoded = STANDARD.encode(digest_bytes(algorithm, bytes));
        format!("{}=:{encoded}:", algorithm.key())
    }

    #[test]
    fn supported_content_digests_match_known_answer_vectors() {
        let field_block = fields(&[(
            "content-digest",
            b"sha-256=:LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=:, sha-512=:m3HSJL1i83hdltRq0+o9czGb+8KJDKra4t/3JRlnPKcjI8PZm6XBHXx6zG4UuMXaDEZjR1wuXDre9G9zvN7AQw==:",
        )]);
        assert_eq!(
            validate_content_digest(
                &field_block,
                &FieldBlock::default(),
                b"hello",
                IntegrityRequirement::RequireSupportedDigest,
            )
            .expect("valid digests"),
            IntegrityStatus::Verified(
                vec![IntegrityAlgorithm::Sha256, IntegrityAlgorithm::Sha512,]
            )
        );
    }

    #[test]
    fn structured_field_parameters_and_duplicate_keys_follow_rfc8941() {
        let correct = digest_member(IntegrityAlgorithm::Sha256, b"");
        let wrong = digest_member(IntegrityAlgorithm::Sha256, b"wrong");
        let parameterized = format!(
            "{correct};flag;integer=-7;decimal=1.25;text=\"a\\\"b\";token=Abc/def;binary=:AQ==:;bool=?0;source=7;source=9"
        );
        assert_eq!(
            validate_content_digest(
                &fields(&[("content-digest", parameterized.as_bytes())]),
                &FieldBlock::default(),
                b"",
                IntegrityRequirement::RequireSupportedDigest,
            )
            .expect("byte sequence item parameters are extensible metadata"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
        );

        assert_eq!(
            validate_content_digest(
                &fields(&[
                    ("content-digest", wrong.as_bytes()),
                    ("content-digest", parameterized.as_bytes()),
                ]),
                &FieldBlock::default(),
                b"",
                IntegrityRequirement::RequireSupportedDigest,
            )
            .expect("the last duplicate dictionary key wins"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
        );
    }

    #[test]
    fn header_and_trailer_digest_members_merge_in_message_order() {
        let sha256 = digest_member(IntegrityAlgorithm::Sha256, b"");
        let sha512 = digest_member(IntegrityAlgorithm::Sha512, b"");
        assert_eq!(
            validate_content_digest(
                &fields(&[("content-digest", sha256.as_bytes())]),
                &fields(&[("content-digest", sha512.as_bytes())]),
                b"",
                IntegrityRequirement::Optional,
            )
            .expect("RFC 9530 permits a digest trailer to merge into the header field"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256, IntegrityAlgorithm::Sha512])
        );

        let wrong = digest_member(IntegrityAlgorithm::Sha256, b"wrong");
        assert_eq!(
            validate_content_digest(
                &fields(&[("content-digest", wrong.as_bytes())]),
                &fields(&[("content-digest", sha256.as_bytes())]),
                b"",
                IntegrityRequirement::Optional,
            )
            .expect("later trailer dictionary members replace duplicate header members"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
        );
    }

    #[test]
    fn absence_and_unsupported_algorithms_follow_policy() {
        assert_eq!(
            validate_content_digest(
                &FieldBlock::default(),
                &FieldBlock::default(),
                b"content",
                IntegrityRequirement::Optional,
            )
            .expect("optional absence"),
            IntegrityStatus::Absent
        );
        assert!(matches!(
            validate_content_digest(
                &FieldBlock::default(),
                &FieldBlock::default(),
                b"content",
                IntegrityRequirement::RequireSupportedDigest,
            ),
            Err(HttpError::SupportedDigestRequired)
        ));
        let unsupported = fields(&[("content-digest", b"sha-999=:AQ==:")]);
        assert_eq!(
            validate_content_digest(
                &unsupported,
                &FieldBlock::default(),
                b"content",
                IntegrityRequirement::Optional,
            )
            .expect("unsupported algorithm"),
            IntegrityStatus::UnsupportedAlgorithm
        );
        assert!(matches!(
            validate_content_digest(
                &unsupported,
                &FieldBlock::default(),
                b"content",
                IntegrityRequirement::RequireSupportedDigest,
            ),
            Err(HttpError::SupportedDigestRequired)
        ));
    }

    #[test]
    fn malformed_structured_field_members_fail_closed() {
        for invalid in [
            b"".as_slice(),
            b"SHA-256=:AQ==:",
            b"sha/256=:AQ==:",
            b"sha-256",
            b"sha-256=AQ==",
            b"sha-256=\"AQ==\"",
            b"sha-256=:not base64:",
            b"sha-256=:AQ==:;=1",
            b"sha-256=:AQ==:;foo=",
            b"sha-256=:AQ==:;Foo=1",
            b"sha-256=:AQ==:;n=-",
            b"sha-256=:AQ==:;n=1234567890123456",
            b"sha-256=:AQ==:;n=1234567890123.1",
            b"sha-256=:AQ==:;n=1.",
            b"sha-256=:AQ==:;n=1.2345",
            b"sha-256=:AQ==:;s=\"bad\\x\"",
            b"sha-256=:AQ==:;s=\"unterminated",
            b"sha-256=:AQ==:;b=?2",
            b"sha-256=:AQ==:;date=@42",
            b"sha-256=:AQ==:;display=%\"ok\"",
            b"sha-256=:AQ==: ,",
        ] {
            assert!(matches!(
                validate_content_digest(
                    &fields(&[("content-digest", invalid)]),
                    &FieldBlock::default(),
                    b"content",
                    IntegrityRequirement::Optional,
                ),
                Err(HttpError::InvalidDigestField)
            ));
        }
    }

    #[test]
    fn empty_structured_byte_sequence_is_syntactically_valid_but_cannot_match_sha256() {
        assert!(matches!(
            validate_content_digest(
                &fields(&[("content-digest", b"sha-256=::")]),
                &FieldBlock::default(),
                b"content",
                IntegrityRequirement::Optional,
            ),
            Err(HttpError::DigestMismatch {
                algorithm: "sha-256"
            })
        ));
    }

    #[test]
    fn supported_digest_mismatch_identifies_only_the_algorithm() {
        let error = validate_content_digest(
            &fields(&[(
                "content-digest",
                b"sha-256=:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=:",
            )]),
            &FieldBlock::default(),
            b"not empty",
            IntegrityRequirement::Optional,
        )
        .expect_err("digest mismatch");
        assert!(matches!(
            error,
            HttpError::DigestMismatch {
                algorithm: "sha-256"
            }
        ));
        assert!(!error.to_string().contains("47DE"));
    }

    #[test]
    fn representation_digest_is_validated_only_for_supported_full_context() {
        let digest = fields(&[(
            "repr-digest",
            b"sha-256=:Y39Vfsc6JaKuw7be30VwWgoMK//ZDxApEd+FUBoaVH8=:",
        )]);
        assert_eq!(
            validate_representation_digest(
                &digest,
                &FieldBlock::default(),
                b"deterministic content",
                200,
                false,
                IntegrityRequirement::Optional,
            )
            .expect("supported representation"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
        );
        for (status, has_content_range) in [(206, false), (200, true)] {
            assert_eq!(
                validate_representation_digest(
                    &digest,
                    &FieldBlock::default(),
                    b"deterministic content",
                    status,
                    has_content_range,
                    IntegrityRequirement::Optional,
                )
                .expect("unsupported context"),
                IntegrityStatus::UnsupportedContext
            );
        }
    }
}
