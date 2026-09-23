#![allow(clippy::expect_used)]

use std::io::Write;
use std::time::Duration;

use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder};

use crate::content::decode_content;
use crate::field::{FieldBlock, FieldLine};
use crate::{AlpnHttp11Policy, ContentCoding, HttpClientPolicy, HttpError, IntegrityRequirement};

fn policy(maximum_encoded_bytes: usize) -> HttpClientPolicy {
    HttpClientPolicy::new(
        Duration::from_secs(1),
        1_024,
        1_024,
        16,
        64,
        256,
        1_024,
        4,
        16,
        4,
        1_024,
        maximum_encoded_bytes,
        1_024,
        32,
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("content policy")
}

fn content_encoding_fields(coding: &[u8]) -> FieldBlock {
    FieldBlock::new(vec![
        FieldLine::new(b"content-encoding", coding, 256, 8_192).expect("content coding"),
    ])
}

#[test]
fn zlib_deflate_rejects_trailing_octets_after_the_coded_representation() {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"payload").expect("encode payload");
    let mut encoded = encoder.finish().expect("finish zlib member");
    encoded.extend_from_slice(b"trailing-octets");

    let error = decode_content(
        &encoded,
        &content_encoding_fields(b"deflate"),
        &policy(encoded.len()),
    )
    .expect_err("bytes after the zlib stream must not be silently discarded");
    assert!(matches!(error, HttpError::ContentDecodingFailed { .. }));
}

#[test]
fn gzip_decodes_all_rfc1952_members_as_one_coded_representation() {
    let mut first = GzEncoder::new(Vec::new(), Compression::default());
    first.write_all(b"first ").expect("encode first member");
    let mut encoded = first.finish().expect("finish first gzip member");

    let mut second = GzEncoder::new(Vec::new(), Compression::default());
    second.write_all(b"second").expect("encode second member");
    encoded.extend_from_slice(&second.finish().expect("finish second gzip member"));

    let decoded = decode_content(
        &encoded,
        &content_encoding_fields(b"gzip"),
        &policy(encoded.len()),
    )
    .expect("RFC 1952 concatenated members form one gzip representation");
    assert_eq!(decoded.bytes, b"first second");
    assert_eq!(decoded.coding, ContentCoding::Gzip);
}
