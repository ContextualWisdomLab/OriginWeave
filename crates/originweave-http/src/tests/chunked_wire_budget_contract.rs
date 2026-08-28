use crate::HttpClientPolicy;
use crate::chunked::{ChunkParseResult, ChunkedDecoder, MAX_CHUNK_LINE_BYTES};
use crate::exchange::{extend_chunked_wire, maximum_chunked_wire_bytes};
use crate::field::FieldBlock;

#[test]
fn default_chunked_wire_prefix_stays_below_eighteen_mib() {
    let policy = HttpClientPolicy::strict_defaults();
    let maximum = maximum_chunked_wire_bytes(&policy);
    let per_chunk_overhead = MAX_CHUNK_LINE_BYTES + 4;
    let composed_maximum = policy
        .max_encoded_content_bytes()
        .saturating_add(policy.max_chunk_count().saturating_mul(per_chunk_overhead))
        .saturating_add(policy.max_trailer_section_bytes());

    assert_eq!(maximum, composed_maximum);
    assert!(maximum < 18 * 1024 * 1024);
}

#[test]
fn chunked_wire_growth_is_rejected_before_the_buffer_can_exceed_its_budget() {
    let mut wire = vec![0_u8; 3];

    assert!(matches!(
        extend_chunked_wire(&mut wire, &[4], 3),
        Err(crate::HttpError::EncodedContentTooLarge {
            byte_count: 4,
            maximum_bytes: 3,
        })
    ));
    assert_eq!(wire, vec![0_u8; 3]);
}

#[test]
fn incremental_decoder_resumes_inside_the_trailer_section() {
    let policy = HttpClientPolicy::strict_defaults();
    let mut decoder = ChunkedDecoder::new();
    let incomplete = b"1\r\na\r\n0\r\n";

    assert!(matches!(
        decoder.parse(incomplete, &policy),
        Ok(ChunkParseResult::Incomplete)
    ));

    let complete = b"1\r\na\r\n0\r\n\r\n";
    assert!(matches!(
        decoder.parse(complete, &policy),
        Ok(ChunkParseResult::Complete(result))
            if result.content == b"a"
                && result.trailers == FieldBlock::default()
                && result.chunk_count == 2
                && result.consumed == complete.len()
    ));
}
