//! Encoded-body collection and bounded HTTP content-coding decoding.

use std::io::{self, Read};

use flate2::bufread::{DeflateDecoder, MultiGzDecoder, ZlibDecoder};

use crate::field::{FieldBlock, trim_optional_whitespace};
use crate::{HttpClientPolicy, HttpError};

/// A normalized non-empty HTTP content-coding name admitted by OriginWeave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCodingName {
    /// The gzip content coding defined by RFC 9110 and RFC 1952.
    Gzip,
    /// The deflate content coding defined by RFC 9110 and RFC 1950/RFC 1951.
    Deflate,
}

/// The decoder path that actually produced one admitted content-coding layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDecoderOutcome {
    /// The standards-defined decoder path completed successfully.
    Standard,
    /// A `deflate` layer required the bounded raw RFC 1951 compatibility fallback.
    RawDeflateCompatibility,
}

/// Immutable audit evidence for one non-empty content-coding layer.
///
/// The variants deliberately make an impossible `gzip` + raw-DEFLATE combination
/// unrepresentable while accessors keep declared coding and decoder outcome separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCodingLayerEvidence {
    /// A gzip layer completed through its standards-defined decoder.
    Gzip,
    /// A deflate layer completed through its standards-defined zlib wrapper.
    Deflate,
    /// A declared deflate layer required the bounded raw RFC 1951 compatibility fallback.
    DeflateRawCompatibility,
}

impl ContentCodingLayerEvidence {
    /// Return the normalized coding declared for this wire-order layer.
    #[must_use]
    pub const fn coding(self) -> ContentCodingName {
        match self {
            Self::Gzip => ContentCodingName::Gzip,
            Self::Deflate | Self::DeflateRawCompatibility => ContentCodingName::Deflate,
        }
    }

    /// Return the decoder path that actually completed this layer.
    #[must_use]
    pub const fn outcome(self) -> ContentDecoderOutcome {
        match self {
            Self::Gzip | Self::Deflate => ContentDecoderOutcome::Standard,
            Self::DeflateRawCompatibility => ContentDecoderOutcome::RawDeflateCompatibility,
        }
    }
}

/// Bounded content-decoding evidence retained by one HTTP exchange.
///
/// `Identity` represents zero admitted non-empty codings. Single-coding variants preserve the
/// original public contract, while `Stacked` retains the exact two-layer wire/application order
/// together with the decoder outcome for each layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCoding {
    /// No non-empty content coding changes the message content bytes.
    Identity,
    /// One gzip layer completed through its standards-defined decoder.
    Gzip,
    /// One `deflate` layer completed through the standards-defined zlib wrapper.
    Deflate,
    /// One non-conforming `deflate` layer required the bounded raw RFC 1951 fallback.
    DeflateRawCompatibility,
    /// Two admitted layers in exact wire/application order, each with its actual decoder outcome.
    Stacked(ContentCodingLayerEvidence, ContentCodingLayerEvidence),
}

impl ContentCoding {
    /// Return the number of admitted non-empty content-coding layers retained in evidence.
    #[must_use]
    pub const fn layer_count(self) -> usize {
        match self {
            Self::Identity => 0,
            Self::Gzip | Self::Deflate | Self::DeflateRawCompatibility => 1,
            Self::Stacked(_, _) => 2,
        }
    }

    /// Return one layer by exact wire/application-order index.
    #[must_use]
    pub const fn layer(self, index: usize) -> Option<ContentCodingLayerEvidence> {
        match (self, index) {
            (Self::Gzip, 0) => Some(ContentCodingLayerEvidence::Gzip),
            (Self::Deflate, 0) => Some(ContentCodingLayerEvidence::Deflate),
            (Self::DeflateRawCompatibility, 0) => {
                Some(ContentCodingLayerEvidence::DeflateRawCompatibility)
            }
            (Self::Stacked(first, _), 0) => Some(first),
            (Self::Stacked(_, second), 1) => Some(second),
            _ => None,
        }
    }

    const fn from_single_layer(layer: ContentCodingLayerEvidence) -> Self {
        match layer {
            ContentCodingLayerEvidence::Gzip => Self::Gzip,
            ContentCodingLayerEvidence::Deflate => Self::Deflate,
            ContentCodingLayerEvidence::DeflateRawCompatibility => Self::DeflateRawCompatibility,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedContentCoding {
    Gzip,
    Deflate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentCodingChain {
    Identity,
    One(SelectedContentCoding),
    Two(SelectedContentCoding, SelectedContentCoding),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentCodingSelection {
    Empty,
    Identity,
    One(SelectedContentCoding),
    Two(SelectedContentCoding, SelectedContentCoding),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedLayer {
    bytes: Vec<u8>,
    evidence: ContentCodingLayerEvidence,
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

    let original_encoded_bytes = encoded.len();
    match select_content_coding_chain(&fields.values("content-encoding"))? {
        ContentCodingChain::Identity => {
            enforce_decoded_limits(encoded.len(), original_encoded_bytes, policy)?;
            Ok(DecodedContent {
                bytes: encoded.to_vec(),
                coding: ContentCoding::Identity,
            })
        }
        ContentCodingChain::One(coding) => {
            let decoded =
                decode_one_content_coding(encoded, coding, original_encoded_bytes, policy)?;
            Ok(DecodedContent {
                bytes: decoded.bytes,
                coding: ContentCoding::from_single_layer(decoded.evidence),
            })
        }
        ContentCodingChain::Two(first, second) => {
            let outer =
                decode_one_content_coding(encoded, second, original_encoded_bytes, policy)?;
            let inner =
                decode_one_content_coding(&outer.bytes, first, original_encoded_bytes, policy)?;
            Ok(DecodedContent {
                bytes: inner.bytes,
                coding: ContentCoding::Stacked(inner.evidence, outer.evidence),
            })
        }
    }
}

fn decode_one_content_coding(
    encoded: &[u8],
    coding: SelectedContentCoding,
    original_encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<DecodedLayer, HttpError> {
    if encoded.is_empty() {
        return Err(content_decoding_error(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "content coding has no coded bytes",
        )));
    }

    match coding {
        SelectedContentCoding::Gzip => Ok(DecodedLayer {
            bytes: decode_gzip(encoded, original_encoded_bytes, policy)?,
            evidence: ContentCodingLayerEvidence::Gzip,
        }),
        SelectedContentCoding::Deflate => {
            decode_deflate(encoded, original_encoded_bytes, policy)
        }
    }
}

fn decode_gzip(
    encoded: &[u8],
    original_encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    let mut decoder = MultiGzDecoder::new(encoded);
    decode_reader(&mut decoder, original_encoded_bytes, policy)
}

fn decode_deflate(
    encoded: &[u8],
    original_encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<DecodedLayer, HttpError> {
    let zlib_error = match decode_zlib_deflate(encoded, original_encoded_bytes, policy) {
        Ok(bytes) => {
            return Ok(DecodedLayer {
                bytes,
                evidence: ContentCodingLayerEvidence::Deflate,
            });
        }
        Err(HttpError::ContentDecodingFailed { source }) => source,
        Err(error) => return Err(error),
    };

    match decode_raw_deflate(encoded, original_encoded_bytes, policy) {
        Ok(bytes) => Ok(DecodedLayer {
            bytes,
            evidence: ContentCodingLayerEvidence::DeflateRawCompatibility,
        }),
        Err(HttpError::ContentDecodingFailed { .. }) => {
            Err(HttpError::ContentDecodingFailed { source: zlib_error })
        }
        Err(error) => Err(error),
    }
}

fn decode_zlib_deflate(
    encoded: &[u8],
    original_encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    let mut decoder = ZlibDecoder::new(encoded);
    let decoded = decode_reader(&mut decoder, original_encoded_bytes, policy)?;
    if !decoder.into_inner().is_empty() {
        return Err(content_decoding_error(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected bytes after the zlib stream",
        )));
    }
    Ok(decoded)
}

fn decode_raw_deflate(
    encoded: &[u8],
    original_encoded_bytes: usize,
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    let mut decoder = DeflateDecoder::new(encoded);
    let decoded = decode_reader(&mut decoder, original_encoded_bytes, policy)?;
    if !decoder.into_inner().is_empty() {
        return Err(content_decoding_error(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected bytes after the raw DEFLATE stream",
        )));
    }
    Ok(decoded)
}

fn select_content_coding_chain(values: &[&[u8]]) -> Result<ContentCodingChain, HttpError> {
    let mut selection = ContentCodingSelection::Empty;

    for value in values {
        for member in value.split(|byte| *byte == b',') {
            let member = trim_optional_whitespace(member);
            if member.is_empty() {
                continue;
            }
            if member.eq_ignore_ascii_case(b"identity") {
                selection = match selection {
                    ContentCodingSelection::Empty => ContentCodingSelection::Identity,
                    ContentCodingSelection::Identity
                    | ContentCodingSelection::One(_)
                    | ContentCodingSelection::Two(_, _) => {
                        return Err(HttpError::UnsupportedContentCoding);
                    }
                };
                continue;
            }
            let coding = if member.eq_ignore_ascii_case(b"gzip") {
                SelectedContentCoding::Gzip
            } else if member.eq_ignore_ascii_case(b"deflate") {
                SelectedContentCoding::Deflate
            } else {
                return Err(HttpError::UnsupportedContentCoding);
            };
            selection = match selection {
                ContentCodingSelection::Empty => ContentCodingSelection::One(coding),
                ContentCodingSelection::One(first) => ContentCodingSelection::Two(first, coding),
                ContentCodingSelection::Identity | ContentCodingSelection::Two(_, _) => {
                    return Err(HttpError::UnsupportedContentCoding);
                }
            };
        }
    }

    Ok(match selection {
        ContentCodingSelection::Empty | ContentCodingSelection::Identity => {
            ContentCodingChain::Identity
        }
        ContentCodingSelection::One(coding) => ContentCodingChain::One(coding),
        ContentCodingSelection::Two(first, second) => ContentCodingChain::Two(first, second),
    })
}

fn decode_reader<R: Read>(
    mut reader: R,
    original_encoded_bytes: usize,
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
        enforce_decoded_limits(next_length, original_encoded_bytes, policy)?;
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
    let ratio_limit = encoded_bytes
        .max(1)
        .saturating_mul(policy.max_content_expansion_ratio());
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
    use std::io::Write;
    use std::time::Duration;

    use flate2::Compression;
    use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};

    use crate::field::FieldLine;
    use crate::{AlpnHttp11Policy, IntegrityRequirement};

    use super::*;

    fn fields(entries: &[(&str, &[u8])]) -> Result<FieldBlock, String> {
        let mut lines = Vec::with_capacity(entries.len());
        for (name, value) in entries {
            lines.push(
                FieldLine::new(name.as_bytes(), value, 256, 8_192)
                    .map_err(|error| format!("field rejected: {error:?}"))?,
            );
        }
        Ok(FieldBlock::new(lines))
    }

    fn policy(
        maximum_encoded_bytes: usize,
        maximum_decoded_bytes: usize,
        maximum_ratio: usize,
    ) -> Result<HttpClientPolicy, String> {
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
        .map_err(|error| format!("content policy rejected: {error:?}"))
    }

    fn gzip(input: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(input)
            .map_err(|error| format!("gzip input: {error:?}"))?;
        encoder
            .finish()
            .map_err(|error| format!("gzip finish: {error:?}"))
    }

    fn deflate(input: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(input)
            .map_err(|error| format!("deflate input: {error:?}"))?;
        encoder
            .finish()
            .map_err(|error| format!("deflate finish: {error:?}"))
    }

    fn raw_deflate(input: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(input)
            .map_err(|error| format!("raw deflate input: {error:?}"))?;
        encoder
            .finish()
            .map_err(|error| format!("raw deflate finish: {error:?}"))
    }

    #[test]
    fn absent_and_identity_content_coding_preserve_bytes() -> Result<(), String> {
        let input = b"plain content";
        for field_block in [
            FieldBlock::default(),
            fields(&[("content-encoding", b" identity \t")])?,
            fields(&[("content-encoding", b", ,")])?,
        ] {
            let decoded = decode_content(input, &field_block, &policy(64, 64, 2)?)
                .map_err(|error| format!("identity content: {error:?}"))?;
            assert_eq!(decoded.bytes, input);
            assert_eq!(decoded.coding, ContentCoding::Identity);
            assert_eq!(decoded.coding.layer_count(), 0);
            assert_eq!(decoded.coding.layer(0), None);
        }
        Ok(())
    }

    #[test]
    fn gzip_and_zlib_deflate_decode_known_content() -> Result<(), String> {
        let input = b"deterministic compressed content";
        for (encoded, name, expected_coding, expected_layer) in [
            (
                gzip(input)?,
                b"gzip".as_slice(),
                ContentCoding::Gzip,
                ContentCodingLayerEvidence::Gzip,
            ),
            (
                deflate(input)?,
                b"DEFLATE".as_slice(),
                ContentCoding::Deflate,
                ContentCodingLayerEvidence::Deflate,
            ),
        ] {
            let decoded = decode_content(
                &encoded,
                &fields(&[("content-encoding", name)])?,
                &policy(1_024, 1_024, 32)?,
            )
            .map_err(|error| format!("decoded content: {error:?}"))?;
            assert_eq!(decoded.bytes, input);
            assert_eq!(decoded.coding, expected_coding);
            assert_eq!(decoded.coding.layer_count(), 1);
            assert_eq!(decoded.coding.layer(0), Some(expected_layer));
            assert_eq!(expected_layer.outcome(), ContentDecoderOutcome::Standard);
        }
        Ok(())
    }

    #[test]
    fn two_supported_codings_decode_in_reverse_application_order() -> Result<(), String> {
        let input = b"two supported content codings";
        let gzip_first = gzip(input)?;
        let wire_body = deflate(&gzip_first)?;
        let decoded = decode_content(
            &wire_body,
            &fields(&[("content-encoding", b"gzip, , deflate,")])?,
            &policy(1_024, 1_024, 32)?,
        )
        .map_err(|error| format!("stacked content: {error:?}"))?;
        assert_eq!(decoded.bytes, input);
        assert_eq!(decoded.coding.layer_count(), 2);
        assert_eq!(
            decoded.coding.layer(0),
            Some(ContentCodingLayerEvidence::Gzip)
        );
        assert_eq!(
            decoded.coding.layer(1),
            Some(ContentCodingLayerEvidence::Deflate)
        );
        assert_eq!(decoded.coding.layer(2), None);
        Ok(())
    }

    #[test]
    fn raw_deflate_compatibility_preserves_bounded_decoding() -> Result<(), String> {
        let input = b"legacy raw deflate compatibility";
        let encoded = raw_deflate(input)?;
        let decoded = decode_content(
            &encoded,
            &fields(&[("content-encoding", b"deflate")])?,
            &policy(1_024, 1_024, 32)?,
        )
        .map_err(|error| format!("raw deflate compatibility: {error:?}"))?;
        assert_eq!(decoded.bytes, input);
        assert_eq!(decoded.coding, ContentCoding::DeflateRawCompatibility);
        let layer = decoded
            .coding
            .layer(0)
            .ok_or_else(|| "raw deflate evidence layer was missing".to_owned())?;
        assert_eq!(layer.coding(), ContentCodingName::Deflate);
        assert_eq!(
            layer.outcome(),
            ContentDecoderOutcome::RawDeflateCompatibility
        );
        Ok(())
    }

    #[test]
    fn deflate_fallback_never_bypasses_decoded_budgets() -> Result<(), String> {
        let expanded = vec![b'a'; 4_096];
        for encoded in [deflate(&expanded)?, raw_deflate(&expanded)?] {
            assert!(matches!(
                decode_content(
                    &encoded,
                    &fields(&[("content-encoding", b"deflate")])?,
                    &policy(8_192, 8_192, 2)?,
                ),
                Err(HttpError::ContentExpansionRatioExceeded { .. })
            ));
        }
        Ok(())
    }

    #[test]
    fn deflate_streams_reject_trailing_bytes() -> Result<(), String> {
        for mut encoded in [deflate(b"zlib")?, raw_deflate(b"raw")?] {
            encoded.extend_from_slice(b"trailing");
            assert!(matches!(
                decode_content(
                    &encoded,
                    &fields(&[("content-encoding", b"deflate")])?,
                    &policy(1_024, 1_024, 32)?,
                ),
                Err(HttpError::ContentDecodingFailed { .. })
            ));
        }
        Ok(())
    }

    #[test]
    fn gzip_accepts_concatenated_members_and_rejects_trailing_bytes() -> Result<(), String> {
        let first = gzip(b"first member")?;
        let second = gzip(b"second member")?;
        let mut concatenated = first.clone();
        concatenated.extend_from_slice(&second);
        let decoded = decode_content(
            &concatenated,
            &fields(&[("content-encoding", b"gzip")])?,
            &policy(1_024, 1_024, 32)?,
        )
        .map_err(|error| format!("RFC 1952 concatenated gzip members: {error:?}"))?;
        assert_eq!(decoded.bytes, b"first membersecond member");

        let mut trailing = first;
        trailing.extend_from_slice(b"trailing bytes");
        assert!(matches!(
            decode_content(
                &trailing,
                &fields(&[("content-encoding", b"gzip")])?,
                &policy(1_024, 1_024, 32)?,
            ),
            Err(HttpError::ContentDecodingFailed { .. })
        ));
        Ok(())
    }

    #[test]
    fn unknown_mixed_identity_and_depth_overflow_are_rejected() -> Result<(), String> {
        for entries in [
            vec![("content-encoding", b"br".as_slice())],
            vec![("content-encoding", b"gzip, identity".as_slice())],
            vec![("content-encoding", b"gzip, deflate, gzip".as_slice())],
            vec![
                ("content-encoding", b"identity".as_slice()),
                ("content-encoding", b"deflate".as_slice()),
            ],
        ] {
            assert!(matches!(
                decode_content(b"bytes", &fields(&entries)?, &policy(64, 64, 4)?),
                Err(HttpError::UnsupportedContentCoding)
            ));
        }
        Ok(())
    }

    #[test]
    fn corrupt_compressed_streams_preserve_decoder_failure_as_source() -> Result<(), String> {
        for coding in [b"gzip".as_slice(), b"deflate".as_slice()] {
            let outcome = decode_content(
                b"not-a-compressed-stream",
                &fields(&[("content-encoding", coding)])?,
                &policy(64, 64, 4)?,
            );
            let Err(error) = outcome else {
                return Err("invalid compressed stream unexpectedly decoded".to_owned());
            };
            assert!(matches!(error, HttpError::ContentDecodingFailed { .. }));
            assert!(std::error::Error::source(&error).is_some());
        }
        Ok(())
    }

    #[test]
    fn encoded_decoded_and_expansion_budgets_fail_closed() -> Result<(), String> {
        assert!(matches!(
            decode_content(b"12345", &FieldBlock::default(), &policy(4, 8, 2)?,),
            Err(HttpError::EncodedContentTooLarge {
                byte_count: 5,
                maximum_bytes: 4,
            })
        ));
        assert!(matches!(
            decode_content(b"12345", &FieldBlock::default(), &policy(8, 4, 2)?,),
            Err(HttpError::DecodedContentTooLarge {
                byte_count: 5,
                maximum_bytes: 4,
            })
        ));

        let expanded = vec![b'a'; 4_096];
        let encoded = gzip(&expanded)?;
        assert!(encoded.len() < expanded.len());
        assert!(matches!(
            decode_content(
                &encoded,
                &fields(&[("content-encoding", b"gzip")])?,
                &policy(8_192, 8_192, 2)?,
            ),
            Err(HttpError::ContentExpansionRatioExceeded { .. })
        ));
        Ok(())
    }

    #[test]
    fn zero_encoded_length_uses_one_byte_ratio_floor() -> Result<(), String> {
        let content_policy = policy(8, 8, 2)?;
        enforce_decoded_limits(2, 0, &content_policy)
            .map_err(|error| format!("one-byte ratio floor: {error:?}"))?;
        assert!(matches!(
            enforce_decoded_limits(3, 0, &content_policy),
            Err(HttpError::ContentExpansionRatioExceeded {
                decoded_bytes: 3,
                encoded_bytes: 0,
                maximum_ratio: 2,
            })
        ));
        Ok(())
    }

    #[test]
    fn exact_decoded_and_ratio_boundaries_are_accepted() -> Result<(), String> {
        let input = b"1234";
        let decoded = decode_content(input, &FieldBlock::default(), &policy(4, 4, 1)?)
            .map_err(|error| format!("exact identity boundary: {error:?}"))?;
        assert_eq!(decoded.bytes, input);

        let compressed = gzip(b"abcdefgh")?;
        let decoded = decode_content(
            &compressed,
            &fields(&[("content-encoding", b"gzip")])?,
            &policy(compressed.len(), 8, 1)?,
        )
        .map_err(|error| format!("encoded input larger than decoded output: {error:?}"))?;
        assert_eq!(decoded.bytes, b"abcdefgh");
        Ok(())
    }
}
