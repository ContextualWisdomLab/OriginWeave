use std::error::Error as _;

use originweave_evidence::ExtractionSchemaError;

fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn extraction_schema_errors_implement_standard_error_contract() {
    assert_standard_error_contract::<ExtractionSchemaError>();

    for (error, message) in [
        (
            ExtractionSchemaError::InvalidIdentifier,
            "invalid extraction schema identifier",
        ),
        (
            ExtractionSchemaError::LimitExceeded,
            "extraction schema limit exceeded",
        ),
        (
            ExtractionSchemaError::MissingSourceChannel,
            "extraction field requires at least one source channel",
        ),
        (
            ExtractionSchemaError::DuplicateSourceChannel,
            "extraction field contains a duplicate source channel",
        ),
        (
            ExtractionSchemaError::InvalidNormalizationRule,
            "extraction normalization rule is incompatible with the field value type",
        ),
        (
            ExtractionSchemaError::MissingField,
            "extraction schema requires at least one field",
        ),
        (
            ExtractionSchemaError::DuplicateField,
            "extraction schema contains a duplicate field identifier",
        ),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(error.source().is_none());
    }
}
