use crate::HttpError;

const BLOCKED_REQUEST_FIELDS: &[&str] = &[
    "host",
    "connection",
    "proxy-connection",
    "keep-alive",
    "transfer-encoding",
    "content-length",
    "trailer",
    "te",
    "upgrade",
    "authorization",
    "proxy-authorization",
    "cookie",
];

/// One validated caller-supplied non-authoritative HTTP request field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestField {
    name: String,
    value: Vec<u8>,
}

impl RequestField {
    /// Validate one request field and reject authority, credential, and framing names.
    pub fn new(name: &str, value: &[u8]) -> Result<Self, HttpError> {
        if name.is_empty() || !name.as_bytes().iter().copied().all(is_token_byte) {
            return Err(HttpError::InvalidRequestField);
        }
        let normalized = name.to_ascii_lowercase();
        if BLOCKED_REQUEST_FIELDS.contains(&normalized.as_str()) {
            return Err(HttpError::ForbiddenRequestField);
        }
        if !value.iter().copied().all(is_field_value_byte) {
            return Err(HttpError::InvalidRequestField);
        }
        Ok(Self {
            name: normalized,
            value: value.to_vec(),
        })
    }

    /// Return the lowercase validated field name.
    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the validated opaque field value bytes.
    #[must_use]
    pub const fn value(&self) -> &[u8] {
        self.value.as_slice()
    }
}

pub(crate) fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
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

pub(crate) fn is_field_value_byte(byte: u8) -> bool {
    byte == b'\t' || byte == b' ' || (0x21..=0x7e).contains(&byte) || byte >= 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_and_value_predicates_cover_boundaries() {
        assert!(is_token_byte(b'A'));
        assert!(is_token_byte(b'~'));
        assert!(!is_token_byte(b':'));
        assert!(is_field_value_byte(b'\t'));
        assert!(is_field_value_byte(b' '));
        assert!(is_field_value_byte(b'!'));
        assert!(is_field_value_byte(0x80));
        assert!(!is_field_value_byte(b'\r'));
        assert!(!is_field_value_byte(0x7f));
    }
}
