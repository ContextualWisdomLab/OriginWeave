use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionNormalizationRule, ExtractionSchemaError,
    ExtractionSourceChannel, ExtractionValueType,
};

#[test]
fn extraction_fields_require_an_explicit_typed_normalization_rule()
-> Result<(), ExtractionSchemaError> {
    let text = ExtractionField::new_with_normalization(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        ExtractionNormalizationRule::TrimTextWhitespace,
        &[ExtractionSourceChannel::SemanticNode],
    )?;
    assert_eq!(
        text.normalization_rule(),
        ExtractionNormalizationRule::TrimTextWhitespace
    );

    let timestamp = ExtractionField::new_with_normalization(
        "captured_at",
        ExtractionValueType::Timestamp,
        ExtractionCardinality::One,
        true,
        ExtractionNormalizationRule::Rfc3339Utc,
        &[ExtractionSourceChannel::NetworkResponse],
    )?;
    assert_eq!(
        timestamp.normalization_rule(),
        ExtractionNormalizationRule::Rfc3339Utc
    );
    Ok(())
}

#[test]
fn extraction_fields_fail_closed_on_type_incompatible_normalization() {
    assert_eq!(
        ExtractionField::new_with_normalization(
            "captured_at",
            ExtractionValueType::Timestamp,
            ExtractionCardinality::One,
            true,
            ExtractionNormalizationRule::TrimTextWhitespace,
            &[ExtractionSourceChannel::NetworkResponse],
        ),
        Err(ExtractionSchemaError::InvalidNormalizationRule)
    );
    assert_eq!(
        ExtractionField::new_with_normalization(
            "product_name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            ExtractionNormalizationRule::Rfc3339Utc,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidNormalizationRule)
    );
}

#[test]
fn existing_fields_default_to_verbatim_normalization()
-> Result<(), ExtractionSchemaError> {
    let field = ExtractionField::new(
        "unit_price",
        ExtractionValueType::Decimal,
        ExtractionCardinality::ZeroOrOne,
        false,
        &[ExtractionSourceChannel::StructuredData],
    )?;
    assert_eq!(
        field.normalization_rule(),
        ExtractionNormalizationRule::Verbatim
    );
    Ok(())
}
