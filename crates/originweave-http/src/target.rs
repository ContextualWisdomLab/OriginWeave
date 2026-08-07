use originweave_core::Origin;
use sha2::{Digest, Sha256};

use crate::HttpError;

const MAX_REQUEST_TARGET_BYTES: usize = 8_192;
const MAX_EVIDENCE_PATH_PREFIX_BYTES: usize = 256;
const HEX_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// One canonical origin-bound HTTP origin-form request target.
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
        if !path_and_query.starts_with('/') {
            return Err(HttpError::InvalidRequestTarget);
        }
        let source = path_and_query.as_bytes();
        let mut encoded = String::with_capacity(source.len());
        let mut byte_index = 0_usize;
        while byte_index < source.len() {
            let byte = source[byte_index];
            if byte == b'%' {
                if byte_index + 2 >= source.len()
                    || !source[byte_index + 1].is_ascii_hexdigit()
                    || !source[byte_index + 2].is_ascii_hexdigit()
                {
                    return Err(HttpError::InvalidPercentEncoding { byte_index });
                }
                encoded.push('%');
                encoded.push(char::from(source[byte_index + 1]));
                encoded.push(char::from(source[byte_index + 2]));
                byte_index += 3;
                continue;
            }
            if byte == b'#'
                || byte == b'\\'
                || byte.is_ascii_control()
                || byte.is_ascii_whitespace()
            {
                return Err(HttpError::InvalidRequestTarget);
            }
            if byte.is_ascii() {
                encoded.push(char::from(byte));
            } else {
                push_percent_encoded(&mut encoded, byte);
            }
            byte_index += 1;
        }
        if encoded.len() > MAX_REQUEST_TARGET_BYTES {
            return Err(HttpError::RequestTargetTooLarge {
                byte_count: encoded.len(),
                maximum_bytes: MAX_REQUEST_TARGET_BYTES,
            });
        }
        let query_index = encoded.find('?');
        let path_end = query_index.unwrap_or(encoded.len());
        let prefix_end = path_end.min(MAX_EVIDENCE_PATH_PREFIX_BYTES);
        let path_prefix = encoded[..prefix_end].to_owned();
        let target_hash = target_identifier(&origin, encoded.as_bytes());
        Ok(Self {
            origin,
            encoded_path_and_query: encoded,
            target_hash,
            query_present: query_index.is_some(),
            path_prefix,
        })
    }

    /// Return the canonical origin that owns the target.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the encoded origin-form path and optional query used on the wire.
    #[must_use]
    pub const fn path_and_query(&self) -> &str {
        self.encoded_path_and_query.as_str()
    }

    /// Return a domain-separated SHA-256 identifier for the exact target.
    #[must_use]
    pub const fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Return whether the exact request target contains a query component.
    #[must_use]
    pub const fn query_present(&self) -> bool {
        self.query_present
    }

    /// Return the bounded encoded path prefix retained without query values.
    #[must_use]
    pub const fn path_prefix(&self) -> &str {
        self.path_prefix.as_str()
    }
}

fn push_percent_encoded(output: &mut String, byte: u8) {
    output.push('%');
    output.push(char::from(HEX_UPPER[usize::from(byte >> 4)]));
    output.push(char::from(HEX_UPPER[usize::from(byte & 0x0f)]));
}

fn target_identifier(origin: &Origin, target: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"originweave-http-target-v1\0");
    hasher.update(origin.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(target);
    let digest = hasher.finalize();
    let mut identifier = String::with_capacity(71);
    identifier.push_str("sha256:");
    for byte in digest {
        identifier.push(char::from(HEX_UPPER[usize::from(byte >> 4)]).to_ascii_lowercase());
        identifier.push(char::from(HEX_UPPER[usize::from(byte & 0x0f)]).to_ascii_lowercase());
    }
    identifier
}
