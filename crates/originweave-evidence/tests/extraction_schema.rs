use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionSchema, ExtractionSchemaError,
    ExtractionSourceChannel, ExtractionValueType, MAX_EXTRACTION_FIELD_COUNT,
    MAX_EXTRACTION_IDENTIFIER_BYTES,
};

fn field(
    identifier: &str,
    value_type: ExtractionValueType,
    cardinality: ExtractionCardinality,
    required: bool,
    source_channels: &[ExtractionSourceChannel],
) -> Result<ExtractionField, ExtractionSchemaError> {
    ExtractionField::new(
        identifier,
        value_type,
        cardinality,
        required,
        source_channels,
    )
}

#[test]
fn schema_binds_versioned_typed_fields_to_explicit_source_channels()
-> Result<(), ExtractionSchemaError> {
    let schema = ExtractionSchema::new(
        "product-card-v1",
        vec![
            field(
                "product_name",
                ExtractionValueType::Text,
                ExtractionCardinality::One,
                true,
                &[
                    ExtractionSourceChannel::SemanticNode,
                    ExtractionSourceChannel::StructuredData,
                ],
            )?,
            field(
                "unit_price",
                ExtractionValueType::Decimal,
                ExtractionCardinality::ZeroOrOne,
                false,
                &[
                    ExtractionSourceChannel::TableCell,
                    ExtractionSourceChannel::NetworkResponse,
                ],
            )?,
        ],
    )?;

    assert_eq!(schema.version(), "product-card-v1");
    assert_eq!(schema.fields().len(), 2);
    assert_eq!(
        schema
            .field("product_name")
            .map(ExtractionField::identifier),
        Some("product_name")
    );
    assert_eq!(
        schema
            .field("product_name")
            .map(ExtractionField::value_type),
        Some(ExtractionValueType::Text)
    );
    assert_eq!(
        schema
            .field("product_name")
            .map(ExtractionField::cardinality),
        Some(ExtractionCardinality::One)
    );
    assert_eq!(
        schema.field("product_name").map(ExtractionField::required),
        Some(true)
    );
    let expected_product_sources = [
        ExtractionSourceChannel::SemanticNode,
        ExtractionSourceChannel::StructuredData,
    ];
    assert_eq!(
        schema
            .field("product_name")
            .map(ExtractionField::source_channels),
        Some(expected_product_sources.as_slice())
    );
    assert_eq!(
        schema.field("unit_price").map(ExtractionField::value_type),
        Some(ExtractionValueType::Decimal)
    );
    assert_eq!(
        schema.field("unit_price").map(ExtractionField::cardinality),
        Some(ExtractionCardinality::ZeroOrOne)
    );
    assert_eq!(
        schema.field("unit_price").map(ExtractionField::required),
        Some(false)
    );
    assert!(schema.field("missing_field").is_none());
    Ok(())
}

#[test]
fn field_accepts_all_reviewed_value_and_source_channel_variants()
-> Result<(), ExtractionSchemaError> {
    let cases = [
        (
            ExtractionValueType::Text,
            ExtractionSourceChannel::SemanticNode,
        ),
        (
            ExtractionValueType::Integer,
            ExtractionSourceChannel::StructuredData,
        ),
        (
            ExtractionValueType::Decimal,
            ExtractionSourceChannel::TableCell,
        ),
        (
            ExtractionValueType::Boolean,
            ExtractionSourceChannel::NetworkResponse,
        ),
        (
            ExtractionValueType::Timestamp,
            ExtractionSourceChannel::ModelInterpretation,
        ),
    ];

    for (index, (value_type, source_channel)) in cases.into_iter().enumerate() {
        let field = field(
            &format!("field_{index}"),
            value_type,
            ExtractionCardinality::Many,
            false,
            &[source_channel],
        )?;
        assert_eq!(field.value_type(), value_type);
        assert_eq!(field.cardinality(), ExtractionCardinality::Many);
        assert_eq!(field.source_channels(), &[source_channel]);
    }

    let required_many = field(
        "required_many",
        ExtractionValueType::Text,
        ExtractionCardinality::Many,
        true,
        &[ExtractionSourceChannel::SemanticNode],
    )?;
    assert!(required_many.required());
    Ok(())
}

#[test]
fn field_rejects_contradictory_required_cardinality_contracts() {
    assert_eq!(
        ExtractionField::new(
            "optional_exactly_one",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            false,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidCardinalityRequirement)
    );
    assert_eq!(
        ExtractionField::new(
            "required_zero_or_one",
            ExtractionValueType::Text,
            ExtractionCardinality::ZeroOrOne,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidCardinalityRequirement)
    );
}

#[test]
fn field_rejects_empty_malformed_or_overlong_identifiers() {
    assert_eq!(
        ExtractionField::new(
            "",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidIdentifier)
    );
    assert_eq!(
        ExtractionField::new(
            "Product Name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidIdentifier)
    );
    assert_eq!(
        ExtractionField::new(
            "product name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidIdentifier)
    );
    assert_eq!(
        ExtractionField::new(
            "1product_name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::InvalidIdentifier)
    );
    assert_eq!(
        ExtractionField::new(
            &"a".repeat(MAX_EXTRACTION_IDENTIFIER_BYTES + 1),
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[ExtractionSourceChannel::SemanticNode],
        ),
        Err(ExtractionSchemaError::LimitExceeded)
    );
}

#[test]
fn field_requires_a_nonempty_duplicate_free_source_channel_set() {
    assert_eq!(
        ExtractionField::new(
            "product_name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[],
        ),
        Err(ExtractionSchemaError::MissingSourceChannel)
    );
    assert_eq!(
        ExtractionField::new(
            "product_name",
            ExtractionValueType::Text,
            ExtractionCardinality::One,
            true,
            &[
                ExtractionSourceChannel::SemanticNode,
                ExtractionSourceChannel::SemanticNode,
            ],
        ),
        Err(ExtractionSchemaError::DuplicateSourceChannel)
    );
}

#[test]
fn schema_rejects_invalid_version_empty_fields_duplicate_fields_and_field_overflow()
-> Result<(), ExtractionSchemaError> {
    assert_eq!(
        ExtractionSchema::new(
            "Product Schema",
            vec![field(
                "product_name",
                ExtractionValueType::Text,
                ExtractionCardinality::One,
                true,
                &[ExtractionSourceChannel::SemanticNode],
            )?]
        ),
        Err(ExtractionSchemaError::InvalidIdentifier)
    );
    assert_eq!(
        ExtractionSchema::new(
            &"a".repeat(MAX_EXTRACTION_IDENTIFIER_BYTES + 1),
            vec![field(
                "product_name",
                ExtractionValueType::Text,
                ExtractionCardinality::One,
                true,
                &[ExtractionSourceChannel::SemanticNode],
            )?],
        ),
        Err(ExtractionSchemaError::LimitExceeded)
    );
    assert_eq!(
        ExtractionSchema::new("product-card-v1", vec![]),
        Err(ExtractionSchemaError::MissingField)
    );

    let duplicate = field(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::SemanticNode],
    )?;
    let duplicate_again = field(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::ZeroOrOne,
        false,
        &[ExtractionSourceChannel::StructuredData],
    )?;
    assert_eq!(
        ExtractionSchema::new("product-card-v1", vec![duplicate, duplicate_again]),
        Err(ExtractionSchemaError::DuplicateField)
    );

    let too_many_fields = (0..=MAX_EXTRACTION_FIELD_COUNT)
        .map(|index| {
            field(
                &format!("field_{index}"),
                ExtractionValueType::Text,
                ExtractionCardinality::ZeroOrOne,
                false,
                &[ExtractionSourceChannel::SemanticNode],
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        ExtractionSchema::new("product-card-v1", too_many_fields),
        Err(ExtractionSchemaError::LimitExceeded)
    );
    Ok(())
}
