#![allow(clippy::expect_used)]

use std::time::Duration;

use crate::response_head::parse_final_response_head;
use crate::{AlpnHttp11Policy, HttpClientPolicy, HttpError, IntegrityRequirement};

fn cumulative_header_policy() -> HttpClientPolicy {
    let defaults = HttpClientPolicy::strict_defaults();
    HttpClientPolicy::new(
        Duration::from_secs(1),
        defaults.max_request_bytes(),
        64,
        4,
        16,
        16,
        64,
        2,
        defaults.max_chunk_count(),
        defaults.max_trailer_field_count(),
        defaults.max_trailer_section_bytes(),
        defaults.max_encoded_content_bytes(),
        defaults.max_decoded_content_bytes(),
        defaults.max_content_expansion_ratio(),
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("bounded cumulative-header policy")
}

#[test]
fn incomplete_final_head_after_interim_respects_cumulative_header_budget() {
    let input = concat!(
        "HTTP/1.1 100 Continue\r\n\r\n",
        "HTTP/1.1 200 OK\r\nx:",
        "aaaaaaaaaa",
        "aaaaaaaaaa",
        "aaaaaaaaaa"
    )
    .as_bytes();

    assert!(matches!(
        parse_final_response_head(input, &cumulative_header_policy()),
        Err(HttpError::HeaderSectionTooLarge {
            byte_count: 74,
            maximum_bytes: 64,
        })
    ));
}
