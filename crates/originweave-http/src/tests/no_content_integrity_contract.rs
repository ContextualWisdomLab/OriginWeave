#![allow(clippy::expect_used)]

use crate::field::{FieldBlock, FieldLine};
use crate::integrity::{
    validate_content_digest_without_content, validate_representation_digest_without_content,
};
use crate::{HttpError, IntegrityRequirement, IntegrityStatus};

fn fields(name: &str, value: &[u8]) -> FieldBlock {
    FieldBlock::new(vec![
        FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("valid fixture field"),
    ])
}

#[test]
fn no_content_digest_contexts_cover_absence_supported_and_unsupported_algorithms() {
    let empty = FieldBlock::default();
    let supported = fields("content-digest", b"sha-256=:AQ==:");
    let unsupported = fields("content-digest", b"md5=:AQ==:");

    assert_eq!(
        validate_content_digest_without_content(&empty, &empty, IntegrityRequirement::Optional,)
            .expect("optional absence"),
        IntegrityStatus::Absent
    );
    assert_eq!(
        validate_content_digest_without_content(
            &supported,
            &empty,
            IntegrityRequirement::Optional,
        )
        .expect("supported digest without content"),
        IntegrityStatus::UnsupportedContext
    );
    assert_eq!(
        validate_content_digest_without_content(
            &unsupported,
            &empty,
            IntegrityRequirement::Optional,
        )
        .expect("unsupported digest without content"),
        IntegrityStatus::UnsupportedAlgorithm
    );
    assert!(matches!(
        validate_content_digest_without_content(
            &empty,
            &empty,
            IntegrityRequirement::RequireSupportedDigest,
        ),
        Err(HttpError::SupportedDigestRequired)
    ));
    assert!(matches!(
        validate_content_digest_without_content(
            &supported,
            &empty,
            IntegrityRequirement::RequireSupportedDigest,
        ),
        Err(HttpError::SupportedDigestRequired)
    ));
}

#[test]
fn no_representation_content_uses_the_same_fail_closed_requirement_semantics() {
    let empty = FieldBlock::default();
    let supported = fields("repr-digest", b"sha-512=:AQ==:");
    let unsupported = fields("repr-digest", b"md5=:AQ==:");

    assert_eq!(
        validate_representation_digest_without_content(
            &empty,
            &empty,
            IntegrityRequirement::Optional,
        )
        .expect("optional absence"),
        IntegrityStatus::Absent
    );
    assert_eq!(
        validate_representation_digest_without_content(
            &supported,
            &empty,
            IntegrityRequirement::Optional,
        )
        .expect("supported digest without representation bytes"),
        IntegrityStatus::UnsupportedContext
    );
    assert_eq!(
        validate_representation_digest_without_content(
            &unsupported,
            &empty,
            IntegrityRequirement::Optional,
        )
        .expect("unsupported digest without representation bytes"),
        IntegrityStatus::UnsupportedAlgorithm
    );
    assert!(matches!(
        validate_representation_digest_without_content(
            &unsupported,
            &empty,
            IntegrityRequirement::RequireSupportedDigest,
        ),
        Err(HttpError::SupportedDigestRequired)
    ));
}
