use originweave_core::Origin;
use sha2::{Digest, Sha256};

use crate::HttpError;

const MAX_REQUEST_TARGET_BYTES: usize = 8_192;
const MAX_PATH_PREFIX_BYTES: usize = 256;

/// One strict origin-form request target bound to a canonical OriginWeave origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequestTarget {
    origin: Origin,
    encoded_path_and_query: String,
    target_hash: String,
    query_present: bool,
    path_prefix: String,
}

impl HttpRequestTarget {
    /// Parse and encode one origin-form path and optional query.
    pub fn parse(origin: Origin, path_and_query: &str) -> Result<Self, HttpError> {
        let encoded_path_and_query = encode_target(path_and_query)?;
        if encoded_path_and_query.len() > MAX_REQUEST_TARGET_BYTES {
            return Err(HttpError::InvalidRequestTarget);
        }
        let query_present = encoded_path_and_query.contains('?');
        let path = encoded_path_and_query
            .split_once('?')
            .map_or(encoded_path_and_query.as_str(), |(path, _query)| path);
        let path_prefix = path
            .get(..std::cmp::min(path.len(), MAX_PATH_PREFIX_BYTES))
            .ok_or(HttpError::InvalidRequestTarget)?
            .to_owned();
        let target_hash = target_identifier(&origin, &encoded_path_and_query);
        Ok(Self {
            origin,
            encoded_path_and_query,
            target_hash,
            query_present,
            path_prefix,
        })
    }

    /// Return the canonical origin authorized for this target.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the encoded origin-form path and query used on the wire.
    #[must_use]
    pub const fn path_and_query(&self) -> &str {
        self.encoded_path_and_query.as_str()
    }

    /// Return the domain-separated SHA-256 target identifier.
    #[must_use]
    pub const fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Return whether the target contains a query without exposing query values separately.
    #[must_use]
    pub const fn query_present(&self) -> bool {
        self.query_present
    }

    /// Return a bounded human-readable path prefix that never contains query values.
    #[must_use]
    pub const fn path_prefix(&self) -> &str {
        self.path_prefix.as_str()
    }
}

fn encode_target(input: &str) -> Result<String, HttpError> {
    if !input.starts_with('/') || input.contains('#') {
        return Err(HttpError::InvalidRequestTarget);
    }
    let bytes = input.as_bytes();
    let mut encoded = String::with_capacity(bytes.len());
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            let high = bytes.get(index + 1).copied();
            let low = bytes.get(index + 2).copied();
            if !matches!((high, low), (Some(high), Some(low)) if high.is_ascii_hexdigit() && low.is_ascii_hexdigit()) {
                return Err(HttpError::InvalidRequestTarget);
            }
            encoded.push('%');
            encoded.push(char::from(high.unwrap_or(b'0')));
            encoded.push(char::from(low.unwrap_or(b'0')));
            index += 3;
        } else if byte.is_ascii() {
            if byte.is_ascii_control() || byte.is_ascii_whitespace() || byte == b'\\' {
                return Err(HttpError::InvalidRequestTarget);
            }
            encoded.push(char::from(byte));
            index += 1;
        } else {
            use std::fmt::Write as _;
            write!(&mut encoded, "%{byte:02X}").map_err(|_error| HttpError::InvalidRequestTarget)?;
            index += 1;
        }
        if encoded.len() > MAX_REQUEST_TARGET_BYTES {
            return Err(HttpError::InvalidRequestTarget);
        }
    }
    Ok(encoded)
}

fn target_identifier(origin: &Origin, target: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"originweave:http-target:v1\0");
    hasher.update(origin.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(target.as_bytes());
    let digest = hasher.finalize();
    let mut identifier = String::with_capacity(71);
    identifier.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        let _result = write!(&mut identifier, "{byte:02x}");
    }
    identifier
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn preserved_percent_escapes_and_encoded_limit_are_deterministic() {
        let origin = Origin::parse("https://example.com").expect("origin");
        let target = HttpRequestTarget::parse(origin, "/a%2fb%2F?x=%41").expect("target");
        assert_eq!(target.path_and_query(), "/a%2fb%2F?x=%41");

        let exact = format!("/{}", "a".repeat(MAX_REQUEST_TARGET_BYTES - 1));
        assert!(HttpRequestTarget::parse(
            Origin::parse("https://example.com").expect("origin"),
            &exact
        )
        .is_ok());
        let excessive = format!("/{}", "a".repeat(MAX_REQUEST_TARGET_BYTES));
        assert!(HttpRequestTarget::parse(
            Origin::parse("https://example.com").expect("origin"),
            &excessive
        )
        .is_err());
    }

    #[test]
    fn long_path_prefix_is_bounded_without_query_data() {
        let origin = Origin::parse("https://example.com").expect("origin");
        let path = format!("/{}?secret=value", "x".repeat(400));
        let target = HttpRequestTarget::parse(origin, &path).expect("target");
        assert_eq!(target.path_prefix().len(), MAX_PATH_PREFIX_BYTES);
        assert!(!target.path_prefix().contains("secret"));
    }
}
