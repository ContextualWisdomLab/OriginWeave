use crate::field::{is_field_value_byte, is_token_byte};
use crate::{HttpClientPolicy, HttpError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseField {
    pub(crate) name: String,
    pub(crate) value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseHead {
    pub(crate) status_code: u16,
    pub(crate) fields: Vec<ResponseField>,
}

pub(crate) fn parse_response_head(
    bytes: &[u8],
    policy: &HttpClientPolicy,
) -> Result<ResponseHead, HttpError> {
    if bytes.len() > policy.max_header_section_bytes() {
        return Err(HttpError::HeaderSectionTooLarge {
            byte_count: bytes.len(),
            maximum_bytes: policy.max_header_section_bytes(),
        });
    }
    if !bytes.ends_with(b"\r\n\r\n") || !has_only_crlf_line_breaks(bytes) {
        return Err(HttpError::InvalidResponseField);
    }
    let content = bytes
        .get(..bytes.len().saturating_sub(4))
        .ok_or(HttpError::InvalidResponseField)?;
    let mut lines = content.split(|byte| *byte == b'\n');
    let status_line = lines.next().ok_or(HttpError::InvalidStatusLine)?;
    let status_line = strip_terminal_cr(status_line).ok_or(HttpError::InvalidStatusLine)?;
    let status_code = parse_status_line(status_line, policy)?;

    let mut fields = Vec::new();
    for raw_line in lines {
        let line = strip_terminal_cr(raw_line).ok_or(HttpError::InvalidResponseField)?;
        if line.is_empty() || matches!(line.first(), Some(b' ' | b'\t')) {
            return Err(HttpError::InvalidResponseField);
        }
        if fields.len() >= policy.max_header_field_count() {
            return Err(HttpError::TooManyResponseFields {
                field_count: fields.len() + 1,
                maximum_count: policy.max_header_field_count(),
            });
        }
        let colon = line
            .iter()
            .position(|byte| *byte == b':')
            .ok_or(HttpError::InvalidResponseField)?;
        let name = line.get(..colon).ok_or(HttpError::InvalidResponseField)?;
        let raw_value = line
            .get(colon + 1..)
            .ok_or(HttpError::InvalidResponseField)?;
        if name.is_empty() || !name.iter().copied().all(is_token_byte) {
            return Err(HttpError::InvalidResponseField);
        }
        if name.len() > policy.max_header_name_bytes() {
            return Err(HttpError::ResponseFieldNameTooLarge {
                byte_count: name.len(),
                maximum_bytes: policy.max_header_name_bytes(),
            });
        }
        let value = trim_ows(raw_value);
        if value.len() > policy.max_header_value_bytes() {
            return Err(HttpError::ResponseFieldValueTooLarge {
                byte_count: value.len(),
                maximum_bytes: policy.max_header_value_bytes(),
            });
        }
        if !value.iter().copied().all(is_field_value_byte) {
            return Err(HttpError::InvalidResponseField);
        }
        let normalized = std::str::from_utf8(name)
            .map_err(|_error| HttpError::InvalidResponseField)?
            .to_ascii_lowercase();
        fields.push(ResponseField {
            name: normalized,
            value: value.to_vec(),
        });
    }
    Ok(ResponseHead {
        status_code,
        fields,
    })
}

fn parse_status_line(line: &[u8], policy: &HttpClientPolicy) -> Result<u16, HttpError> {
    if line.len() > policy.max_status_line_bytes() {
        return Err(HttpError::StatusLineTooLarge {
            byte_count: line.len(),
            maximum_bytes: policy.max_status_line_bytes(),
        });
    }
    let digits = line
        .strip_prefix(b"HTTP/1.1 ")
        .and_then(|remaining| remaining.get(..3))
        .ok_or(HttpError::InvalidStatusLine)?;
    if digits.len() != 3 || !digits.iter().all(u8::is_ascii_digit) {
        return Err(HttpError::InvalidStatusLine);
    }
    let suffix = line.get(12..).ok_or(HttpError::InvalidStatusLine)?;
    if !suffix.is_empty() {
        let reason = suffix
            .strip_prefix(b" ")
            .ok_or(HttpError::InvalidStatusLine)?;
        if !reason.iter().copied().all(is_field_value_byte) {
            return Err(HttpError::InvalidStatusLine);
        }
    }
    let code = u16::from(digits[0] - b'0') * 100
        + u16::from(digits[1] - b'0') * 10
        + u16::from(digits[2] - b'0');
    if !(100..=599).contains(&code) {
        return Err(HttpError::InvalidStatusLine);
    }
    Ok(code)
}

fn has_only_crlf_line_breaks(bytes: &[u8]) -> bool {
    let mut index = 0_usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                if bytes.get(index + 1) != Some(&b'\n') {
                    return false;
                }
                index += 2;
            }
            b'\n' => return false,
            _ => index += 1,
        }
    }
    true
}

fn strip_terminal_cr(line: &[u8]) -> Option<&[u8]> {
    line.strip_suffix(b"\r")
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
    value.get(start..end).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use super::*;
    use crate::AlpnHttp11Policy;

    fn policy() -> HttpClientPolicy {
        HttpClientPolicy::new(
            Duration::from_secs(1),
            500,
            32,
            3,
            16,
            16,
            128,
            100,
            AlpnHttp11Policy::RequireHttp11,
        )
        .expect("policy")
    }

    #[test]
    fn response_head_parser_accepts_strict_http11_and_trims_ows() {
        let head = parse_response_head(
            b"HTTP/1.1 200 OK\r\nContent-Length:  3 \t\r\nX-Test:\tvalue\r\n\r\n",
            &policy(),
        )
        .expect("response head");
        assert_eq!(head.status_code, 200);
        assert_eq!(head.fields[0].name, "content-length");
        assert_eq!(head.fields[0].value, b"3");
        assert_eq!(head.fields[1].value, b"value");
    }

    #[test]
    fn response_head_parser_rejects_line_and_field_ambiguity() {
        for bytes in [
            b"HTTP/1.1 200 OK\nX: y\n\n".as_slice(),
            b"HTTP/1.1 200 OK\rX: y\r\n\r\n".as_slice(),
            b"HTTP/1.1 200 OK\r\n folded: x\r\n\r\n".as_slice(),
            b"HTTP/1.1 200 OK\r\nBad Name: x\r\n\r\n".as_slice(),
            b"HTTP/1.1 200 OK\r\nX: bad\x7f\r\n\r\n".as_slice(),
        ] {
            assert!(parse_response_head(bytes, &policy()).is_err(), "{bytes:?}");
        }
    }

    #[test]
    fn status_line_validation_is_strict_and_bounded() {
        for bytes in [
            b"HTTP/1.0 200 OK\r\n\r\n".as_slice(),
            b"HTTP/1.1 099 Low\r\n\r\n".as_slice(),
            b"HTTP/1.1 600 High\r\n\r\n".as_slice(),
            b"HTTP/1.1 abc Nope\r\n\r\n".as_slice(),
            b"HTTP/1.1 200X\r\n\r\n".as_slice(),
        ] {
            assert!(parse_response_head(bytes, &policy()).is_err(), "{bytes:?}");
        }
        let long_reason = format!("HTTP/1.1 200 {}\r\n\r\n", "x".repeat(30));
        assert!(matches!(
            parse_response_head(long_reason.as_bytes(), &policy()),
            Err(HttpError::StatusLineTooLarge { .. })
        ));
    }

    #[test]
    fn response_head_budgets_are_enforced() {
        let many = b"HTTP/1.1 200 OK\r\nA: 1\r\nB: 2\r\nC: 3\r\nD: 4\r\n\r\n";
        assert!(matches!(
            parse_response_head(many, &policy()),
            Err(HttpError::TooManyResponseFields { .. })
        ));
        let long_name = b"HTTP/1.1 200 OK\r\nabcdefghijklmnopq: 1\r\n\r\n";
        assert!(matches!(
            parse_response_head(long_name, &policy()),
            Err(HttpError::ResponseFieldNameTooLarge { .. })
        ));
        let long_value = b"HTTP/1.1 200 OK\r\nX: 12345678901234567\r\n\r\n";
        assert!(matches!(
            parse_response_head(long_value, &policy()),
            Err(HttpError::ResponseFieldValueTooLarge { .. })
        ));
        assert!(matches!(
            parse_response_head(&vec![b'x'; 129], &policy()),
            Err(HttpError::HeaderSectionTooLarge { .. })
        ));
    }
}
