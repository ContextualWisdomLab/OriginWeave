//! HTTP/1.1 response body-length and message-framing decisions.

use crate::field::{FieldBlock, trim_optional_whitespace};
use crate::{HttpError, HttpMethod};

/// The framing selected for one strict HTTP/1.1 response message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFraming {
    /// Response semantics expose no message content.
    NoContent,
    /// The message content has one validated platform-sized decimal length.
    ContentLength(usize),
    /// The message content uses the strict chunked transfer-coding profile.
    Chunked,
    /// Message content ends only when the authenticated stream closes cleanly.
    CloseDelimited,
}

pub(crate) fn determine_body_framing(
    method: HttpMethod,
    status_code: u16,
    fields: &FieldBlock,
    maximum_encoded_bytes: usize,
) -> Result<BodyFraming, HttpError> {
    let transfer_encoding_values = fields.values("transfer-encoding");
    let content_length_values = fields.values("content-length");
    if !transfer_encoding_values.is_empty() && !content_length_values.is_empty() {
        return Err(HttpError::TransferEncodingWithContentLength);
    }

    let transfer_coding = parse_transfer_encoding(&transfer_encoding_values)?;
    let suppresses_content = method.suppresses_response_content()
        || (100..200).contains(&status_code)
        || matches!(status_code, 204 | 304);
    let content_length = parse_content_length(
        &content_length_values,
        if suppresses_content {
            None
        } else {
            Some(maximum_encoded_bytes)
        },
    )?;
    if suppresses_content {
        return Ok(BodyFraming::NoContent);
    }
    if transfer_coding {
        return Ok(BodyFraming::Chunked);
    }
    if let Some(content_length) = content_length {
        return Ok(BodyFraming::ContentLength(content_length));
    }
    Ok(BodyFraming::CloseDelimited)
}

fn parse_transfer_encoding(values: &[&[u8]]) -> Result<bool, HttpError> {
    if values.is_empty() {
        return Ok(false);
    }
    let mut token_count = 0_usize;
    for value in values {
        for member in value.split(|byte| *byte == b',') {
            let member = trim_optional_whitespace(member);
            if member.is_empty() || !member.eq_ignore_ascii_case(b"chunked") {
                return Err(HttpError::UnsupportedTransferCoding);
            }
            token_count += 1;
        }
    }
    if token_count != 1 {
        return Err(HttpError::UnsupportedTransferCoding);
    }
    Ok(true)
}

fn parse_content_length(
    values: &[&[u8]],
    maximum_encoded_bytes: Option<usize>,
) -> Result<Option<usize>, HttpError> {
    if values.is_empty() {
        return Ok(None);
    }
    let mut canonical_digits = None;
    let mut parsed_length = None;
    for value in values {
        for member in value.split(|byte| *byte == b',') {
            let member = trim_optional_whitespace(member);
            let canonical_member = canonical_decimal(member)?;
            match canonical_digits {
                Some(existing) if existing != canonical_member => {
                    return Err(HttpError::ConflictingContentLength);
                }
                Some(_existing) => {}
                None => canonical_digits = Some(canonical_member),
            }
            if let Some(maximum_encoded_bytes) = maximum_encoded_bytes {
                let parsed = parse_decimal_usize(member)?;
                if parsed > maximum_encoded_bytes {
                    return Err(HttpError::EncodedContentTooLarge {
                        byte_count: u64::try_from(parsed).unwrap_or(u64::MAX),
                        maximum_bytes: maximum_encoded_bytes,
                    });
                }
                parsed_length = Some(parsed);
            }
        }
    }
    // When response semantics suppress content, decimal syntax and duplicate equality still
    // matter, but no platform-sized body length is required or returned.
    Ok(parsed_length)
}

fn canonical_decimal(input: &[u8]) -> Result<&[u8], HttpError> {
    if input.is_empty() || !input.iter().all(u8::is_ascii_digit) {
        return Err(HttpError::InvalidContentLength);
    }
    Ok(input
        .iter()
        .position(|byte| *byte != b'0')
        .map_or(&input[input.len() - 1..], |index| &input[index..]))
}

fn parse_decimal_usize(input: &[u8]) -> Result<usize, HttpError> {
    input.iter().try_fold(0_usize, |value, byte| {
        value
            .checked_mul(10)
            .and_then(|scaled| scaled.checked_add(usize::from(*byte - b'0')))
            .ok_or(HttpError::InvalidContentLength)
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use crate::field::FieldLine;

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

    #[test]
    fn method_and_status_semantics_suppress_content_after_field_validation() {
        for (method, status) in [
            (HttpMethod::Head, 200),
            (HttpMethod::Get, 100),
            (HttpMethod::Get, 199),
            (HttpMethod::Get, 204),
            (HttpMethod::Get, 304),
        ] {
            assert_eq!(
                determine_body_framing(
                    method,
                    status,
                    &fields(&[("content-length", b"999")]),
                    1_000,
                )
                .expect("no-content framing"),
                BodyFraming::NoContent
            );
        }
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Head,
                200,
                &fields(&[("content-length", b"not-a-number")]),
                1_000,
            ),
            Err(HttpError::InvalidContentLength)
        ));
    }

    #[test]
    fn no_content_semantics_do_not_apply_body_size_budget_to_content_length_metadata() {
        for (method, status) in [(HttpMethod::Head, 200), (HttpMethod::Get, 304)] {
            for metadata in [b"1001".as_slice(), b"18446744073709551616", b"000, 0"] {
                assert_eq!(
                    determine_body_framing(
                        method,
                        status,
                        &fields(&[("content-length", metadata)]),
                        1_000,
                    )
                    .expect("valid no-content metadata must not consume the body budget"),
                    BodyFraming::NoContent
                );
            }
        }
    }

    #[test]
    fn transfer_encoding_and_content_length_are_always_ambiguous() {
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Head,
                200,
                &fields(&[("transfer-encoding", b"chunked"), ("content-length", b"0"),]),
                1_000,
            ),
            Err(HttpError::TransferEncodingWithContentLength)
        ));
    }

    #[test]
    fn only_one_exact_chunked_transfer_coding_is_supported() {
        assert_eq!(
            determine_body_framing(
                HttpMethod::Get,
                200,
                &fields(&[("transfer-encoding", b" CHUNKED \t")]),
                1_000,
            )
            .expect("chunked framing"),
            BodyFraming::Chunked
        );
        for values in [
            vec![("transfer-encoding", b"gzip".as_slice())],
            vec![("transfer-encoding", b"gzip, chunked".as_slice())],
            vec![("transfer-encoding", b"chunked, chunked".as_slice())],
            vec![("transfer-encoding", b"".as_slice())],
            vec![
                ("transfer-encoding", b"chunked".as_slice()),
                ("transfer-encoding", b"chunked".as_slice()),
            ],
        ] {
            assert!(matches!(
                determine_body_framing(HttpMethod::Get, 200, &fields(&values), 1_000,),
                Err(HttpError::UnsupportedTransferCoding)
            ));
        }
    }

    #[test]
    fn identical_content_lengths_select_one_bounded_length() {
        for entries in [
            vec![("content-length", b"42".as_slice())],
            vec![("content-length", b"42, 42".as_slice())],
            vec![("content-length", b"042, 42".as_slice())],
        ] {
            assert_eq!(
                determine_body_framing(HttpMethod::Get, 200, &fields(&entries), 100,)
                    .expect("content length"),
                BodyFraming::ContentLength(42)
            );
        }
    }

    #[test]
    fn malformed_conflicting_and_excessive_content_lengths_fail() {
        for invalid in [
            b"".as_slice(),
            b"+1",
            b"-1",
            b"1 0",
            b"1,,1",
            b"18446744073709551616",
        ] {
            assert!(matches!(
                determine_body_framing(
                    HttpMethod::Get,
                    200,
                    &fields(&[("content-length", invalid)]),
                    100,
                ),
                Err(HttpError::InvalidContentLength)
            ));
        }
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Get,
                200,
                &fields(&[("content-length", b"41, 42")]),
                100,
            ),
            Err(HttpError::ConflictingContentLength)
        ));
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Get,
                200,
                &fields(&[("content-length", b"101")]),
                100,
            ),
            Err(HttpError::EncodedContentTooLarge {
                byte_count: 101,
                maximum_bytes: 100,
            })
        ));
    }

    #[test]
    fn absent_framing_fields_select_connection_close() {
        assert_eq!(
            determine_body_framing(HttpMethod::Get, 200, &FieldBlock::default(), 100)
                .expect("close-delimited framing"),
            BodyFraming::CloseDelimited
        );
    }
}
