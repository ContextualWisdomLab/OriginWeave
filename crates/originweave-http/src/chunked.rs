//! Bounded HTTP/1.1 chunked transfer-coding and trailer parsing.

use crate::field::{FieldBlock, FieldLine, FieldSyntaxError};
use crate::{HttpClientPolicy, HttpError};

pub(crate) const MAX_CHUNK_LINE_BYTES: usize = 1_024;
const FORBIDDEN_TRAILER_FIELDS: &[&str] = &[
    "transfer-encoding",
    "content-length",
    "host",
    "connection",
    "trailer",
    "content-encoding",
    "content-type",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChunkedResult {
    pub(crate) content: Vec<u8>,
    pub(crate) trailers: FieldBlock,
    pub(crate) chunk_count: usize,
    pub(crate) consumed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChunkParseResult {
    Incomplete,
    Complete(ChunkedResult),
}

pub(crate) fn parse_chunked_body(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<ChunkParseResult, HttpError> {
    let mut cursor = 0_usize;
    let mut content = Vec::new();
    let mut chunk_count = 0_usize;
    loop {
        let Some(line_end) = find_crlf(input, cursor, MAX_CHUNK_LINE_BYTES)? else {
            return Ok(ChunkParseResult::Incomplete);
        };
        let size_line = &input[cursor..line_end];
        let chunk_size = parse_chunk_size(size_line)?;
        chunk_count = chunk_count
            .checked_add(1)
            .ok_or(HttpError::ExcessiveChunkCount {
                chunk_count: usize::MAX,
                maximum_count: policy.max_chunk_count(),
            })?;
        if chunk_count > policy.max_chunk_count() {
            return Err(HttpError::ExcessiveChunkCount {
                chunk_count,
                maximum_count: policy.max_chunk_count(),
            });
        }
        cursor = line_end + 2;
        if chunk_size == 0 {
            return parse_trailers(input, cursor, content, chunk_count, policy);
        }
        let next_content_length =
            content
                .len()
                .checked_add(chunk_size)
                .ok_or(HttpError::EncodedContentTooLarge {
                    byte_count: u64::MAX,
                    maximum_bytes: policy.max_encoded_content_bytes(),
                })?;
        if next_content_length > policy.max_encoded_content_bytes() {
            return Err(HttpError::EncodedContentTooLarge {
                byte_count: u64::try_from(next_content_length).unwrap_or(u64::MAX),
                maximum_bytes: policy.max_encoded_content_bytes(),
            });
        }
        let Some(data_end) = cursor.checked_add(chunk_size) else {
            return Err(HttpError::EncodedContentTooLarge {
                byte_count: u64::MAX,
                maximum_bytes: policy.max_encoded_content_bytes(),
            });
        };
        let Some(message_end) = data_end.checked_add(2) else {
            return Err(HttpError::MalformedChunkedBody);
        };
        if input.len() < message_end {
            return Ok(ChunkParseResult::Incomplete);
        }
        if &input[data_end..message_end] != b"\r\n" {
            return Err(HttpError::MalformedChunkedBody);
        }
        content.extend_from_slice(&input[cursor..data_end]);
        cursor = message_end;
    }
}

fn parse_trailers(
    input: &[u8],
    start: usize,
    content: Vec<u8>,
    chunk_count: usize,
    policy: &HttpClientPolicy,
) -> Result<ChunkParseResult, HttpError> {
    let mut cursor = start;
    let mut trailers = Vec::new();
    loop {
        let consumed_trailer_bytes = cursor.saturating_sub(start);
        if consumed_trailer_bytes >= policy.max_trailer_section_bytes() {
            return Err(HttpError::TrailerSectionTooLarge {
                byte_count: consumed_trailer_bytes + 1,
                maximum_bytes: policy.max_trailer_section_bytes(),
            });
        }
        let remaining_budget = policy
            .max_trailer_section_bytes()
            .saturating_sub(consumed_trailer_bytes);
        let Some(line_end) = find_crlf(input, cursor, remaining_budget)? else {
            return Ok(ChunkParseResult::Incomplete);
        };
        let line = &input[cursor..line_end];
        let after_line = line_end + 2;
        let total_trailer_bytes = after_line - start;
        if total_trailer_bytes > policy.max_trailer_section_bytes() {
            return Err(HttpError::TrailerSectionTooLarge {
                byte_count: total_trailer_bytes,
                maximum_bytes: policy.max_trailer_section_bytes(),
            });
        }
        if line.is_empty() {
            return Ok(ChunkParseResult::Complete(ChunkedResult {
                content,
                trailers: FieldBlock::new(trailers),
                chunk_count,
                consumed: after_line,
            }));
        }
        if line
            .first()
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            return Err(HttpError::InvalidTrailerSection);
        }
        let colon = line
            .iter()
            .position(|byte| *byte == b':')
            .ok_or(HttpError::InvalidTrailerSection)?;
        let name = &line[..colon];
        let value = trim_optional_whitespace(&line[colon + 1..]);
        let field = FieldLine::new(
            name,
            value,
            policy.max_header_name_bytes(),
            policy.max_header_value_bytes(),
        )
        .map_err(trailer_field_error)?;
        if FORBIDDEN_TRAILER_FIELDS.contains(&field.name()) {
            return Err(HttpError::InvalidTrailerSection);
        }
        let field_count = trailers.len() + 1;
        if field_count > policy.max_trailer_field_count() {
            return Err(HttpError::ExcessiveTrailerFieldCount {
                field_count,
                maximum_count: policy.max_trailer_field_count(),
            });
        }
        trailers.push(field);
        cursor = after_line;
    }
}

fn find_crlf(
    input: &[u8],
    start: usize,
    maximum_line_bytes: usize,
) -> Result<Option<usize>, HttpError> {
    let mut index = start;
    while index < input.len() {
        let line_length = index - start;
        if line_length > maximum_line_bytes {
            return Err(HttpError::ChunkLineTooLarge {
                byte_count: line_length,
                maximum_bytes: maximum_line_bytes,
            });
        }
        match input[index] {
            b'\r' => {
                if index + 1 == input.len() {
                    return Ok(None);
                }
                if input[index + 1] != b'\n' {
                    return Err(HttpError::MalformedChunkedBody);
                }
                return Ok(Some(index));
            }
            b'\n' => return Err(HttpError::MalformedChunkedBody),
            _other => index += 1,
        }
    }
    if input.len().saturating_sub(start) > maximum_line_bytes {
        return Err(HttpError::ChunkLineTooLarge {
            byte_count: input.len() - start,
            maximum_bytes: maximum_line_bytes,
        });
    }
    Ok(None)
}

fn parse_chunk_size(input: &[u8]) -> Result<usize, HttpError> {
    if input.is_empty() || !input.iter().all(u8::is_ascii_hexdigit) {
        return Err(HttpError::MalformedChunkedBody);
    }
    input.iter().try_fold(0_usize, |value, byte| {
        value
            .checked_mul(16)
            .and_then(|scaled| scaled.checked_add(usize::from(hex_value(*byte))))
            .ok_or(HttpError::MalformedChunkedBody)
    })
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _other => 0,
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

fn trailer_field_error(_error: FieldSyntaxError) -> HttpError {
    HttpError::InvalidTrailerSection
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic)]

    use std::time::Duration;

    use super::*;
    use crate::{AlpnHttp11Policy, IntegrityRequirement};

    fn policy(
        max_chunk_count: usize,
        max_trailer_field_count: usize,
        max_trailer_section_bytes: usize,
        max_encoded_content_bytes: usize,
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
            max_chunk_count,
            max_trailer_field_count,
            max_trailer_section_bytes,
            max_encoded_content_bytes,
            1_024,
            4,
            AlpnHttp11Policy::RequireHttp11,
            IntegrityRequirement::Optional,
        )
        .expect("chunk policy")
    }

    #[test]
    fn valid_chunks_trailers_and_following_bytes_are_preserved() {
        let input = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\nX-Trace: ok\r\n\r\nnext";
        let parsed = parse_chunked_body(input, &policy(8, 4, 128, 64)).expect("chunked body");
        let ChunkParseResult::Complete(result) = parsed else {
            panic!("chunked body must be complete");
        };
        assert_eq!(result.content, b"Wikipedia");
        assert_eq!(result.chunk_count, 3);
        assert_eq!(result.trailers.values("x-trace"), [b"ok".as_slice()]);
        assert_eq!(&input[result.consumed..], b"next");
    }

    #[test]
    fn uppercase_hex_and_empty_trailer_are_supported() {
        let input = b"A\r\n0123456789\r\n0\r\n\r\n";
        let parsed = parse_chunked_body(input, &policy(4, 2, 32, 16)).expect("chunked body");
        let ChunkParseResult::Complete(result) = parsed else {
            panic!("chunked body must be complete");
        };
        assert_eq!(result.content, b"0123456789");
        assert_eq!(result.trailers.len(), 0);
        assert_eq!(result.consumed, input.len());
    }

    #[test]
    fn every_proper_prefix_of_a_valid_body_is_incomplete() {
        let input = b"1\r\na\r\n0\r\n\r\n";
        for length in 0..input.len() {
            assert_eq!(
                parse_chunked_body(&input[..length], &policy(4, 2, 32, 16)).expect("valid prefix"),
                ChunkParseResult::Incomplete,
                "prefix length {length}"
            );
        }
    }

    #[test]
    fn chunk_syntax_is_strict_and_extensions_are_rejected() {
        for invalid in [
            b"\r\n".as_slice(),
            b"g\r\n",
            b"4;name=value\r\nWiki\r\n0\r\n\r\n",
            b"1\na\r\n0\r\n\r\n",
            b"1\rXa\r\n0\r\n\r\n",
            b"1\r\naXX0\r\n\r\n",
        ] {
            assert!(matches!(
                parse_chunked_body(invalid, &policy(8, 4, 128, 64)),
                Err(HttpError::MalformedChunkedBody)
            ));
        }
    }

    #[test]
    fn chunk_line_count_and_encoded_content_are_bounded() {
        let long_line = format!("{}\r\n", "1".repeat(MAX_CHUNK_LINE_BYTES + 1));
        assert!(matches!(
            parse_chunked_body(long_line.as_bytes(), &policy(8, 4, 128, 64)),
            Err(HttpError::ChunkLineTooLarge { .. })
        ));
        assert!(matches!(
            parse_chunked_body(b"1\r\na\r\n0\r\n\r\n", &policy(1, 4, 128, 64)),
            Err(HttpError::ExcessiveChunkCount {
                chunk_count: 2,
                maximum_count: 1,
            })
        ));
        assert!(matches!(
            parse_chunked_body(b"5\r\nhello\r\n0\r\n\r\n", &policy(4, 4, 128, 4)),
            Err(HttpError::EncodedContentTooLarge {
                byte_count: 5,
                maximum_bytes: 4,
            })
        ));
    }

    #[test]
    fn trailer_syntax_names_counts_and_bytes_are_bounded() {
        for invalid in [
            b"0\r\n folded\r\n\r\n".as_slice(),
            b"0\r\nNo-Colon\r\n\r\n",
            b"0\r\nBad Name: x\r\n\r\n",
            b"0\r\nContent-Length: 0\r\n\r\n",
            b"0\r\nX-Test: value\0\r\n\r\n",
        ] {
            assert!(matches!(
                parse_chunked_body(invalid, &policy(4, 4, 128, 64)),
                Err(HttpError::InvalidTrailerSection)
            ));
        }
        assert!(matches!(
            parse_chunked_body(b"0\r\nA: 1\r\nB: 2\r\n\r\n", &policy(4, 1, 128, 64),),
            Err(HttpError::ExcessiveTrailerFieldCount {
                field_count: 2,
                maximum_count: 1,
            })
        ));
        assert!(matches!(
            parse_chunked_body(b"0\r\nX: 1\r\n\r\n", &policy(4, 4, 5, 64)),
            Err(HttpError::TrailerSectionTooLarge { .. })
        ));
    }

    #[test]
    fn adversarial_numeric_and_unterminated_chunk_lines_fail_closed() {
        let maximum_size_line = format!("{:x}", usize::MAX);
        let accumulated = format!("1\r\na\r\n{maximum_size_line}\r\n");
        assert!(matches!(
            parse_chunked_body(accumulated.as_bytes(), &policy(8, 4, 128, 64)),
            Err(HttpError::EncodedContentTooLarge {
                byte_count: u64::MAX,
                maximum_bytes: 64,
            })
        ));

        let overflowing_size = "f".repeat((usize::BITS as usize / 4) + 1);
        assert!(matches!(
            parse_chunk_size(overflowing_size.as_bytes()),
            Err(HttpError::MalformedChunkedBody)
        ));

        let unterminated = vec![b'1'; MAX_CHUNK_LINE_BYTES + 2];
        assert!(matches!(
            parse_chunked_body(&unterminated, &policy(8, 4, 128, 64)),
            Err(HttpError::ChunkLineTooLarge {
                byte_count,
                maximum_bytes: MAX_CHUNK_LINE_BYTES,
            }) if byte_count == MAX_CHUNK_LINE_BYTES + 1
        ));
    }

    #[test]
    fn exact_trailer_budget_requires_room_for_the_terminal_empty_line() {
        assert!(matches!(
            parse_chunked_body(b"0\r\nX: 1\r\n", &policy(4, 4, 6, 64)),
            Err(HttpError::TrailerSectionTooLarge {
                byte_count: 7,
                maximum_bytes: 6,
            })
        ));
    }

    #[test]
    fn parser_helpers_handle_boundary_only_inputs() {
        assert_eq!(hex_value(b'g'), 0);
        assert_eq!(trim_optional_whitespace(b" \t "), b"");
    }
}
