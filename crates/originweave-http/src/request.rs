use std::collections::BTreeSet;

use crate::{HttpClientPolicy, HttpError, HttpRequestTarget, RequestField};

/// HTTP request methods admitted by the first bounded exchange slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Retrieve a representation.
    Get,
    /// Retrieve response metadata while suppressing response content semantics.
    Head,
}

impl HttpMethod {
    /// Return the exact uppercase HTTP method token.
    #[must_use]
    pub const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Get => b"GET",
            Self::Head => b"HEAD",
        }
    }

    /// Return whether response semantics suppress message content.
    #[must_use]
    pub const fn suppresses_response_content(self) -> bool {
        match self {
            Self::Get => false,
            Self::Head => true,
        }
    }
}

pub(crate) fn serialize_request(
    method: HttpMethod,
    target: &HttpRequestTarget,
    fields: &[RequestField],
    policy: &HttpClientPolicy,
) -> Result<Vec<u8>, HttpError> {
    let authority = target
        .origin()
        .as_str()
        .split_once("://")
        .map(|(_scheme, authority)| authority)
        .ok_or(HttpError::InvalidRequestTarget)?;
    let mut names = BTreeSet::new();
    let mut request = Vec::new();
    request.extend_from_slice(method.as_bytes());
    request.extend_from_slice(b" ");
    request.extend_from_slice(target.path_and_query().as_bytes());
    request.extend_from_slice(b" HTTP/1.1\r\nHost: ");
    request.extend_from_slice(authority.as_bytes());
    request.extend_from_slice(b"\r\nConnection: close\r\nAccept-Encoding: identity\r\n");
    for field in fields {
        if !names.insert(field.name()) {
            return Err(HttpError::DuplicateRequestField);
        }
        request.extend_from_slice(field.name().as_bytes());
        request.extend_from_slice(b": ");
        request.extend_from_slice(field.value());
        request.extend_from_slice(b"\r\n");
        if request.len() > policy.max_request_bytes() {
            return Err(HttpError::RequestTooLarge {
                byte_count: request.len(),
                maximum_bytes: policy.max_request_bytes(),
            });
        }
    }
    request.extend_from_slice(b"\r\n");
    if request.len() > policy.max_request_bytes() {
        return Err(HttpError::RequestTooLarge {
            byte_count: request.len(),
            maximum_bytes: policy.max_request_bytes(),
        });
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::time::Duration;

    use originweave_core::Origin;

    use super::*;
    use crate::AlpnHttp11Policy;

    fn policy(max_request_bytes: usize) -> HttpClientPolicy {
        HttpClientPolicy::new(
            Duration::from_secs(1),
            max_request_bytes,
            100,
            10,
            20,
            100,
            500,
            500,
            AlpnHttp11Policy::RequireHttp11,
        )
        .expect("policy")
    }

    #[test]
    fn request_serialization_is_deterministic_and_host_bound() {
        let target = HttpRequestTarget::parse(
            Origin::parse("https://example.com:8443").expect("origin"),
            "/a?b=1",
        )
        .expect("target");
        let fields = [RequestField::new("accept-language", b"en").expect("field")];
        let request = serialize_request(HttpMethod::Get, &target, &fields, &policy(500))
            .expect("request");
        assert_eq!(
            request,
            b"GET /a?b=1 HTTP/1.1\r\nHost: example.com:8443\r\nConnection: close\r\nAccept-Encoding: identity\r\naccept-language: en\r\n\r\n"
        );
    }

    #[test]
    fn duplicate_and_oversized_requests_fail_closed() {
        let target = HttpRequestTarget::parse(
            Origin::parse("https://example.com").expect("origin"),
            "/",
        )
        .expect("target");
        let fields = [
            RequestField::new("x-test", b"one").expect("field"),
            RequestField::new("X-Test", b"two").expect("field"),
        ];
        assert!(matches!(
            serialize_request(HttpMethod::Get, &target, &fields, &policy(500)),
            Err(HttpError::DuplicateRequestField)
        ));
        assert!(matches!(
            serialize_request(HttpMethod::Head, &target, &[], &policy(1)),
            Err(HttpError::RequestTooLarge { .. })
        ));
    }
}
