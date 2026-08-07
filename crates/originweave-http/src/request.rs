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

    let authority = target
        .origin()
        .as_str()
        .strip_prefix("https://")
        .or_else(|| target.origin().as_str().strip_prefix("http://"))
        .ok_or(HttpError::InvalidRequestTarget)?;
    let maximum = policy.max_request_bytes();
    let mut output = Vec::with_capacity(maximum.min(1_024));
    append_bounded(&mut output, method.as_str().as_bytes(), maximum)?;
    append_bounded(&mut output, b" ", maximum)?;
    append_bounded(&mut output, target.path_and_query().as_bytes(), maximum)?;
    append_bounded(&mut output, b" HTTP/1.1\r\nHost: ", maximum)?;
    append_bounded(&mut output, authority.as_bytes(), maximum)?;
    append_bounded(
        &mut output,
        b"\r\nConnection: close\r\nAccept-Encoding: gzip, deflate\r\n",
        maximum,
    )?;
    for field in fields {
        append_bounded(&mut output, field.name().as_bytes(), maximum)?;
        append_bounded(&mut output, b": ", maximum)?;
        append_bounded(&mut output, field.value(), maximum)?;
        append_bounded(&mut output, b"\r\n", maximum)?;
    }
    append_bounded(&mut output, b"\r\n", maximum)?;
    Ok(output)
}

fn append_bounded(output: &mut Vec<u8>, bytes: &[u8], maximum: usize) -> Result<(), HttpError> {
    let next_length = output.len().checked_add(bytes.len()).unwrap_or(usize::MAX);
    if next_length > maximum {
        return Err(HttpError::RequestTooLarge {
            byte_count: next_length,
            maximum_bytes: maximum,
        });
    }
    output.extend_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use originweave_core::Origin;

    use super::*;

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
}
