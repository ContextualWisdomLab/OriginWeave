#![allow(clippy::expect_used)]

use crate::response_head::{HeadParseResult, parse_response_head};
use crate::HttpClientPolicy;

#[test]
fn response_head_retains_exact_reason_phrase_octets() {
    let policy = HttpClientPolicy::strict_defaults();
    let cases: &[(&[u8], &[u8])] = &[
        (b"HTTP/1.1 200 OK\r\n\r\n", b"OK"),
        (b"HTTP/1.1 204 \r\n\r\n", b""),
        (b"HTTP/1.1 500 O\xffK\r\n\r\n", b"O\xffK"),
    ];

    for (wire, expected_reason_phrase) in cases {
        match parse_response_head(wire, &policy).expect("valid RFC 9112 status line") {
            HeadParseResult::Complete { head, .. } => {
                assert_eq!(head.reason_phrase(), *expected_reason_phrase);
            }
            HeadParseResult::Incomplete => {
                assert!(false, "complete status line must not remain incomplete");
            }
        }
    }
}
