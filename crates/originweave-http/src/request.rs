use std::collections::BTreeSet;

use crate::{HttpClientPolicy, HttpError, HttpRequestTarget, RequestField};

/// The read-only HTTP request methods supported by the first HTTP slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Retrieve the selected representation content and metadata.
    Get,
    /// Retrieve only the response metadata for the selected representation.
    Head,
}

impl HttpMethod {
    /// Return the uppercase HTTP method token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
        }
    }

    /// Return whether response semantics suppress message content.
    #[must_use]
    pub const fn suppresses_response_content(self) -> bool {
        matches!(self, Self::Head)
    }
}

pub(crate) fn serialize_request(
    method: HttpMethod,
    target: &HttpRequestTarget,
    fields: &[RequestField],
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    if fields.len() > policy.max_header_field_count() {
        return Err(HttpError::ExcessiveRequestFieldCount {
            field_count: fields.len(),
            maximum_count: policy.max_header_field_count(),
        });
    }
    let mut seen = BTreeSet::new();
    for field in fields {
        if field.name().len() > policy.max_header_name_bytes() {
            return Err(HttpError::RequestFieldNameTooLarge {
                byte_count: field.name().len(),
                maximum_bytes: policy.max_header_name_bytes(),
            });
        }
        if field.value_byte_count() > policy.max_header_value_bytes() {
            return Err(HttpError::RequestFieldValueTooLarge {
                byte_count: field.value_byte_count(),
                maximum_bytes: policy.max_header_value_bytes(),
            });
        }
        if !seen.insert(field.name()) {
            return Err(HttpError::DuplicateRequestField {
                field_name: field.name().to_owned(),
            });
        }
    }

    let canonical_origin = target.origin().as_str();
    let authority = &canonical_origin[target.origin().scheme().len() + 3..];
    let fixed_request_fields = b"\r\nConnection: close\r\nAccept-Encoding: gzip, deflate\r\n";
    let mut byte_count = method
        .as_str()
        .len()
        .saturating_add(1)
        .saturating_add(target.path_and_query().len())
        .saturating_add(b" HTTP/1.1\r\nHost: ".len())
        .saturating_add(authority.len())
        .saturating_add(fixed_request_fields.len());
    for field in fields {
        byte_count = byte_count
            .saturating_add(field.name().len())
            .saturating_add(b": ".len())
            .saturating_add(field.value_byte_count())
            .saturating_add(b"\r\n".len());
    }
    byte_count = byte_count.saturating_add(b"\r\n".len());
    let maximum = policy.max_request_bytes();
    if byte_count > maximum {
        return Err(HttpError::RequestTooLarge {
            byte_count,
            maximum_bytes: maximum,
        });
    }

    let mut output = Vec::with_capacity(byte_count);
    output.extend_from_slice(method.as_str().as_bytes());
    output.extend_from_slice(b" ");
    output.extend_from_slice(target.path_and_query().as_bytes());
    output.extend_from_slice(b" HTTP/1.1\r\nHost: ");
    output.extend_from_slice(authority.as_bytes());
    output.extend_from_slice(fixed_request_fields);
    for field in fields {
        output.extend_from_slice(field.name().as_bytes());
        output.extend_from_slice(b": ");
        output.extend_from_slice(field.value());
        output.extend_from_slice(b"\r\n");
    }
    output.extend_from_slice(b"\r\n");
    Ok(output)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use originweave_core::Origin;

    use super::*;

    fn constrained_policy(
        max_request_bytes: usize,
        max_header_field_count: usize,
        max_header_name_bytes: usize,
        max_header_value_bytes: usize,
    ) -> HttpClientPolicy {
        let defaults = HttpClientPolicy::strict_defaults();
        HttpClientPolicy::new(
            Duration::from_secs(1),
            max_request_bytes,
            defaults.max_status_line_bytes(),
            max_header_field_count,
            max_header_name_bytes,
            max_header_value_bytes,
            defaults.max_header_section_bytes(),
            defaults.max_interim_response_count(),
            defaults.max_chunk_count(),
            defaults.max_trailer_field_count(),
            defaults.max_trailer_section_bytes(),
            defaults.max_encoded_content_bytes(),
            defaults.max_decoded_content_bytes(),
            defaults.max_content_expansion_ratio(),
            crate::AlpnHttp11Policy::RequireHttp11,
            crate::IntegrityRequirement::Optional,
        )
        .expect("constrained request policy")
    }

    #[test]
    fn methods_expose_exact_tokens_and_content_semantics() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
        assert!(!HttpMethod::Get.suppresses_response_content());
        assert!(HttpMethod::Head.suppresses_response_content());
    }

    #[test]
    fn request_serialization_is_deterministic_and_authority_bound() {
        let target = HttpRequestTarget::parse(
            Origin::parse("https://example.com:8443").expect("origin"),
            "/items?q=one",
        )
        .expect("target");
        let fields = [RequestField::new("Accept", b"application/json").expect("field")];
        let request = serialize_request(
            HttpMethod::Get,
            &target,
            &fields,
            &HttpClientPolicy::strict_defaults(),
        )
        .expect("request");
        assert_eq!(
            request,
            b"GET /items?q=one HTTP/1.1\r\nHost: example.com:8443\r\nConnection: close\r\nAccept-Encoding: gzip, deflate\r\naccept: application/json\r\n\r\n"
        );
    }

    #[test]
    fn request_serialization_rejects_every_narrower_per_exchange_limit() {
        let target =
            HttpRequestTarget::parse(Origin::parse("https://example.com").expect("origin"), "/")
                .expect("target");
        let first = RequestField::new("x-a", b"a").expect("first field");
        let second = RequestField::new("x-b", b"b").expect("second field");
        assert!(matches!(
            serialize_request(
                HttpMethod::Get,
                &target,
                &[first.clone(), second],
                &constrained_policy(16_384, 1, 256, 8_192),
            ),
            Err(HttpError::ExcessiveRequestFieldCount {
                field_count: 2,
                maximum_count: 1,
            })
        ));

        let long_name = RequestField::new("xx", b"a").expect("two-byte name");
        assert!(matches!(
            serialize_request(
                HttpMethod::Get,
                &target,
                &[long_name],
                &constrained_policy(16_384, 128, 1, 8_192),
            ),
            Err(HttpError::RequestFieldNameTooLarge {
                byte_count: 2,
                maximum_bytes: 1,
            })
        ));

        let long_value = RequestField::new("x", b"ab").expect("two-byte value");
        assert!(matches!(
            serialize_request(
                HttpMethod::Get,
                &target,
                &[long_value],
                &constrained_policy(16_384, 128, 256, 1),
            ),
            Err(HttpError::RequestFieldValueTooLarge {
                byte_count: 2,
                maximum_bytes: 1,
            })
        ));

        let duplicate_error = serialize_request(
            HttpMethod::Get,
            &target,
            &[first.clone(), first],
            &constrained_policy(16_384, 128, 256, 8_192),
        )
        .expect_err("duplicate request field");
        assert_eq!(
            format!("{duplicate_error:?}"),
            "DuplicateRequestField { field_name: \"x-a\" }"
        );

        assert!(matches!(
            serialize_request(
                HttpMethod::Get,
                &target,
                &[],
                &constrained_policy(1, 128, 256, 8_192),
            ),
            Err(HttpError::RequestTooLarge {
                byte_count: 88,
                maximum_bytes: 1,
            })
        ));
    }
}
