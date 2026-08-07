use std::fmt;

use crate::{
    DEFAULT_MAX_HEADER_NAME_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES, HttpError,
};

const FORBIDDEN_REQUEST_FIELDS: &[&str] = &[
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

/// One validated non-authoritative HTTP request field.
#[derive(Clone, PartialEq, Eq)]
pub struct RequestField {
    name: String,
    value: Vec<u8>,
}

impl RequestField {
    /// Validate one caller field without retaining it in logs or evidence.
    pub fn new(name: &str, value: &[u8]) -> Result<Self, HttpError> {
        if name.is_empty() || !name.as_bytes().iter().copied().all(is_token_byte) {
            return Err(HttpError::InvalidRequestFieldName);
        }
        if name.len() > DEFAULT_MAX_HEADER_NAME_BYTES {
            return Err(HttpError::RequestFieldNameTooLarge {
                byte_count: name.len(),
                maximum_bytes: DEFAULT_MAX_HEADER_NAME_BYTES,
            });
        }
        if value.len() > DEFAULT_MAX_HEADER_VALUE_BYTES {
            return Err(HttpError::RequestFieldValueTooLarge {
                byte_count: value.len(),
                maximum_bytes: DEFAULT_MAX_HEADER_VALUE_BYTES,
            });
        }
        if !value.iter().copied().all(is_field_value_byte) {
            return Err(HttpError::InvalidRequestFieldValue);
        }
        let normalized_name = name.to_ascii_lowercase();
        if FORBIDDEN_REQUEST_FIELDS.contains(&normalized_name.as_str()) {
            return Err(HttpError::ForbiddenRequestField {
                field_name: normalized_name,
            });
        }
        Ok(Self {
            name: normalized_name,
            value: value.to_vec(),
        })
    }

    /// Return the normalized lowercase field name.
    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the field-value byte count without exposing its bytes.
    #[must_use]
    pub const fn value_byte_count(&self) -> usize {
        self.value.len()
    }

    pub(crate) const fn value(&self) -> &[u8] {
        self.value.as_slice()
    }
}

impl fmt::Debug for RequestField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestField")
            .field("name", &self.name)
            .field("value_byte_count", &self.value.len())
            .finish()
    }
}

pub(crate) const fn is_token_byte(byte: u8) -> bool {
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

pub(crate) const fn is_field_value_byte(byte: u8) -> bool {
    byte == b'\t' || (byte >= 0x20 && byte != 0x7f)
}
