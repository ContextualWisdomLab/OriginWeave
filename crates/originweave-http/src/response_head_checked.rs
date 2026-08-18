//! Status-aware final response-head parsing over the strict syntax parser.
//!
//! The raw parser owns line syntax and bounded parsing. This layer adds HTTP semantic constraints
//! that depend on the response status while preserving the raw parser's cumulative header budget,
//! informational-response limit, and protocol-upgrade behavior.

use crate::field::FieldBlock;
use crate::response_head_raw::{self, HeadParseResult};
use crate::{HttpClientPolicy, HttpError};

pub(crate) use crate::response_head_raw::{FinalHeadParseResult, ResponseHead};

pub(crate) fn parse_final_response_head(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<FinalHeadParseResult, HttpError> {
    let mut offset = 0_usize;
    let mut interim_response_count = 0_usize;
    loop {
        match response_head_raw::parse_response_head(&input[offset..], policy)? {
            HeadParseResult::Incomplete => {
                if input.len() > policy.max_header_section_bytes() {
                    return Err(HttpError::HeaderSectionTooLarge {
                        byte_count: input.len(),
                        maximum_bytes: policy.max_header_section_bytes(),
                    });
                }
                return Ok(FinalHeadParseResult::Incomplete);
            }
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
                    return Err(HttpError::SwitchingProtocolsUnsupported);
                }
                validate_status_framing_fields(head.status_code, &head.fields)?;
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

fn validate_status_framing_fields(
    status_code: u16,
    fields: &FieldBlock,
) -> Result<(), HttpError> {
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
