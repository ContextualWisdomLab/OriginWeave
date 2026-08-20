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
) -> ExtractionField {
    ExtractionField::new(
        identifier,
        value_type,
        cardinality,
        required,
        source_channels,
    )
    .expect("fixture field must be valid")
}

#[test]
fn schema_binds_versioned_typed_fields_to_explicit_source_channels() {
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
            ),
            field(
                "unit_price",
                ExtractionValueType::Decimal,
                ExtractionCardinality::ZeroOrOne,
                false,
                &[
                    ExtractionSourceChannel::TableCell,
                    ExtractionSourceChannel::NetworkResponse,
                ],
            ),
        ],
    )
    .expect("schema must be admitted");

    assert_eq!(schema.version(), "product-card-v1");
    assert_eq!(schema.fields().len(), 2);
    assert_eq!(
        schema.field("product_name").unwrap().identifier(),
        "product_name"
    );
    assert_eq!(
        schema.field("product_name").unwrap().value_type(),
        ExtractionValueType::Text
    );
    assert_eq!(
        schema.field("product_name").unwrap().cardinality(),
        ExtractionCardinality::One
    );
    assert!(schema.field("product_name").unwrap().required());
    assert_eq!(
        schema.field("product_name").unwrap().source_channels(),
        &[
            ExtractionSourceChannel::SemanticNode,
            ExtractionSourceChannel::StructuredData,
        ]
    );
    assert_eq!(
        schema.field("unit_price").unwrap().value_type(),
        ExtractionValueType::Decimal
    );
    assert_eq!(
        schema.field("unit_price").unwrap().cardinality(),
        ExtractionCardinality::ZeroOrOne
    );
    assert!(!schema.field("unit_price").unwrap().required());
    assert!(schema.field("missing_field").is_none());
}

#[test]
fn field_accepts_all_reviewed_value_and_source_channel_variants() {
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
        );
        assert_eq!(field.value_type(), value_type);
        assert_eq!(field.cardinality(), ExtractionCardinality::Many);
        assert_eq!(field.source_channels(), &[source_channel]);
    }
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
fn schema_rejects_invalid_version_empty_fields_duplicate_fields_and_field_overflow() {
    assert_eq!(
        ExtractionSchema::new(
            "Product Schema",
            vec![field(
                "product_name",
                ExtractionValueType::Text,
                ExtractionCardinality::One,
                true,
                &[ExtractionSourceChannel::SemanticNode],
            )]
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
            )],
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
    );
    let duplicate_again = field(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::ZeroOrOne,
        false,
        &[ExtractionSourceChannel::StructuredData],
    );
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
        .collect();
    assert_eq!(
        ExtractionSchema::new("product-card-v1", too_many_fields),
        Err(ExtractionSchemaError::LimitExceeded)
    );
}
