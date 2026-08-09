use crate::HttpClientPolicy;
use crate::chunked::{ChunkParseResult, ChunkedDecoder, ChunkedResult};
use crate::exchange::{extend_chunked_wire, maximum_chunked_wire_bytes};
use crate::field::{FieldBlock, FieldLine};

#[test]
fn default_chunked_wire_prefix_stays_below_eighteen_mib() {
    let maximum = maximum_chunked_wire_bytes(&HttpClientPolicy::strict_defaults());
    assert_eq!(maximum, 18_104_340);
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
    let incomplete = b"1\r\na\r\n0\r\nX-Trace";

    assert_eq!(
        decoder
            .parse(incomplete, &policy)
            .expect("partial trailer remains incomplete"),
        ChunkParseResult::Incomplete
    );

    let complete = b"1\r\na\r\n0\r\nX-Trace: ok\r\n\r\n";
    let expected_trailer = FieldLine::new(b"X-Trace", b"ok", 64, 256).expect("trailer field");
    assert_eq!(
        decoder
            .parse(complete, &policy)
            .expect("trailer parsing resumes from committed decoder state"),
        ChunkParseResult::Complete(ChunkedResult {
            content: b"a".to_vec(),
            trailers: FieldBlock::new(vec![expected_trailer]),
            chunk_count: 2,
            consumed: complete.len(),
        })
    );
}
