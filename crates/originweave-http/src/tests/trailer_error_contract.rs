use crate::chunked::parse_chunked_body;
use crate::{HttpClientPolicy, HttpError};

#[test]
fn overlong_trailer_line_reports_the_trailer_section_budget() {
    let policy = HttpClientPolicy::strict_defaults();
    let mut body = b"0\r\nX-Long: ".to_vec();
    body.extend(std::iter::repeat_n(
        b'a',
        policy.max_trailer_section_bytes(),
    ));

    assert!(matches!(
        parse_chunked_body(&body, &policy),
        Err(HttpError::TrailerSectionTooLarge {
            byte_count,
            maximum_bytes,
        }) if byte_count == policy.max_trailer_section_bytes() + 1
            && maximum_bytes == policy.max_trailer_section_bytes()
    ));
}

#[test]
fn malformed_trailer_line_endings_remain_chunked_syntax_errors() {
    let policy = HttpClientPolicy::strict_defaults();
    assert!(matches!(
        parse_chunked_body(b"0\r\nX-Test: value\rX", &policy),
        Err(HttpError::MalformedChunkedBody)
    ));
}
