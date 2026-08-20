#![allow(clippy::expect_used)]

use std::io::Write;
use std::time::Duration;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::content::decode_content;
use crate::field::{FieldBlock, FieldLine};
use crate::{AlpnHttp11Policy, HttpClientPolicy, HttpError, IntegrityRequirement};

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

fn deflate_fields() -> FieldBlock {
    FieldBlock::new(vec![
        FieldLine::new(b"content-encoding", b"deflate", 256, 8_192)
            .expect("deflate content coding"),
    ])
}

#[test]
fn zlib_deflate_rejects_trailing_octets_after_the_coded_representation() {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"payload").expect("encode payload");
    let mut encoded = encoder.finish().expect("finish zlib member");
    encoded.extend_from_slice(b"trailing-octets");

    let error = decode_content(&encoded, &deflate_fields(), &policy(encoded.len()))
        .expect_err("bytes after the zlib stream must not be silently discarded");
    assert!(matches!(error, HttpError::ContentDecodingFailed { .. }));
}
