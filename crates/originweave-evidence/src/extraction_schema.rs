//! Versioned schema contracts for typed evidence extraction.
//!
//! These value objects describe what may be extracted and which reviewed
//! evidence channels may support each field. They do not read browser data,
//! disclose protected values, persist artifacts, execute models, or grant any
//! browser, network, secret, approval, or storage authority.

use std::collections::BTreeSet;

/// Maximum encoded byte length for an extraction schema or field identifier.
pub const MAX_EXTRACTION_IDENTIFIER_BYTES: usize = 128;
/// Maximum number of fields admitted by one extraction schema.
pub const MAX_EXTRACTION_FIELD_COUNT: usize = 256;

/// The typed value contract for one extracted field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtractionValueType {
    /// Bounded textual data.
    Text,
    /// A whole-number value.
    Integer,
    /// A decimal numeric value.
    Decimal,
    /// A boolean value.
    Boolean,
    /// A timestamp value whose concrete normalization is defined by the schema version.
    Timestamp,
}

/// The number of values admitted for one extracted field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtractionCardinality {
    /// Exactly one value is admitted.
    One,
    /// Zero or one value is admitted.
    ZeroOrOne,
    /// A bounded collection may be admitted by a later extraction runtime.
    Many,
}

/// A reviewed evidence channel that may support an extracted value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtractionSourceChannel {
    /// A semantic browser node with an independently validated identity.
    SemanticNode,
    /// Embedded structured metadata such as JSON-LD, RDFa, or Microdata.
    StructuredData,
    /// A bounded table-cell observation.
    TableCell,
    /// A bounded network response whose origin and response identity are independently verified.
    NetworkResponse,
    /// A separately approved model interpretation backed by explicit evidence identifiers.
    ModelInterpretation,
}

/// A validation failure while constructing an extraction schema contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionSchemaError {
    /// A schema or field identifier was empty or outside the accepted identifier grammar.
    InvalidIdentifier,
    /// An identifier or field collection exceeded its bounded limit.
    LimitExceeded,
    /// A field did not declare any reviewed source channel.
    MissingSourceChannel,
    /// A field declared the same source channel more than once.
    DuplicateSourceChannel,
    /// A schema did not contain any field definitions.
    MissingField,
    /// A schema declared the same field identifier more than once.
    DuplicateField,
}

/// One typed field declared by a versioned extraction schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionField {
    identifier: String,
    value_type: ExtractionValueType,
    cardinality: ExtractionCardinality,
    required: bool,
    source_channels: Vec<ExtractionSourceChannel>,
}

impl ExtractionField {
    /// Validate and construct one extraction field contract.
    pub fn new(
        identifier: &str,
        value_type: ExtractionValueType,
        cardinality: ExtractionCardinality,
        required: bool,
        source_channels: &[ExtractionSourceChannel],
    ) -> Result<Self, ExtractionSchemaError> {
        validate_identifier(identifier)?;
        if source_channels.is_empty() {
            return Err(ExtractionSchemaError::MissingSourceChannel);
        }

        let mut seen_channels = BTreeSet::new();
        for source_channel in source_channels {
            if !seen_channels.insert(*source_channel) {
                return Err(ExtractionSchemaError::DuplicateSourceChannel);
            }
        }

        Ok(Self {
            identifier: identifier.to_owned(),
            value_type,
            cardinality,
            required,
            source_channels: source_channels.to_vec(),
        })
    }

    /// Return the stable field identifier.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Return the declared value type.
    #[must_use]
    pub const fn value_type(&self) -> ExtractionValueType {
        self.value_type
    }

    /// Return the declared cardinality.
    #[must_use]
    pub const fn cardinality(&self) -> ExtractionCardinality {
        self.cardinality
    }

    /// Return whether the field must be present in a conforming extraction result.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }

    /// Return the reviewed source channels that may support this field.
    #[must_use]
    pub fn source_channels(&self) -> &[ExtractionSourceChannel] {
        &self.source_channels
    }
}

/// A bounded versioned collection of typed extraction-field contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionSchema {
    version: String,
    fields: Vec<ExtractionField>,
}

impl ExtractionSchema {
    /// Validate and construct one versioned extraction schema.
    pub fn new(
        version: &str,
        fields: Vec<ExtractionField>,
    ) -> Result<Self, ExtractionSchemaError> {
        validate_identifier(version)?;
        if fields.is_empty() {
            return Err(ExtractionSchemaError::MissingField);
        }
        if fields.len() > MAX_EXTRACTION_FIELD_COUNT {
            return Err(ExtractionSchemaError::LimitExceeded);
        }

        let mut field_identifiers = BTreeSet::new();
        for field in &fields {
            if !field_identifiers.insert(field.identifier()) {
                return Err(ExtractionSchemaError::DuplicateField);
            }
        }

        Ok(Self {
            version: version.to_owned(),
            fields,
        })
    }

    /// Return the immutable schema version identifier.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Return the schema's ordered field definitions.
    #[must_use]
    pub fn fields(&self) -> &[ExtractionField] {
        &self.fields
    }

    /// Find one field by its stable identifier.
    #[must_use]
    pub fn field(&self, identifier: &str) -> Option<&ExtractionField> {
        self.fields
            .iter()
            .find(|field| field.identifier() == identifier)
    }
}

fn validate_identifier(identifier: &str) -> Result<(), ExtractionSchemaError> {
    if identifier.len() > MAX_EXTRACTION_IDENTIFIER_BYTES {
        return Err(ExtractionSchemaError::LimitExceeded);
    }

    let mut bytes = identifier.bytes();
    let Some(first_byte) = bytes.next() else {
        return Err(ExtractionSchemaError::InvalidIdentifier);
    };
    if !first_byte.is_ascii_lowercase() {
        return Err(ExtractionSchemaError::InvalidIdentifier);
    }
    if bytes.any(|byte| !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-')) {
        return Err(ExtractionSchemaError::InvalidIdentifier);
    }

    Ok(())
}
