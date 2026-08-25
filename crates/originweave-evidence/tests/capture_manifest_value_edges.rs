use originweave_evidence::{
    CaptureManifestError, CaptureManifestValueBinding, MAX_EXTRACTION_IDENTIFIER_BYTES,
};

const VALUE_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";

#[test]
fn structured_value_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            CaptureManifestError::ValueLimitExceeded,
            "capture manifest structured-value limit exceeded",
        ),
        (
            CaptureManifestError::InvalidValueField,
            "capture manifest structured-value field is invalid",
        ),
        (
            CaptureManifestError::InvalidValueDigest,
            "capture manifest structured-value digest is not canonical SHA-256",
        ),
        (
            CaptureManifestError::UnknownValueField,
            "capture manifest structured-value field is absent from the schema",
        ),
        (
            CaptureManifestError::ValueSourceRecordMissing,
            "capture manifest structured value references an absent WARC record",
        ),
        (
            CaptureManifestError::ValueSourceChannelMismatch,
            "capture manifest structured value is not admitted by the field source channels",
        ),
        (
            CaptureManifestError::DuplicateValue,
            "capture manifest contains a duplicate structured value",
        ),
        (
            CaptureManifestError::ValueCardinalityExceeded,
            "capture manifest structured value exceeds field cardinality",
        ),
        (
            CaptureManifestError::RequiredValueMissing,
            "capture manifest is missing a required structured value",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn value_binding_rejects_every_identifier_and_digest_shape_boundary() {
    let overlong_field = "a".repeat(MAX_EXTRACTION_IDENTIFIER_BYTES + 1);
    for invalid_field in ["", "Title", "title.value", overlong_field.as_str()] {
        assert_eq!(
            CaptureManifestValueBinding::new(invalid_field, VALUE_HASH, RECORD_ID),
            Err(CaptureManifestError::InvalidValueField)
        );
    }

    assert_eq!(
        CaptureManifestValueBinding::new("title", "not-a-sha256", RECORD_ID),
        Err(CaptureManifestError::InvalidValueDigest)
    );
    let invalid_hex_digest = format!("sha256:{}", "g".repeat(64));
    assert_eq!(
        CaptureManifestValueBinding::new("title", &invalid_hex_digest, RECORD_ID),
        Err(CaptureManifestError::InvalidValueDigest)
    );

    let numeric_hex_digest = format!("sha256:{}", "0".repeat(64));
    assert!(CaptureManifestValueBinding::new("title", &numeric_hex_digest, RECORD_ID).is_ok());
    assert!(CaptureManifestValueBinding::new("title_1-tag", VALUE_HASH, RECORD_ID).is_ok());
}
