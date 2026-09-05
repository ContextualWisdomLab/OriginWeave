//! Status-aware final response-head parsing over the strict syntax parser.
//!
//! The raw parser owns line syntax and bounded parsing. This layer adds HTTP semantic constraints
//! that depend on the response status while preserving the raw parser's cumulative header budget,
//! informational-response limit, and protocol-upgrade behavior.

use crate::field::FieldBlock;
use crate::response_head_raw;
use crate::{HttpClientPolicy, HttpError};

#[cfg(test)]
pub(crate) use crate::response_head_raw::parse_response_head;
pub(crate) use crate::response_head_raw::{FinalHeadParseResult, HeadParseResult, ResponseHead};

pub(crate) fn parse_final_response_head(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<FinalHeadParseResult, HttpError> {
    let mut offset = 0_usize;
    let mut interim_response_count = 0_usize;
    loop {
        match response_head_raw::parse_response_head(&input[offset..], policy)? {
            HeadParseResult::Incomplete => break,
            HeadParseResult::Complete { head, consumed } => {
                // `consumed` is an index inside `input[offset..]`, so a successful parse proves
                // `offset + consumed <= input.len()` and makes arithmetic overflow impossible.
                offset += consumed;
                if offset > policy.max_header_section_bytes() {
                    return Err(HttpError::HeaderSectionTooLarge {
                        byte_count: offset,
                        maximum_bytes: policy.max_header_section_bytes(),
                    });
                }
                if head.status_code == 101 {
                    break;
                }
                if (100..200).contains(&head.status_code) {
                    interim_response_count += 1;
                    if interim_response_count > policy.max_interim_response_count() {
                        return Err(HttpError::ExcessiveInterimResponses {
                            response_count: interim_response_count,
                            maximum_count: policy.max_interim_response_count(),
                        });
                    }
                    validate_status_framing_fields(head.status_code, &head.fields)?;
                    continue;
                }
                validate_status_framing_fields(head.status_code, &head.fields)?;
                break;
            }
        }
    }

    // The raw parser remains the authority for final assembly, incomplete-state reporting,
    // protocol-upgrade rejection, and exact cumulative accounting. The pre-scan above adds only
    // the status-specific field legality that the syntax parser intentionally does not own.
    response_head_raw::parse_final_response_head(input, policy)
}

fn validate_status_framing_fields(status_code: u16, fields: &FieldBlock) -> Result<(), HttpError> {
    if (100..200).contains(&status_code) || status_code == 204 {
        if !fields.values("transfer-encoding").is_empty() {
            return Err(HttpError::UnsupportedTransferCoding);
        }
        if !fields.values("content-length").is_empty() {
            return Err(HttpError::InvalidContentLength);
        }
    }
    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use super::*;
    use crate::{AlpnHttp11Policy, IntegrityRequirement};

    fn bounded_head_policy() -> HttpClientPolicy {
        HttpClientPolicy::new(
            Duration::from_secs(1),
            1_024,
            64,
            4,
            16,
            16,
            32,
            2,
            16,
            8,
            1_024,
            1_024,
            1_024,
            2,
            AlpnHttp11Policy::RequireHttp11,
            IntegrityRequirement::Optional,
        )
        .expect("bounded response-head policy")
    }

    #[test]
    fn checked_budget_and_raw_error_propagation_are_explicit() {
        let cumulative = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\n\r\n";
        assert!(matches!(
            parse_final_response_head(cumulative, &bounded_head_policy()),
            Err(HttpError::HeaderSectionTooLarge {
                byte_count: 44,
                maximum_bytes: 32,
            })
        ));

        assert!(matches!(
            response_head_raw::parse_final_response_head(
                b"HTTP/1.1 200 OK\n\n",
                &HttpClientPolicy::strict_defaults(),
            ),
            Err(HttpError::InvalidResponseLineEnding)
        ));
    }
}
