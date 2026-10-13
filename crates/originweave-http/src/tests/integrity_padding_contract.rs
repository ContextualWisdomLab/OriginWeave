#![allow(clippy::expect_used)]

use crate::field::{FieldBlock, FieldLine};
use crate::integrity::{IntegrityAlgorithm, IntegrityStatus, validate_content_digest};
use crate::{HttpError, IntegrityRequirement};

fn fields(value: &[u8]) -> FieldBlock {
    FieldBlock::new(vec![
        FieldLine::new(b"content-digest", value, 256, 8_192).expect("digest field"),
    ])
}

#[test]
fn rfc8941_byte_sequence_accepts_omitted_base64_padding() {
    let unpadded_sha256 = b"sha-256=:LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ:";
    assert_eq!(
        validate_content_digest(
            &fields(unpadded_sha256),
            &FieldBlock::default(),
            b"hello",
            IntegrityRequirement::RequireSupportedDigest,
        )
        .expect("RFC 8941 permits omitted base64 padding when the decoder supports it"),
        IntegrityStatus::Verified(vec![IntegrityAlgorithm::Sha256])
    );
}

#[test]
fn structured_byte_sequence_stays_strict_about_malformed_base64() {
    for invalid in [
        b"sha-256=:A:".as_slice(),
        b"sha-256=:AA*:".as_slice(),
        b"sha-256=:AB:".as_slice(),
        b"sha-256=:AA".as_slice(),
    ] {
        assert!(matches!(
            validate_content_digest(
                &fields(invalid),
                &FieldBlock::default(),
                b"hello",
                IntegrityRequirement::Optional,
            ),
            Err(HttpError::InvalidDigestField)
        ));
    }
}
