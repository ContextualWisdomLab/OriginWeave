use std::fmt;

use crate::{DEFAULT_MAX_HEADER_NAME_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES, HttpError};

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
    inner: FieldLine,
}

impl RequestField {
    /// Validate one caller field without retaining it in logs or evidence.
    pub fn new(name: &str, value: &[u8]) -> Result<Self, HttpError> {
        let inner = FieldLine::new(
            name.as_bytes(),
            value,
            DEFAULT_MAX_HEADER_NAME_BYTES,
            DEFAULT_MAX_HEADER_VALUE_BYTES,
        )
        .map_err(request_field_error)?;
        if FORBIDDEN_REQUEST_FIELDS.contains(&inner.name()) {
            return Err(HttpError::ForbiddenRequestField {
                field_name: inner.name().to_owned(),
            });
        }
        Ok(Self { inner })
    }

    /// Return the normalized lowercase field name.
    #[must_use]
    pub const fn name(&self) -> &str {
        self.inner.name()
    }

    /// Return the field-value byte count without exposing its bytes.
    #[must_use]
    pub const fn value_byte_count(&self) -> usize {
        self.inner.value().len()
    }

    pub(crate) const fn value(&self) -> &[u8] {
        self.inner.value()
    }
}

impl fmt::Debug for RequestField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestField")
            .field("name", &self.name())
            .field("value_byte_count", &self.value_byte_count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FieldLine {
    name: String,
    value: Vec<u8>,
}

impl FieldLine {
    pub(crate) fn new(
        name: &[u8],
        value: &[u8],
        maximum_name_bytes: usize,
        maximum_value_bytes: usize,
    ) -> Result<Self, FieldSyntaxError> {
        if name.is_empty() || !name.iter().copied().all(is_token_byte) {
            return Err(FieldSyntaxError::InvalidName);
        }
        if name.len() > maximum_name_bytes {
            return Err(FieldSyntaxError::NameTooLarge {
                byte_count: name.len(),
                maximum_bytes: maximum_name_bytes,
            });
        }
        if value.len() > maximum_value_bytes {
            return Err(FieldSyntaxError::ValueTooLarge {
                byte_count: value.len(),
                maximum_bytes: maximum_value_bytes,
            });
        }
        if !value.iter().copied().all(is_field_value_byte) {
            return Err(FieldSyntaxError::InvalidValue);
        }
        // `is_token_byte` admits ASCII only, so lowercase normalization can construct the
        // canonical field name directly without a fallible UTF-8 conversion branch.
        let name = name
            .iter()
            .map(|byte| char::from(byte.to_ascii_lowercase()))
            .collect();
        Ok(Self {
            name,
            value: value.to_vec(),
        })
    }

    pub(crate) const fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) const fn value(&self) -> &[u8] {
        self.value.as_slice()
    }
}

impl fmt::Debug for FieldLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldLine")
            .field("name", &self.name)
            .field("value_byte_count", &self.value.len())
            .finish()
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct FieldBlock {
    fields: Vec<FieldLine>,
}

impl FieldBlock {
    pub(crate) fn new(fields: Vec<FieldLine>) -> Self {
        Self { fields }
    }

    pub(crate) const fn len(&self) -> usize {
        self.fields.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FieldLine> {
        self.fields.iter()
    }

    pub(crate) fn values(&self, name: &str) -> Vec<&[u8]> {
        let mut values = Vec::with_capacity(self.len());
        values.extend(
            self.fields
                .iter()
                .filter(|field| field.name() == name)
                .map(FieldLine::value),
        );
        values
    }
}

impl fmt::Debug for FieldBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(&self.fields).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldSyntaxError {
    InvalidName,
    InvalidValue,
    NameTooLarge {
        byte_count: usize,
        maximum_bytes: usize,
    },
    ValueTooLarge {
        byte_count: usize,
        maximum_bytes: usize,
    },
}

fn request_field_error(error: FieldSyntaxError) -> HttpError {
    match error {
        FieldSyntaxError::InvalidName => HttpError::InvalidRequestFieldName,
        FieldSyntaxError::InvalidValue => HttpError::InvalidRequestFieldValue,
        FieldSyntaxError::NameTooLarge {
            byte_count,
            maximum_bytes,
        } => HttpError::RequestFieldNameTooLarge {
            byte_count,
            maximum_bytes,
        },
        FieldSyntaxError::ValueTooLarge {
            byte_count,
            maximum_bytes,
        } => HttpError::RequestFieldValueTooLarge {
            byte_count,
            maximum_bytes,
        },
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn private_field_debug_is_structural_and_never_exposes_values() {
        let field = FieldLine::new(
            b"X-Trace",
            b"secret-value",
            DEFAULT_MAX_HEADER_NAME_BYTES,
            DEFAULT_MAX_HEADER_VALUE_BYTES,
        )
        .expect("valid internal field");

        let field_debug = format!("{field:?}");
        assert!(field_debug.contains("FieldLine"));
        assert!(field_debug.contains("x-trace"));
        assert!(field_debug.contains("value_byte_count"));
        assert!(!field_debug.contains("secret-value"));

        let block = FieldBlock::new(vec![field]);
        let block_debug = format!("{block:?}");
        assert!(block_debug.contains("x-trace"));
        assert!(block_debug.contains("value_byte_count"));
        assert!(!block_debug.contains("secret-value"));
    }
}
