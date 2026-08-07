//! RFC 9530 digest-field parsing and bounded integrity validation.

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256, Sha512};

use crate::field::FieldBlock;
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
    let headers = parse_digest_dictionary(header_values)?;
    let trailers = parse_digest_dictionary(trailer_values)?;
    match (headers, trailers) {
        (None, None) => Ok(None),
        (Some(dictionary), None) | (None, Some(dictionary)) => Ok(Some(dictionary)),
        (Some(headers), Some(trailers)) if headers == trailers => Ok(Some(headers)),
        (Some(_headers), Some(_trailers)) => Err(HttpError::InvalidDigestField),
    }
}

fn parse_digest_dictionary(
    values: &[&[u8]],
) -> Result<Option<BTreeMap<String, Vec<u8>>>, HttpError> {
    if values.is_empty() {
        return Ok(None);
    }
    let mut dictionary = BTreeMap::new();
    for value in values {
        for raw_member in value.split(|byte| *byte == b',') {
            let member = trim_optional_whitespace(raw_member);
            let equals = member
                .iter()
                .position(|byte| *byte == b'=')
                .ok_or(HttpError::InvalidDigestField)?;
            let key = trim_optional_whitespace(&member[..equals]);
            let encoded = trim_optional_whitespace(&member[equals + 1..]);
            if !valid_dictionary_key(key)
                || encoded.len() < 2
                || encoded.first() != Some(&b':')
                || encoded.last() != Some(&b':')
            {
                return Err(HttpError::InvalidDigestField);
            }
            let key = std::str::from_utf8(key)
                .map_err(|_error| HttpError::InvalidDigestField)?
                .to_owned();
            let bytes = STANDARD
                .decode(&encoded[1..encoded.len() - 1])
                .map_err(|_error| HttpError::InvalidDigestField)?;
            if bytes.is_empty() || dictionary.insert(key, bytes).is_some() {
                return Err(HttpError::InvalidDigestField);
            }
        }
    }
    Ok(Some(dictionary))
}

fn valid_dictionary_key(key: &[u8]) -> bool {
    let Some(first) = key.first() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || *first == b'*') {
        return false;
    }
    key[1..].iter().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'_' | b'-' | b'.' | b'*' | b'/')
    })
}

fn digest_bytes(algorithm: IntegrityAlgorithm, bytes: &[u8]) -> Vec<u8> {
    match algorithm {
        IntegrityAlgorithm::Sha256 => Sha256::digest(bytes).to_vec(),
        IntegrityAlgorithm::Sha512 => Sha512::digest(bytes).to_vec(),
    }
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
    fn header_and_trailer_dictionaries_must_be_identical() {
        let value = b"sha-256=:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=:";
        assert_eq!(
            validate_content_digest(
                &fields(&[("content-digest", value)]),
                &fields(&[("content-digest", value)]),
                b"",
                IntegrityRequirement::Optional,
            )
            .expect("matching dictionaries"),
            IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
        );
        assert!(matches!(
            validate_content_digest(
                &fields(&[("content-digest", value)]),
                &fields(&[(
                    "content-digest",
                    b"sha-256=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=:"
                )]),
                b"",
                IntegrityRequirement::Optional,
            ),
            Err(HttpError::InvalidDigestField)
        ));
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
            b"sha-256",
            b"sha-256=AQ==",
            b"sha-256=\"AQ==\"",
            b"sha-256=::",
            b"sha-256=:not base64:",
            b"sha-256=:AQ==:;foo=1",
            b"sha-256=:AQ==:",
            b"sha-256=:AQ==:, sha-256=:AQ==:",
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
