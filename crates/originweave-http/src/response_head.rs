//! Strict response status-line, field, and interim-response parsing.

use crate::field::{FieldBlock, FieldLine, FieldSyntaxError, trim_optional_whitespace};
use crate::{HttpClientPolicy, HttpError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseHead {
    pub(crate) status_code: u16,
    reason_phrase: Vec<u8>,
    pub(crate) fields: FieldBlock,
}

impl ResponseHead {
    pub(crate) const fn reason_phrase(&self) -> &[u8] {
        self.reason_phrase.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HeadParseResult {
    Incomplete,
    Complete { head: ResponseHead, consumed: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FinalHeadParseResult {
    Incomplete,
    Complete {
        head: ResponseHead,
        consumed: usize,
        interim_response_count: usize,
    },
}

struct HeaderScan {
    consumed: usize,
    lines: Vec<(usize, usize)>,
}

pub(crate) fn parse_response_head(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<HeadParseResult, HttpError> {
    let Some(scan) = scan_header_lines(input, policy)? else {
        return Ok(HeadParseResult::Incomplete);
    };
    let Some((&(status_start, status_end), field_ranges)) = scan.lines.split_first() else {
        return Err(HttpError::InvalidResponseStatusLine);
    };
    let (status_code, reason_phrase) = parse_status_line(&input[status_start..status_end])?;
    let mut fields = Vec::new();
    for &(line_start, line_end) in field_ranges {
        let line = &input[line_start..line_end];
        if line
            .first()
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            return Err(HttpError::ObsoleteFieldFolding);
        }
        let colon = line
            .iter()
            .position(|byte| *byte == b':')
            .ok_or(HttpError::InvalidResponseFieldName)?;
        let name = &line[..colon];
        let value = trim_optional_whitespace(&line[colon + 1..]);
        let field = FieldLine::new(
            name,
            value,
            policy.max_header_name_bytes(),
            policy.max_header_value_bytes(),
        )
        .map_err(response_field_error)?;
        let field_count = fields.len() + 1;
        if field_count > policy.max_header_field_count() {
            return Err(HttpError::ExcessiveResponseFieldCount {
                field_count,
                maximum_count: policy.max_header_field_count(),
            });
        }
        fields.push(field);
    }
    Ok(HeadParseResult::Complete {
        head: ResponseHead {
            status_code,
            reason_phrase,
            fields: FieldBlock::new(fields),
        },
        consumed: scan.consumed,
    })
}

pub(crate) fn parse_final_response_head(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<FinalHeadParseResult, HttpError> {
    let mut offset = 0_usize;
    let mut interim_response_count = 0_usize;
    loop {
        match parse_response_head(&input[offset..], policy)? {
            HeadParseResult::Incomplete => return Ok(FinalHeadParseResult::Incomplete),
            HeadParseResult::Complete { head, consumed } => {
                // `consumed` is an index inside `input[offset..]`, so a successful parse proves
                // `offset + consumed <= input.len()` and makes arithmetic overflow impossible.
                offset += consumed;
                if head.status_code == 101 {
                    return Err(HttpError::SwitchingProtocolsUnsupported);
                }
                if (100..200).contains(&head.status_code) {
                    interim_response_count += 1;
                    if interim_response_count > policy.max_interim_response_count() {
                        return Err(HttpError::ExcessiveInterimResponses {
                            response_count: interim_response_count,
                            maximum_count: policy.max_interim_response_count(),
                        });
                    }
                    continue;
                }
                return Ok(FinalHeadParseResult::Complete {
                    head,
                    consumed: offset,
                    interim_response_count,
                });
            }
        }
    }
}

fn scan_header_lines(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<Option<HeaderScan>, HttpError> {
    let mut line_start = 0_usize;
    let mut index = 0_usize;
    let mut lines = Vec::new();
    while index < input.len() {
        if index >= policy.max_header_section_bytes() {
            return Err(HttpError::HeaderSectionTooLarge {
                byte_count: index + 1,
                maximum_bytes: policy.max_header_section_bytes(),
            });
        }
        match input[index] {
            b'\r' => {
                if index + 1 == input.len() {
                    return Ok(None);
                }
                if input[index + 1] != b'\n' {
                    return Err(HttpError::InvalidResponseLineEnding);
                }
                let line_length = index - line_start;
                if line_start == 0 && line_length > policy.max_status_line_bytes() {
                    return Err(HttpError::StatusLineTooLarge {
                        byte_count: line_length,
                        maximum_bytes: policy.max_status_line_bytes(),
                    });
                }
                let consumed = index + 2;
                if consumed > policy.max_header_section_bytes() {
                    return Err(HttpError::HeaderSectionTooLarge {
                        byte_count: consumed,
                        maximum_bytes: policy.max_header_section_bytes(),
                    });
                }
                if line_length == 0 {
                    return Ok(Some(HeaderScan { consumed, lines }));
                }
                lines.push((line_start, index));
                line_start = consumed;
                index = consumed;
            }
            b'\n' => return Err(HttpError::InvalidResponseLineEnding),
            _other => index += 1,
        }
    }
    if line_start == 0 && input.len() > policy.max_status_line_bytes() {
        return Err(HttpError::StatusLineTooLarge {
            byte_count: input.len(),
            maximum_bytes: policy.max_status_line_bytes(),
        });
    }
    Ok(None)
}

fn parse_status_line(line: &[u8]) -> Result<(u16, Vec<u8>), HttpError> {
    if line.len() < 13 {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    if !line.starts_with(b"HTTP/") {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    if &line[..8] != b"HTTP/1.1" {
        return Err(HttpError::UnsupportedHttpVersion);
    }
    if line[8] != b' ' {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    let digits = &line[9..12];
    if !digits.iter().all(u8::is_ascii_digit) {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    if line[12] != b' ' {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    let reason_phrase = &line[13..];
    if !reason_phrase
        .iter()
        .copied()
        .all(crate::field::is_field_value_byte)
    {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    let status_code = u16::from(digits[0] - b'0') * 100
        + u16::from(digits[1] - b'0') * 10
        + u16::from(digits[2] - b'0');
    if !(100..=599).contains(&status_code) {
        return Err(HttpError::InvalidResponseStatusLine);
    }
    Ok((status_code, reason_phrase.to_vec()))
}

fn response_field_error(error: FieldSyntaxError) -> HttpError {
    match error {
        FieldSyntaxError::InvalidName => HttpError::InvalidResponseFieldName,
        FieldSyntaxError::InvalidValue => HttpError::InvalidResponseFieldValue,
        FieldSyntaxError::NameTooLarge {
            byte_count,
            maximum_bytes,
        } => HttpError::ResponseFieldNameTooLarge {
            byte_count,
            maximum_bytes,
        },
        FieldSyntaxError::ValueTooLarge {
            byte_count,
            maximum_bytes,
        } => HttpError::ResponseFieldValueTooLarge {
            byte_count,
            maximum_bytes,
        },
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use super::*;
    use crate::{AlpnHttp11Policy, IntegrityRequirement};

    fn policy(
        max_status_line_bytes: usize,
        max_header_field_count: usize,
        max_header_name_bytes: usize,
        max_header_value_bytes: usize,
        max_header_section_bytes: usize,
        max_interim_response_count: usize,
    ) -> HttpClientPolicy {
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1_024,
            max_status_line_bytes,
            max_header_field_count,
            max_header_name_bytes,
            max_header_value_bytes,
            max_header_section_bytes,
            max_interim_response_count,
            16,
            8,
            1_024,
            1_024,
            1_024,
            2,
            AlpnHttp11Policy::RequireHttp11,
            IntegrityRequirement::Optional,
        )
        .expect("test policy")
    }

    #[test]
    fn complete_head_preserves_status_fields_and_body_offset() {
        let input = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nX-Bytes: \tvalue\t \r\n\r\nhello";
        let parsed =
            parse_response_head(input, &HttpClientPolicy::strict_defaults()).expect("valid head");
        let expected_fields = FieldBlock::new(vec![
            FieldLine::new(b"Content-Length", b"5", 256, 8_192).expect("content-length"),
            FieldLine::new(b"X-Bytes", b"value", 256, 8_192).expect("x-bytes"),
        ]);
        assert_eq!(
            parsed,
            HeadParseResult::Complete {
                head: ResponseHead {
                    status_code: 200,
                    reason_phrase: b"OK".to_vec(),
                    fields: expected_fields,
                },
                consumed: input.len() - b"hello".len(),
            }
        );
    }

    #[test]
    fn status_line_requires_separator_before_optional_reason_phrase() {
        let complete = b"HTTP/1.1 200 \r\n\r\n";
        assert_eq!(
            parse_response_head(complete, &HttpClientPolicy::strict_defaults())
                .expect("empty reason phrase with mandatory separator is valid"),
            HeadParseResult::Complete {
                head: ResponseHead {
                    status_code: 200,
                    reason_phrase: Vec::new(),
                    fields: FieldBlock::default(),
                },
                consumed: complete.len(),
            }
        );

        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::InvalidResponseStatusLine)
        ));
    }

    #[test]
    fn bounded_prefixes_remain_incomplete_until_the_empty_line() {
        for prefix in [b"".as_slice(), b"HTTP/1.1 200", b"HTTP/1.1 200 OK\r"] {
            assert_eq!(
                parse_response_head(prefix, &HttpClientPolicy::strict_defaults())
                    .expect("bounded prefix"),
                HeadParseResult::Incomplete
            );
        }
    }

    #[test]
    fn invalid_status_lines_and_versions_fail_closed() {
        let cases = [
            (b"BAD 200 OK\r\n\r\n".as_slice(), false),
            (b"HTTP/1.0 200 OK\r\n\r\n", true),
            (b"HTTP/1.1 99 OK\r\n\r\n", false),
            (b"HTTP/1.1 600 OK\r\n\r\n", false),
            (b"HTTP/1.1 2A0 OK\r\n\r\n", false),
            (b"HTTP/1.1 200\r\n\r\n", false),
            (b"HTTP/1.1 200 OK\0\r\n\r\n", false),
        ];
        for (input, version_error) in cases {
            let error = parse_response_head(input, &HttpClientPolicy::strict_defaults())
                .expect_err("invalid status line");
            if version_error {
                assert!(matches!(error, HttpError::UnsupportedHttpVersion));
            } else {
                assert!(matches!(error, HttpError::InvalidResponseStatusLine));
            }
        }
    }

    #[test]
    fn line_endings_folding_and_field_syntax_are_strict() {
        for input in [
            b"HTTP/1.1 200 OK\n\n".as_slice(),
            b"HTTP/1.1 200 OK\rX: y\r\n\r\n",
        ] {
            assert!(matches!(
                parse_response_head(input, &HttpClientPolicy::strict_defaults()),
                Err(HttpError::InvalidResponseLineEnding)
            ));
        }
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\n folded\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::ObsoleteFieldFolding)
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nNo-Colon\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::InvalidResponseFieldName)
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nX Test: value\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::InvalidResponseFieldName)
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nX-Test: value\0\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::InvalidResponseFieldValue)
        ));
    }

    #[test]
    fn status_field_count_name_value_and_section_limits_are_exact() {
        assert!(matches!(
            parse_response_head(b"HTTP/1.1 200 OK\r\n\r\n", &policy(14, 2, 8, 8, 64, 2)),
            Err(HttpError::StatusLineTooLarge { .. })
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nA: 1\r\nB: 2\r\n\r\n",
                &policy(32, 1, 8, 8, 64, 2),
            ),
            Err(HttpError::ExcessiveResponseFieldCount { .. })
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nLong-Name: 1\r\n\r\n",
                &policy(32, 2, 4, 8, 64, 2),
            ),
            Err(HttpError::ResponseFieldNameTooLarge { .. })
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nX: 12345\r\n\r\n",
                &policy(32, 2, 8, 4, 64, 2),
            ),
            Err(HttpError::ResponseFieldValueTooLarge { .. })
        ));
        assert!(matches!(
            parse_response_head(
                b"HTTP/1.1 200 OK\r\nX: 1\r\n\r\n",
                &policy(32, 2, 8, 8, 20, 2),
            ),
            Err(HttpError::HeaderSectionTooLarge { .. })
        ));
    }

    #[test]
    fn informational_responses_are_bounded_and_upgrade_is_rejected() {
        let input = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 103 Early Hints\r\nLink: x\r\n\r\nHTTP/1.1 200 OK\r\n\r\nbody";
        assert_eq!(
            parse_final_response_head(input, &policy(64, 4, 16, 16, 128, 2)).expect("final head"),
            FinalHeadParseResult::Complete {
                head: ResponseHead {
                    status_code: 200,
                    reason_phrase: b"OK".to_vec(),
                    fields: FieldBlock::default(),
                },
                consumed: input.len() - b"body".len(),
                interim_response_count: 2,
            }
        );

        assert!(matches!(
            parse_final_response_head(input, &policy(64, 4, 16, 16, 128, 1)),
            Err(HttpError::ExcessiveInterimResponses { .. })
        ));
        assert!(matches!(
            parse_final_response_head(
                b"HTTP/1.1 101 Switching Protocols\r\n\r\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::SwitchingProtocolsUnsupported)
        ));
    }

    #[test]
    fn interim_and_final_heads_share_one_header_section_budget() {
        let input = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\n\r\n";
        assert!(matches!(
            parse_final_response_head(input, &policy(64, 4, 16, 16, 32, 2)),
            Err(HttpError::HeaderSectionTooLarge {
                byte_count: 44,
                maximum_bytes: 32,
            })
        ));
    }
}
