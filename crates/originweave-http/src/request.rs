use originweave_core::Origin;

use crate::HttpRequestError;

/// The largest request-target accepted by the first HTTP/1.1 slice.
pub const MAX_REQUEST_TARGET_BYTES: usize = 8 * 1024;

/// A method admitted by the read-only HTTP authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// Retrieve a representation and its body.
    Get,
    /// Retrieve response metadata without a response body.
    Head,
}

impl HttpMethod {
    pub(crate) const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Get => b"GET",
            Self::Head => b"HEAD",
        }
    }
}

/// A validated HTTP origin-form request target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequestTarget(String);

impl HttpRequestTarget {
    /// Validate a path and optional query without accepting an absolute URI.
    pub fn parse(input: &str) -> Result<Self, HttpRequestError> {
        let bytes = input.as_bytes();
        let valid = !bytes.is_empty()
            && bytes.len() <= MAX_REQUEST_TARGET_BYTES
            && bytes[0] == b'/'
            && !input.starts_with("//")
            && !input.contains('#')
            && bytes.iter().all(|byte| (0x21..=0x7e).contains(byte));
        if !valid {
            return Err(HttpRequestError::InvalidRequestTarget);
        }
        Ok(Self(input.to_owned()))
    }

    /// Return the canonical request-target bytes as UTF-8 text.
    #[must_use]
    #[inline(never)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A request whose method, origin, target, and framing fields are owned by this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: HttpMethod,
    origin: Origin,
    target: HttpRequestTarget,
    authority: String,
}

impl HttpRequest {
    /// Construct a read-only request for one canonical HTTPS origin.
    pub fn new(
        method: HttpMethod,
        origin: Origin,
        target: HttpRequestTarget,
    ) -> Result<Self, HttpRequestError> {
        let authority = match origin.as_str().strip_prefix("https://") {
            Some(authority) => authority.to_owned(),
            None => return Err(HttpRequestError::InsecureOrigin),
        };
        Ok(Self {
            method,
            origin,
            target,
            authority,
        })
    }

    /// Return the explicit request method.
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Return the authenticated-origin contract required by this request.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the validated origin-form target.
    #[must_use]
    #[inline(never)]
    pub const fn target(&self) -> &HttpRequestTarget {
        &self.target
    }

    /// Serialize only the method, target, generated host, and close framing fields.
    pub fn serialize(&self) -> Vec<u8> {
        let mut request = Vec::with_capacity(
            self.method.as_bytes().len() + self.target.0.len() + self.authority.len() + 40,
        );
        request.extend_from_slice(self.method.as_bytes());
        request.extend_from_slice(b" ");
        request.extend_from_slice(self.target.0.as_bytes());
        request.extend_from_slice(b" HTTP/1.1\r\nHost: ");
        request.extend_from_slice(self.authority.as_bytes());
        request.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
        request
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn rejects_every_invalid_origin_form_target_shape() {
        for input in ["", "relative", "//absolute", "/fragment#bad", "/control\n"] {
            assert!(matches!(
                HttpRequestTarget::parse(input),
                Err(HttpRequestError::InvalidRequestTarget)
            ));
        }
        let oversized = format!("/{}", "a".repeat(MAX_REQUEST_TARGET_BYTES));
        assert!(matches!(
            HttpRequestTarget::parse(&oversized),
            Err(HttpRequestError::InvalidRequestTarget)
        ));
    }
}
