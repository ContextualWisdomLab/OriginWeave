use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionSchema, ExtractionSchemaError,
    ExtractionSourceChannel, ExtractionValueType, MAX_EXTRACTION_FIELD_COUNT,
    MAX_EXTRACTION_IDENTIFIER_BYTES,
};

fn field(identifier: &str) -> Result<ExtractionField, ExtractionSchemaError> {
    ExtractionField::new(
        identifier,
        ExtractionValueType::Text,
        ExtractionCardinality::ZeroOrOne,
        false,
        &[ExtractionSourceChannel::SemanticNode],
    )
}

#[test]
fn exact_128_byte_field_and_version_identifiers_are_preserved() -> Result<(), ExtractionSchemaError>
{
    assert_eq!(MAX_EXTRACTION_IDENTIFIER_BYTES, 128);
    let identifier = "a".repeat(128);
    let accepted_field = field(&identifier)?;
    assert_eq!(accepted_field.identifier(), identifier);
    let schema = ExtractionSchema::new(&identifier, vec![accepted_field])?;
    assert_eq!(schema.version(), identifier);
    let over_limit = "a".repeat(129);
    assert_eq!(
        field(&over_limit),
        Err(ExtractionSchemaError::LimitExceeded)
    );
    assert_eq!(
        ExtractionSchema::new(&over_limit, vec![field("field")?]),
        Err(ExtractionSchemaError::LimitExceeded)
    );
    Ok(())
}

#[test]
fn exact_256_distinct_fields_retain_order_and_lookup() -> Result<(), ExtractionSchemaError> {
    assert_eq!(MAX_EXTRACTION_FIELD_COUNT, 256);
    let fields = (0..256)
        .map(|index| field(&format!("field_{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = fields.clone();
    let schema = ExtractionSchema::new("schema-v1", fields)?;
    assert_eq!(schema.fields(), expected);
    for (index, accepted_field) in schema.fields().iter().enumerate() {
        assert_eq!(accepted_field.identifier(), format!("field_{index}"));
        assert_eq!(
            schema.field(accepted_field.identifier()),
            Some(accepted_field)
        );
    }
    let mut too_many = expected;
    too_many.push(field("field_256")?);
    assert_eq!(
        ExtractionSchema::new("schema-v1", too_many),
        Err(ExtractionSchemaError::LimitExceeded)
    );
    Ok(())
}
