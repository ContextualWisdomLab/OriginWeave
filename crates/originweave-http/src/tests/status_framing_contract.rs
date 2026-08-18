//! Status-specific HTTP/1.1 framing-field legality regressions.
//!
//! RFC 9110 Section 8.6 and RFC 9112 Section 6.1 forbid `Content-Length` and
//! `Transfer-Encoding`, respectively, on 1xx and 204 responses. HEAD and 304
//! responses remain allowed to carry representation-length / transfer-coding
//! metadata under the RFC-defined conditions.

use crate::response_head::{FinalHeadParseResult, parse_final_response_head};
use crate::{HttpClientPolicy, HttpError};

#[test]
fn informational_response_rejects_transfer_encoding_before_final_response() {
    let input = b"HTTP/1.1 103 Early Hints\r\nTransfer-Encoding: chunked\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

    assert!(matches!(
        parse_final_response_head(input, &HttpClientPolicy::strict_defaults()),
        Err(HttpError::UnsupportedTransferCoding)
    ));
}

#[test]
fn informational_response_rejects_content_length_before_final_response() {
    let input = b"HTTP/1.1 103 Early Hints\r\nContent-Length: 0\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

    assert!(matches!(
        parse_final_response_head(input, &HttpClientPolicy::strict_defaults()),
        Err(HttpError::InvalidContentLength)
    ));
}

#[test]
fn no_content_response_rejects_transfer_encoding() {
    let input = b"HTTP/1.1 204 No Content\r\nTransfer-Encoding: chunked\r\n\r\n";

    assert!(matches!(
        parse_final_response_head(input, &HttpClientPolicy::strict_defaults()),
        Err(HttpError::UnsupportedTransferCoding)
    ));
}

#[test]
fn no_content_response_rejects_content_length() {
    let input = b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";

    assert!(matches!(
        parse_final_response_head(input, &HttpClientPolicy::strict_defaults()),
        Err(HttpError::InvalidContentLength)
    ));
}

#[test]
fn valid_informational_response_without_framing_fields_remains_accepted() {
    let input = b"HTTP/1.1 103 Early Hints\r\nLink: </style.css>; rel=preload\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

    assert!(matches!(
        parse_final_response_head(input, &HttpClientPolicy::strict_defaults()),
        Ok(FinalHeadParseResult::Complete {
            interim_response_count: 1,
            ..
        })
    ));
}

#[test]
fn not_modified_response_keeps_rfc_permitted_framing_metadata() {
    for field in [
        b"Transfer-Encoding: chunked\r\n".as_slice(),
        b"Content-Length: 42\r\n",
    ] {
        let mut input = b"HTTP/1.1 304 Not Modified\r\n".to_vec();
        input.extend_from_slice(field);
        input.extend_from_slice(b"\r\n");

        assert!(matches!(
            parse_final_response_head(&input, &HttpClientPolicy::strict_defaults()),
            Ok(FinalHeadParseResult::Complete { .. })
        ));
    }
}
