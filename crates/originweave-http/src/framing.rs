use crate::response_head::ResponseField;
use crate::{BodyFraming, HttpClientPolicy, HttpError, HttpMethod};

pub(crate) fn determine_body_framing(
    method: HttpMethod,
    status_code: u16,
    fields: &[ResponseField],
    policy: &HttpClientPolicy,
) -> Result<BodyFraming, HttpError> {
    let transfer_encoding_present = fields
        .iter()
        .any(|field| field.name == "transfer-encoding");
    let content_lengths = fields
        .iter()
        .filter(|field| field.name == "content-length")
        .map(|field| parse_content_length(field.value.as_slice()))
        .collect::<Result<Vec<_>, _>>()?;

    if transfer_encoding_present {
        return Err(HttpError::UnsupportedTransferEncoding);
    }
    let content_length = consistent_content_length(&content_lengths)?;
    if method.suppresses_response_content()
        || (100..=199).contains(&status_code)
        || status_code == 204
        || status_code == 304
    {
        return Ok(BodyFraming::NoContent);
    }
    match content_length {
        Some(length) => {
            let byte_count = usize::try_from(length).map_err(|_error| HttpError::ContentTooLarge {
                byte_count: usize::MAX,
                maximum_bytes: policy.max_encoded_content_bytes(),
            })?;
            if byte_count > policy.max_encoded_content_bytes() {
                return Err(HttpError::ContentTooLarge {
                    byte_count,
                    maximum_bytes: policy.max_encoded_content_bytes(),
                });
            }
            Ok(BodyFraming::ContentLength(length))
        }
        None => Err(HttpError::UnsupportedBodyFraming),
    }
}

fn parse_content_length(value: &[u8]) -> Result<u64, HttpError> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(HttpError::InvalidContentLength);
    }
    let text = std::str::from_utf8(value).map_err(|_error| HttpError::InvalidContentLength)?;
    text.parse::<u64>()
        .map_err(|_error| HttpError::InvalidContentLength)
}

fn consistent_content_length(values: &[u64]) -> Result<Option<u64>, HttpError> {
    let Some(first) = values.first().copied() else {
        return Ok(None);
    };
    if values.iter().all(|value| *value == first) {
        Ok(Some(first))
    } else {
        Err(HttpError::InvalidContentLength)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use super::*;
    use crate::AlpnHttp11Policy;

    fn policy(max_content: usize) -> HttpClientPolicy {
        HttpClientPolicy::new(
            Duration::from_secs(1),
            500,
            100,
            10,
            100,
            100,
            500,
            max_content,
            AlpnHttp11Policy::RequireHttp11,
        )
        .expect("policy")
    }

    fn field(name: &str, value: &[u8]) -> ResponseField {
        ResponseField {
            name: name.to_owned(),
            value: value.to_vec(),
        }
    }

    #[test]
    fn no_content_semantics_override_a_valid_length() {
        let fields = [field("content-length", b"9")];
        for (method, status) in [
            (HttpMethod::Head, 200),
            (HttpMethod::Get, 100),
            (HttpMethod::Get, 199),
            (HttpMethod::Get, 204),
            (HttpMethod::Get, 304),
        ] {
            assert_eq!(
                determine_body_framing(method, status, &fields, &policy(10))
                    .expect("no-content semantics"),
                BodyFraming::NoContent
            );
        }
    }

    #[test]
    fn fixed_length_framing_accepts_identical_duplicates_only() {
        let fields = [
            field("content-length", b"3"),
            field("content-length", b"3"),
        ];
        assert_eq!(
            determine_body_framing(HttpMethod::Get, 200, &fields, &policy(3))
                .expect("fixed length"),
            BodyFraming::ContentLength(3)
        );
        let conflicting = [
            field("content-length", b"3"),
            field("content-length", b"4"),
        ];
        assert!(matches!(
            determine_body_framing(HttpMethod::Get, 200, &conflicting, &policy(10)),
            Err(HttpError::InvalidContentLength)
        ));
    }

    #[test]
    fn unsupported_or_excessive_framing_fails_closed() {
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Get,
                200,
                &[field("transfer-encoding", b"chunked")],
                &policy(10)
            ),
            Err(HttpError::UnsupportedTransferEncoding)
        ));
        assert!(matches!(
            determine_body_framing(HttpMethod::Get, 200, &[], &policy(10)),
            Err(HttpError::UnsupportedBodyFraming)
        ));
        assert!(matches!(
            determine_body_framing(
                HttpMethod::Get,
                200,
                &[field("content-length", b"11")],
                &policy(10)
            ),
            Err(HttpError::ContentTooLarge { .. })
        ));
        for invalid in [b"".as_slice(), b"+1", b"1, 1", b"18446744073709551616"] {
            assert!(parse_content_length(invalid).is_err(), "{invalid:?}");
        }
    }
}
