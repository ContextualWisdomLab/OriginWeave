use std::time::Duration;

use crate::chunked::{ChunkParseResult, ChunkedDecoder};
use crate::{AlpnHttp11Policy, HttpClientPolicy, HttpError, IntegrityRequirement};

fn policy() -> Result<HttpClientPolicy, HttpError> {
    HttpClientPolicy::new(
        Duration::from_secs(1),
        1_024,
        1_024,
        16,
        64,
        256,
        1_024,
        4,
        8,
        4,
        128,
        64,
        1_024,
        4,
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
}

#[test]
fn incremental_decoder_rejects_mutated_committed_prefix() -> Result<(), HttpError> {
    let policy = policy()?;
    let mut decoder = ChunkedDecoder::new();
    assert_eq!(
        decoder.parse(b"1\r\na\r\n", &policy)?,
        ChunkParseResult::Incomplete
    );

    let mutated_committed_prefix = b"g\r\nx\r\n1\r\nb\r\n0\r\n\r\n";
    assert!(matches!(
        decoder.parse(mutated_committed_prefix, &policy),
        Err(HttpError::MalformedChunkedBody)
    ));
    Ok(())
}

#[test]
fn incremental_decoder_accepts_append_only_growth() -> Result<(), HttpError> {
    let policy = policy()?;
    let mut decoder = ChunkedDecoder::new();
    assert_eq!(
        decoder.parse(b"1\r\na\r\n", &policy)?,
        ChunkParseResult::Incomplete
    );

    let append_only = b"1\r\na\r\n1\r\nb\r\n0\r\n\r\n";
    let result = decoder.parse(append_only, &policy)?;
    assert!(
        matches!(&result, ChunkParseResult::Complete(_)),
        "append-only continuation must complete"
    );
    if let ChunkParseResult::Complete(result) = result {
        assert_eq!(result.content, b"ab");
        assert_eq!(result.consumed, append_only.len());
    }
    Ok(())
}
