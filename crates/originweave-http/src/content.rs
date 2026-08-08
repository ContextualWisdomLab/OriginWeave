//! Encoded-body collection and bounded HTTP content-coding decoding.

use std::io::{self, Read};

use flate2::read::{GzDecoder, ZlibDecoder};

use crate::field::{FieldBlock, trim_optional_whitespace};
use crate::{HttpClientPolicy, HttpError};

/// One supported HTTP content-coding decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCoding {
    /// No content coding changes the message content bytes.
    Identity,
    /// The content uses the gzip wrapper and DEFLATE coding.
    Gzip,
    /// The content uses zlib-wrapped DEFLATE; raw wrapperless DEFLATE is rejected as
    /// [`HttpError::ContentDecodingFailed`].
    Deflate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedContent {
    pub(crate) bytes: Vec<u8>,
    pub(crate) coding: ContentCoding,
}

pub(crate) fn decode_content(
    encoded: &[u8],
    fields: &FieldBlock,
    policy: &HttpClientPolicy,
) -> Result<DecodedContent, HttpError> {
    if encoded.len() > policy.max_encoded_content_bytes() {
        return Err(HttpError::EncodedContentTooLarge {
            // OriginWeave supports Rust targets whose pointer width is at most 64 bits, so this
            // widening conversion cannot truncate a valid slice length.
            byte_count: encoded.len() as u64,
            maximum_bytes: policy.max_encoded_content_bytes(),
        });
    }
    let coding = select_content_coding(&fields.values("content-encoding"))?;
    let bytes = match coding {
        ContentCoding::Identity => {
            enforce_decoded_limits(encoded.len(), encoded.len(), policy)?;
            encoded.to_vec()
        }
        ContentCoding::Gzip => decode_reader(GzDecoder::new(encoded), encoded.len(), policy)?,
        ContentCoding::Deflate => decode_reader(ZlibDecoder::new(encoded), encoded.len(), policy)?,
    };
    Ok(DecodedContent { bytes, coding })
}

fn select_content_coding(values: &[&[u8]]) -> Result<ContentCoding, HttpError> {
    if values.is_empty() {
        return Ok(ContentCoding::Identity);
    }
    let mut selected = ContentCoding::Identity;
    let mut selected_one = false;
    for value in values {
        for member in value.split(|byte| *byte == b',') {
            let member = trim_optional_whitespace(member);
            let coding = if member.eq_ignore_ascii_case(b"identity") {
                ContentCoding::Identity
            } else if member.eq_ignore_ascii_case(b"gzip") {
                ContentCoding::Gzip
            } else if member.eq_ignore_ascii_case(b"deflate") {
                ContentCoding::Deflate
            } else {
                return Err(HttpError::UnsupportedContentCoding);
            };
            if selected_one {
                return Err(HttpError::UnsupportedContentCoding);
            }
            selected = coding;
            selected_one = true;
        }
    }
    // A non-empty value slice always contributes at least one split member. Empty members are
    // rejected above, so successful parsing has assigned exactly one coding without a nullable
    // end state.
    Ok(selected)
}

fn decode_reader<R: Read>(
    mut reader: R,
    encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    let mut decoded = Vec::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let byte_count = reader.read(&mut buffer).map_err(content_decoding_error)?;
        if byte_count == 0 {
            break;
        }
        // The decoder scratch buffer is fixed at 8 KiB and the configured decoded-content
        // budget is bounded far below `usize::MAX`, so saturation preserves the fail-closed
        // result without carrying an unreachable arithmetic-error branch.
        let next_length = decoded.len().saturating_add(byte_count);
        enforce_decoded_limits(next_length, encoded_bytes, policy)?;
        decoded.extend_from_slice(&buffer[..byte_count]);
    }
    Ok(decoded)
}

fn enforce_decoded_limits(
    decoded_bytes: usize,
    encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<(), HttpError> {
    if decoded_bytes > policy.max_decoded_content_bytes() {
        return Err(HttpError::DecodedContentTooLarge {
            byte_count: decoded_bytes,
            maximum_bytes: policy.max_decoded_content_bytes(),
        });
    }
    let ratio_limit = encoded_bytes.saturating_mul(policy.max_content_expansion_ratio());
    if decoded_bytes > ratio_limit {
        return Err(HttpError::ContentExpansionRatioExceeded {
            decoded_bytes,
            encoded_bytes,
            maximum_ratio: policy.max_content_expansion_ratio(),
        });
    }
    Ok(())
}

fn content_decoding_error(source: io::Error) -> HttpError {
    HttpError::ContentDecodingFailed { source }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use std::io::Write;
    use std::time::Duration;

    use flate2::Compression;
    use flate2::write::{GzEncoder, ZlibEncoder};

    use crate::field::FieldLine;
    use crate::{AlpnHttp11Policy, IntegrityRequirement};

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

    fn policy(
        maximum_encoded_bytes: usize,
        maximum_decoded_bytes: usize,
        maximum_ratio: usize,
    ) -> HttpClientPolicy {
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
            maximum_decoded_bytes,
            maximum_ratio,
            AlpnHttp11Policy::RequireHttp11,
            IntegrityRequirement::Optional,
        )
        .expect("content policy")
    }

    fn gzip(input: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(input).expect("gzip input");
        encoder.finish().expect("gzip finish")
    }

    fn deflate(input: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(input).expect("deflate input");
        encoder.finish().expect("deflate finish")
    }

    #[test]
    fn absent_and_identity_content_coding_preserve_bytes() {
        let input = b"plain content";
        for field_block in [
            FieldBlock::default(),
            fields(&[("content-encoding", b" identity \t")]),
        ] {
            let decoded =
                decode_content(input, &field_block, &policy(64, 64, 2)).expect("identity content");
            assert_eq!(decoded.bytes, input);
            assert_eq!(decoded.coding, ContentCoding::Identity);
        }
    }

    #[test]
    fn gzip_and_zlib_deflate_decode_known_content() {
        let input = b"deterministic compressed content";
        for (encoded, name, expected_coding) in [
            (gzip(input), b"gzip".as_slice(), ContentCoding::Gzip),
            (
                deflate(input),
                b"DEFLATE".as_slice(),
                ContentCoding::Deflate,
            ),
        ] {
            let decoded = decode_content(
                &encoded,
                &fields(&[("content-encoding", name)]),
                &policy(1_024, 1_024, 32),
            )
            .expect("decoded content");
            assert_eq!(decoded.bytes, input);
            assert_eq!(decoded.coding, expected_coding);
        }
    }

    #[test]
    fn multiple_empty_and_unknown_codings_are_rejected() {
        for entries in [
            vec![("content-encoding", b"".as_slice())],
            vec![("content-encoding", b"br".as_slice())],
            vec![("content-encoding", b"gzip, deflate".as_slice())],
            vec![
                ("content-encoding", b"gzip".as_slice()),
                ("content-encoding", b"identity".as_slice()),
            ],
        ] {
            assert!(matches!(
                decode_content(b"bytes", &fields(&entries), &policy(64, 64, 4)),
                Err(HttpError::UnsupportedContentCoding)
            ));
        }
    }

    #[test]
    fn corrupt_compressed_streams_preserve_decoder_failure_as_source() {
        for coding in [b"gzip".as_slice(), b"deflate".as_slice()] {
            let error = decode_content(
                b"not-a-compressed-stream",
                &fields(&[("content-encoding", coding)]),
                &policy(64, 64, 4),
            )
            .expect_err("invalid compressed stream");
            assert!(matches!(error, HttpError::ContentDecodingFailed { .. }));
            assert!(std::error::Error::source(&error).is_some());
        }
    }

    #[test]
    fn encoded_decoded_and_expansion_budgets_fail_closed() {
        assert!(matches!(
            decode_content(b"12345", &FieldBlock::default(), &policy(4, 8, 2),),
            Err(HttpError::EncodedContentTooLarge {
                byte_count: 5,
                maximum_bytes: 4,
            })
        ));
        assert!(matches!(
            decode_content(b"12345", &FieldBlock::default(), &policy(8, 4, 2),),
            Err(HttpError::DecodedContentTooLarge {
                byte_count: 5,
                maximum_bytes: 4,
            })
        ));

        let expanded = vec![b'a'; 4_096];
        let encoded = gzip(&expanded);
        assert!(encoded.len() < expanded.len());
        assert!(matches!(
            decode_content(
                &encoded,
                &fields(&[("content-encoding", b"gzip")]),
                &policy(8_192, 8_192, 2),
            ),
            Err(HttpError::ContentExpansionRatioExceeded { .. })
        ));
    }

    #[test]
    fn exact_decoded_and_ratio_boundaries_are_accepted() {
        let input = b"1234";
        let decoded = decode_content(input, &FieldBlock::default(), &policy(4, 4, 1))
            .expect("exact identity boundary");
        assert_eq!(decoded.bytes, input);

        let compressed = gzip(b"abcdefgh");
        let decoded = decode_content(
            &compressed,
            &fields(&[("content-encoding", b"gzip")]),
            &policy(compressed.len(), 8, 1),
        )
        .expect("encoded input larger than decoded output");
        assert_eq!(decoded.bytes, b"abcdefgh");
    }
}
