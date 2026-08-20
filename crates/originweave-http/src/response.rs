use std::io::Read;
use std::time::{Duration, Instant};

use crate::error::{HttpError, HttpPolicyError, io_result};
use crate::request::HttpMethod;

/// The largest accepted HTTP/1.1 status line, excluding its CRLF.
pub const MAX_STATUS_LINE_BYTES: usize = 128;

/// The largest accepted number of response header fields.
pub const MAX_HEADER_FIELDS: usize = 64;

/// The largest accepted field name.
pub const MAX_HEADER_NAME_BYTES: usize = 64;

/// The largest accepted field value.
pub const MAX_HEADER_VALUE_BYTES: usize = 4 * 1024;

/// The largest accepted complete response header section.
pub const MAX_HEADER_SECTION_BYTES: usize = 16 * 1024;

/// The largest number of chunks accepted in one response.
pub const MAX_CHUNK_COUNT: usize = 16 * 1024;

/// The largest number of trailer fields accepted in one response.
pub const MAX_TRAILER_FIELDS: usize = 64;

/// The largest complete trailer section accepted in one response.
pub const MAX_TRAILER_SECTION_BYTES: usize = 4 * 1024;

/// The largest caller-configurable exchange duration.
pub const MAX_HTTP_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest caller-configurable response body.
pub const MAX_HTTP_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Bounded resources for one non-reusable HTTP/1.1 exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpExchangePolicy {
    exchange_timeout: Duration,
    max_body_bytes: usize,
    allow_absent_alpn: bool,
}

impl HttpExchangePolicy {
    /// Construct a policy with an explicit total deadline and body ceiling.
    pub fn new(exchange_timeout: Duration, max_body_bytes: usize) -> Result<Self, HttpPolicyError> {
        if exchange_timeout.is_zero() || exchange_timeout > MAX_HTTP_EXCHANGE_TIMEOUT {
            return Err(HttpPolicyError::InvalidExchangeTimeout {
                timeout: exchange_timeout,
                maximum: MAX_HTTP_EXCHANGE_TIMEOUT,
            });
        }
        if max_body_bytes == 0 || max_body_bytes > MAX_HTTP_BODY_BYTES {
            return Err(HttpPolicyError::InvalidBodyLimit {
                limit: max_body_bytes,
                maximum: MAX_HTTP_BODY_BYTES,
            });
        }
        Ok(Self {
            exchange_timeout,
            max_body_bytes,
            allow_absent_alpn: false,
        })
    }

    /// Permit an explicit absent-ALPN direct-test policy.
    #[must_use]
    #[inline(never)]
    pub const fn permit_absent_alpn(mut self) -> Self {
        self.allow_absent_alpn = true;
        self
    }

    /// Return the total monotonic exchange deadline budget.
    #[must_use]
    #[inline(never)]
    pub const fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }

    /// Return the decoded response body ceiling.
    #[must_use]
    #[inline(never)]
    pub const fn max_body_bytes(&self) -> usize {
        self.max_body_bytes
    }

    pub(crate) const fn allow_absent_alpn(&self) -> bool {
        self.allow_absent_alpn
    }
}

impl Default for HttpExchangePolicy {
    #[inline(never)]
    fn default() -> Self {
        Self {
            exchange_timeout: Duration::from_secs(10),
            max_body_bytes: MAX_HTTP_BODY_BYTES,
            allow_absent_alpn: false,
        }
    }
}

/// One safe, bounded response field retained for downstream policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader {
    name: String,
    value: String,
}

impl HttpHeader {
    /// Return the lower-case field name.
    #[must_use]
    #[inline(never)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the bounded field value.
    #[must_use]
    #[inline(never)]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// A complete bounded HTTP/1.1 response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    status_code: u16,
    reason_phrase: Vec<u8>,
    headers: Vec<HttpHeader>,
    body: Vec<u8>,
}

impl HttpResponse {
    /// Parse one complete HTTP/1.1 response from a bounded reader.
    #[inline(never)]
    pub fn parse(
        reader: &mut dyn Read,
        method: HttpMethod,
        policy: &HttpExchangePolicy,
    ) -> Result<Self, HttpError> {
        parse_response_until(reader, method, policy, None)
    }

    /// Return the exact three-digit status code.
    #[must_use]
    #[inline(never)]
    pub const fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Return the reason phrase bytes separately from the status code.
    #[must_use]
    #[inline(never)]
    pub fn reason_phrase(&self) -> &[u8] {
        self.reason_phrase.as_slice()
    }

    /// Return allow-listed, credential-free response fields.
    #[must_use]
    #[inline(never)]
    pub fn headers(&self) -> &[HttpHeader] {
        self.headers.as_slice()
    }

    /// Return the complete response body.
    #[must_use]
    #[inline(never)]
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }

    /// Return whether all response bytes permitted by framing were received.
    #[must_use]
    #[inline(never)]
    pub const fn is_complete(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
struct RawHeader {
    name: String,
    value: String,
}

pub(crate) fn parse_response_until(
    reader: &mut dyn Read,
    method: HttpMethod,
    policy: &HttpExchangePolicy,
    deadline: Option<Instant>,
) -> Result<HttpResponse, HttpError> {
    let mut section_bytes = 0_usize;
    let status_line = read_line(
        reader,
        MAX_STATUS_LINE_BYTES,
        &mut section_bytes,
        MAX_HEADER_SECTION_BYTES,
        deadline,
    )?;
    let (status_code, reason_phrase) = parse_status_line(&status_line)?;
    let mut headers = Vec::new();
    loop {
        let line = read_line(
            reader,
            MAX_HEADER_VALUE_BYTES + MAX_HEADER_NAME_BYTES + 4,
            &mut section_bytes,
            MAX_HEADER_SECTION_BYTES,
            deadline,
        )?;
        if line.is_empty() {
            break;
        }
        if headers.len() == MAX_HEADER_FIELDS {
            return Err(HttpError::HeaderFieldLimitExceeded);
        }
        headers.push(parse_header_line(&line)?);
    }

    let content_length = content_length(&headers)?;
    let transfer_encoding = transfer_encoding(&headers)?;
    if (300..400).contains(&status_code) && headers.iter().any(|header| header.name == "location") {
        return Err(HttpError::RedirectNotSupported);
    }
    if content_length.is_some() && transfer_encoding {
        return Err(HttpError::FramingAmbiguous);
    }
    validate_content_encoding(&headers)?;
    let no_body = method == HttpMethod::Head
        || (100..200).contains(&status_code)
        || matches!(status_code, 204 | 304);
    let body = if no_body {
        Vec::new()
    } else if transfer_encoding {
        read_chunked_body(reader, policy.max_body_bytes(), deadline)?
    } else if let Some(length) = content_length {
        read_fixed_body(reader, length, policy.max_body_bytes(), deadline)?
    } else {
        read_close_delimited_body(reader, policy.max_body_bytes(), deadline)?
    };

    let retained_headers = headers
        .into_iter()
        .filter(|header| {
            matches!(
                header.name.as_str(),
                "content-length"
                    | "content-type"
                    | "content-encoding"
                    | "transfer-encoding"
                    | "connection"
            )
        })
        .map(|header| HttpHeader {
            name: header.name,
            value: header.value,
        })
        .collect();
    Ok(HttpResponse {
        status_code,
        reason_phrase,
        headers: retained_headers,
        body,
    })
}

#[allow(clippy::question_mark)]
fn read_line(
    reader: &mut dyn Read,
    maximum: usize,
    section_bytes: &mut usize,
    section_limit: usize,
    deadline: Option<Instant>,
) -> Result<Vec<u8>, HttpError> {
    let mut line = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let read = read_once(reader, &mut byte, "read response", deadline)?;
        if read == 0 {
            return Err(HttpError::IncompleteResponse);
        }
        if byte[0] == b'\r' {
            let mut line_feed = [0_u8; 1];
            let result = read_once(
                reader,
                &mut line_feed,
                "read response line ending",
                deadline,
            );
            let read = match result {
                Ok(read) => read,
                Err(error) => return Err(error),
            };
            if read == 0 {
                return Err(HttpError::IncompleteResponse);
            }
            if line_feed[0] != b'\n' {
                return Err(HttpError::MalformedHeader);
            }
            *section_bytes = section_bytes.saturating_add(line.len() + 2);
            if *section_bytes > section_limit {
                return Err(if section_limit == MAX_TRAILER_SECTION_BYTES {
                    HttpError::TrailerSectionLimitExceeded
                } else {
                    HttpError::HeaderSectionLimitExceeded
                });
            }
            return Ok(line);
        }
        if byte[0] == b'\n' {
            return Err(HttpError::MalformedHeader);
        }
        line.push(byte[0]);
        if line.len() > maximum {
            return Err(HttpError::HeaderLineLimitExceeded);
        }
    }
}

fn parse_status_line(line: &[u8]) -> Result<(u16, Vec<u8>), HttpError> {
    if line.len() < 13 || &line[..9] != b"HTTP/1.1 " || line[12] != b' ' {
        return Err(HttpError::MalformedStatusLine);
    }
    if !line[9..12].iter().all(u8::is_ascii_digit) {
        return Err(HttpError::MalformedStatusLine);
    }
    let status_code = u16::from(line[9] - b'0') * 100
        + u16::from(line[10] - b'0') * 10
        + u16::from(line[11] - b'0');
    if !(100..=599).contains(&status_code)
        || !line[13..].iter().all(|byte| (0x20..=0x7e).contains(byte))
    {
        return Err(HttpError::MalformedStatusLine);
    }
    Ok((status_code, line[13..].to_vec()))
}

fn parse_header_line(line: &[u8]) -> Result<RawHeader, HttpError> {
    if line
        .first()
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        return Err(HttpError::MalformedHeader);
    }
    let Some(colon) = line.iter().position(|byte| *byte == b':') else {
        return Err(HttpError::MalformedHeader);
    };
    if colon == 0 || colon > MAX_HEADER_NAME_BYTES || !line[..colon].iter().all(is_token) {
        return Err(HttpError::MalformedHeader);
    }
    let raw_value = &line[colon + 1..];
    let value = trim_ows(raw_value);
    if value.len() > MAX_HEADER_VALUE_BYTES
        || !value
            .iter()
            .all(|byte| *byte == b'\t' || (0x20..=0x7e).contains(byte))
    {
        return Err(HttpError::MalformedHeader);
    }
    let name = line[..colon]
        .iter()
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect::<String>();
    let value = String::from_utf8_lossy(value).into_owned();
    Ok(RawHeader { name, value })
}

fn trim_ows(value: &[u8]) -> &[u8] {
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

const fn is_token(byte: &u8) -> bool {
    matches!(
        *byte,
        b'0'..=b'9'
            | b'a'..=b'z'
            | b'A'..=b'Z'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn content_length(headers: &[RawHeader]) -> Result<Option<usize>, HttpError> {
    let mut value = None;
    for header in headers
        .iter()
        .filter(|header| header.name == "content-length")
    {
        if value.is_some() {
            return Err(HttpError::DuplicateContentLength);
        }
        let bytes = header.value.as_bytes();
        if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
            return Err(HttpError::InvalidContentLength);
        }
        let mut parsed = 0_usize;
        for byte in bytes {
            parsed = parsed
                .checked_mul(10)
                .and_then(|current| current.checked_add(usize::from(byte - b'0')))
                .ok_or(HttpError::InvalidContentLength)?;
        }
        value = Some(parsed);
    }
    Ok(value)
}

fn transfer_encoding(headers: &[RawHeader]) -> Result<bool, HttpError> {
    let mut values = headers
        .iter()
        .filter(|header| header.name == "transfer-encoding")
        .map(|header| header.value.as_str());
    let Some(value) = values.next() else {
        return Ok(false);
    };
    if values.next().is_some() {
        return Err(HttpError::UnsupportedTransferCoding);
    }
    if value.eq_ignore_ascii_case("chunked") {
        Ok(true)
    } else {
        Err(HttpError::UnsupportedTransferCoding)
    }
}

fn validate_content_encoding(headers: &[RawHeader]) -> Result<(), HttpError> {
    for header in headers
        .iter()
        .filter(|header| header.name == "content-encoding")
    {
        if !header.value.eq_ignore_ascii_case("identity") {
            return Err(HttpError::UnsupportedContentCoding);
        }
    }
    Ok(())
}

fn read_fixed_body(
    reader: &mut dyn Read,
    length: usize,
    maximum: usize,
    deadline: Option<Instant>,
) -> Result<Vec<u8>, HttpError> {
    if length > maximum {
        return Err(HttpError::BodyLimitExceeded);
    }
    let mut body = vec![0_u8; length];
    read_exact(reader, &mut body, deadline)?;
    Ok(body)
}

fn read_close_delimited_body(
    reader: &mut dyn Read,
    maximum: usize,
    deadline: Option<Instant>,
) -> Result<Vec<u8>, HttpError> {
    let mut body = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        let read = read_once(reader, &mut buffer, "read close-delimited body", deadline)?;
        if read == 0 {
            return Ok(body);
        }
        if body.len().saturating_add(read) > maximum {
            return Err(HttpError::BodyLimitExceeded);
        }
        body.extend_from_slice(&buffer[..read]);
    }
}

fn read_chunked_body(
    reader: &mut dyn Read,
    maximum: usize,
    deadline: Option<Instant>,
) -> Result<Vec<u8>, HttpError> {
    let mut body = Vec::new();
    let mut chunks = 0_usize;
    let mut chunk_line_bytes = 0_usize;
    loop {
        chunks += 1;
        if chunks > MAX_CHUNK_COUNT {
            return Err(HttpError::ChunkLimitExceeded);
        }
        let line = read_line(reader, 128, &mut chunk_line_bytes, usize::MAX, deadline)?;
        if line.contains(&b';') {
            return Err(HttpError::MalformedChunk);
        }
        let size_text = line.as_slice();
        if size_text.is_empty() || !size_text.iter().all(u8::is_ascii_hexdigit) {
            return Err(HttpError::MalformedChunk);
        }
        let size = usize::from_str_radix(&String::from_utf8_lossy(size_text), 16)
            .map_err(|_| HttpError::BodyLimitExceeded)?;
        if size == 0 {
            read_trailers(reader, deadline)?;
            return Ok(body);
        }
        if size > maximum || body.len().saturating_add(size) > maximum {
            return Err(HttpError::BodyLimitExceeded);
        }
        let old_len = body.len();
        body.resize(old_len + size, 0);
        read_exact(reader, &mut body[old_len..], deadline)?;
        let mut ending = [0_u8; 2];
        read_exact(reader, &mut ending, deadline)?;
        if ending != *b"\r\n" {
            return Err(HttpError::MalformedChunk);
        }
    }
}

fn read_trailers(reader: &mut dyn Read, deadline: Option<Instant>) -> Result<(), HttpError> {
    let mut fields = 0_usize;
    let mut trailer_bytes = 0_usize;
    loop {
        let line = read_line(
            reader,
            MAX_HEADER_VALUE_BYTES + MAX_HEADER_NAME_BYTES + 4,
            &mut trailer_bytes,
            MAX_TRAILER_SECTION_BYTES,
            deadline,
        )?;
        if line.is_empty() {
            return Ok(());
        }
        fields += 1;
        if fields > MAX_TRAILER_FIELDS {
            return Err(HttpError::TrailerLimitExceeded);
        }
        parse_header_line(&line)?;
    }
}

#[allow(clippy::question_mark)]
fn read_exact(
    reader: &mut dyn Read,
    buffer: &mut [u8],
    deadline: Option<Instant>,
) -> Result<(), HttpError> {
    let mut offset = 0_usize;
    while offset < buffer.len() {
        let result = read_once(
            reader,
            &mut buffer[offset..],
            "read response body",
            deadline,
        );
        let read = match result {
            Ok(read) => read,
            Err(error) => return Err(error),
        };
        if read == 0 {
            return Err(HttpError::IncompleteResponse);
        }
        offset += read;
    }
    Ok(())
}

fn read_once(
    reader: &mut dyn Read,
    buffer: &mut [u8],
    operation: &'static str,
    deadline: Option<Instant>,
) -> Result<usize, HttpError> {
    if deadline.is_some_and(|limit| Instant::now() >= limit) {
        return Err(HttpError::ExchangeTimedOut);
    }
    io_result(reader.read(buffer), operation)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used)]
mod tests {
    use std::error::Error;
    use std::io::{self, Cursor};
    use std::time::Duration;

    use super::*;

    fn policy() -> HttpExchangePolicy {
        HttpExchangePolicy::new(Duration::from_secs(1), 32).expect("test policy")
    }

    fn parse(input: &[u8], method: HttpMethod) -> Result<HttpResponse, HttpError> {
        parse_response_until(&mut Cursor::new(input), method, &policy(), None)
    }

    #[test]
    fn parses_fixed_length_response_and_retains_safe_headers() {
        let response = parse(
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Type: text/plain\r\nContent-Encoding: identity\r\nSet-Cookie: secret\r\n\r\nhello",
            HttpMethod::Get,
        )
        .expect("valid fixed response");
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.reason_phrase(), b"OK");
        assert_eq!(response.body(), b"hello");
        assert!(response.is_complete());
        assert_eq!(
            response
                .headers()
                .iter()
                .map(|header| header.name())
                .collect::<Vec<_>>(),
            vec!["content-length", "content-type", "content-encoding"]
        );
    }

    #[test]
    fn applies_no_body_semantics_for_head_and_statuses() {
        for (method, status) in [
            (HttpMethod::Head, 200),
            (HttpMethod::Get, 101),
            (HttpMethod::Get, 204),
            (HttpMethod::Get, 304),
        ] {
            let input = format!("HTTP/1.1 {status} No Content\r\nContent-Length: 5\r\n\r\nhello");
            let response = parse(input.as_bytes(), method).expect("no-body response");
            assert!(response.body().is_empty());
        }
    }

    #[test]
    fn parses_chunked_body_and_bounded_trailers() {
        let response = parse(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\nX-Trace: bounded\r\n\r\n",
            HttpMethod::Get,
        )
        .expect("valid chunked response");
        assert_eq!(response.body(), b"Wikipedia");
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nBad Trailer\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::MalformedHeader)
        ));
    }

    #[test]
    fn parses_close_delimited_body() {
        let response = parse(b"HTTP/1.1 200 OK\r\n\r\nclose", HttpMethod::Get)
            .expect("close-delimited response");
        assert_eq!(response.body(), b"close");
    }

    #[test]
    fn rejects_malformed_status_and_header_lines() {
        assert!(matches!(
            parse(b"", HttpMethod::Get),
            Err(HttpError::IncompleteResponse)
        ));
        assert!(matches!(
            parse(b"HTTP/1.1 200 OK\r", HttpMethod::Get),
            Err(HttpError::IncompleteResponse)
        ));
        assert!(matches!(
            parse(b"HTTP/1.1 200 OK\r\nX: y\r", HttpMethod::Get),
            Err(HttpError::IncompleteResponse)
        ));
        for input in [
            &b"HTTP/1.0 200 OK\r\n\r\n"[..],
            &b"HTTP/1.1 99 No\r\n\r\n"[..],
            &b"HTTP/1.1 A00 OK\r\n\r\n"[..],
            &b"HTTP/1.1 600 OK\r\n\r\n"[..],
            &b"HTTP/1.1 200 \x7f\r\n\r\n"[..],
            &b"HTTP/1.1 200\r\n\r\n"[..],
            &b"HTTP/1.1 200 OK\n\n"[..],
            &b"HTTP/1.1 200 OK\rX: y\r\n\r\n"[..],
            &b"HTTP/1.1 200 OK\r\n X: y\r\n\r\n"[..],
            &b"HTTP/1.1 200 OK\r\nX y\r\n\r\n"[..],
            &b"HTTP/1.1 200 OK\r\nBad Name: y\r\n\r\n"[..],
            &b"HTTP/1.1 200 OK\r\nX: \x01\r\n\r\n"[..],
        ] {
            assert!(matches!(
                parse(input, HttpMethod::Get),
                Err(HttpError::MalformedStatusLine | HttpError::MalformedHeader)
            ));
        }
    }

    #[test]
    fn rejects_ambiguous_and_unsupported_framing() {
        let cases: &[(&[u8], &str)] = &[
            (
                &b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\na"[..],
                "HTTP response contains duplicate content length",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nContent-Length: one\r\n\r\n"[..],
                "HTTP response content length is invalid",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nContent-Length:\r\n\r\n"[..],
                "HTTP response content length is invalid",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nContent-Length: 999999999999999999999999999999999999999999\r\n\r\n"[..],
                "HTTP response content length is invalid",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
                "HTTP response framing is ambiguous",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n"[..],
                "HTTP transfer coding is unsupported",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
                "HTTP transfer coding is unsupported",
            ),
            (
                &b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\n\r\n"[..],
                "HTTP content coding is unsupported",
            ),
        ];
        for (input, expected) in cases {
            let error = parse(input, HttpMethod::Get).expect_err("invalid framing");
            assert_eq!(error.to_string(), *expected);
        }
    }

    #[test]
    fn rejects_incomplete_and_over_budget_bodies() {
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nno",
                HttpMethod::Get
            ),
            Err(HttpError::IncompleteResponse)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nContent-Length: 33\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::BodyLimitExceeded)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\n\r\n123456789012345678901234567890123",
                HttpMethod::Get
            ),
            Err(HttpError::BodyLimitExceeded)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3;bad\r\nabc\r\n0\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::MalformedChunk)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabcX0\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::MalformedChunk)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::MalformedChunk)
        ));
        assert!(matches!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n21\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::BodyLimitExceeded)
        ));
        let mut overflowing_chunk =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        overflowing_chunk.extend(std::iter::repeat_n(b'f', 128));
        overflowing_chunk.extend_from_slice(b"\r\n");
        assert!(matches!(
            parse(&overflowing_chunk, HttpMethod::Get),
            Err(HttpError::BodyLimitExceeded)
        ));
    }

    #[test]
    fn rejects_redirects_and_expired_deadlines_without_following() {
        assert!(matches!(
            parse(
                b"HTTP/1.1 302 Found\r\nLocation: https://other.test/\r\n\r\n",
                HttpMethod::Get
            ),
            Err(HttpError::RedirectNotSupported)
        ));
        let mut input = Cursor::new(b"HTTP/1.1 200 OK\r\n\r\n".to_vec());
        assert!(matches!(
            parse_response_until(
                &mut input,
                HttpMethod::Get,
                &policy(),
                Some(std::time::Instant::now() - Duration::from_secs(1)),
            ),
            Err(HttpError::ExchangeTimedOut)
        ));
    }

    #[test]
    fn rejects_header_section_and_count_budgets() {
        let mut many = b"HTTP/1.1 200 OK\r\n".to_vec();
        for _ in 0..=MAX_HEADER_FIELDS {
            many.extend_from_slice(b"X-Test: y\r\n");
        }
        many.extend_from_slice(b"\r\n");
        assert!(matches!(
            parse(&many, HttpMethod::Get),
            Err(HttpError::HeaderFieldLimitExceeded)
        ));

        let mut large = b"HTTP/1.1 200 OK\r\nX: ".to_vec();
        large.extend(std::iter::repeat_n(
            b'a',
            MAX_HEADER_VALUE_BYTES + MAX_HEADER_NAME_BYTES + 5,
        ));
        large.extend_from_slice(b"\r\n\r\n");
        assert!(matches!(
            parse(&large, HttpMethod::Get),
            Err(HttpError::HeaderLineLimitExceeded)
        ));

        let mut section = b"HTTP/1.1 200 OK\r\n".to_vec();
        for _ in 0..4 {
            section.extend_from_slice(b"X: ");
            section.extend(std::iter::repeat_n(b'a', MAX_HEADER_VALUE_BYTES));
            section.extend_from_slice(b"\r\n");
        }
        section.extend_from_slice(b"\r\n");
        assert!(matches!(
            parse(&section, HttpMethod::Get),
            Err(HttpError::HeaderSectionLimitExceeded)
        ));
    }

    #[test]
    fn rejects_chunk_and_trailer_budgets() {
        let mut chunks = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        for _ in 0..=MAX_CHUNK_COUNT {
            chunks.extend_from_slice(b"1\r\na\r\n");
        }
        chunks.extend_from_slice(b"0\r\n\r\n");
        let wide_policy = HttpExchangePolicy::new(Duration::from_secs(1), MAX_HTTP_BODY_BYTES)
            .expect("wide chunk test policy");
        let chunk_error = parse_response_until(
            &mut Cursor::new(chunks),
            HttpMethod::Get,
            &wide_policy,
            None,
        )
        .expect_err("chunk count limit");
        assert!(matches!(chunk_error, HttpError::ChunkLimitExceeded));

        let mut trailers =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX: ".to_vec();
        trailers.extend(std::iter::repeat_n(b'a', MAX_TRAILER_SECTION_BYTES));
        trailers.extend_from_slice(b"\r\n\r\n");
        assert!(matches!(
            parse(&trailers, HttpMethod::Get),
            Err(HttpError::TrailerSectionLimitExceeded | HttpError::HeaderLineLimitExceeded)
        ));

        let mut trailer_count =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n".to_vec();
        for _ in 0..=MAX_TRAILER_FIELDS {
            trailer_count.extend_from_slice(b"X: y\r\n");
        }
        trailer_count.extend_from_slice(b"\r\n");
        assert!(matches!(
            parse(&trailer_count, HttpMethod::Get),
            Err(HttpError::TrailerLimitExceeded)
        ));
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("test I/O failure"))
        }
    }

    #[test]
    fn preserves_non_timeout_io_sources() {
        let error = parse_response_until(&mut FailingReader, HttpMethod::Get, &policy(), None)
            .expect_err("I/O failure");
        assert!(matches!(error, HttpError::Io { .. }));
        assert!(error.source().is_some());
    }

    struct FailingAfterReader {
        input: Cursor<Vec<u8>>,
    }

    impl Read for FailingAfterReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let read = self.input.read(buffer)?;
            if read == 0 {
                Err(io::Error::other("test I/O failure after bytes"))
            } else {
                Ok(read)
            }
        }
    }

    #[test]
    fn preserves_errors_after_a_partial_response() {
        for input in [
            b"HTTP/1.1 200 OK\r".to_vec(),
            b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec(),
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nab".to_vec(),
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabcX".to_vec(),
        ] {
            let error = parse_response_until(
                &mut FailingAfterReader {
                    input: Cursor::new(input),
                },
                HttpMethod::Get,
                &policy(),
                None,
            )
            .expect_err("partial response I/O failure");
            assert!(matches!(error, HttpError::Io { .. }));
        }
    }
}
