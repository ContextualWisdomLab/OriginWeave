use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionNormalizationRule, ExtractionSchemaError,
    ExtractionSourceChannel, ExtractionValueType,
};

#[test]
fn every_value_type_normalization_rule_pair_matches_the_reviewed_matrix()
-> Result<(), ExtractionSchemaError> {
    let value_types = [
        ExtractionValueType::Text,
        ExtractionValueType::Integer,
        ExtractionValueType::Decimal,
        ExtractionValueType::Boolean,
        ExtractionValueType::Timestamp,
    ];
    let rules = [
        ExtractionNormalizationRule::Verbatim,
        ExtractionNormalizationRule::TrimTextWhitespace,
        ExtractionNormalizationRule::Rfc3339Utc,
    ];

    for value_type in value_types {
        for rule in rules {
            let compatible = matches!(
                (value_type, rule),
                (_, ExtractionNormalizationRule::Verbatim)
                    | (
                        ExtractionValueType::Text,
                        ExtractionNormalizationRule::TrimTextWhitespace
                    )
                    | (
                        ExtractionValueType::Timestamp,
                        ExtractionNormalizationRule::Rfc3339Utc
                    )
            );

            let created = ExtractionField::new_with_normalization(
                "field",
                value_type,
                ExtractionCardinality::Many,
                false,
                rule,
                &[ExtractionSourceChannel::SemanticNode],
            );
            assert_eq!(
                created.is_ok(),
                compatible,
                "{value_type:?} + {rule:?} compatibility must match the reviewed matrix",
            );

            match created {
                Ok(field) => {
                    assert_eq!(field.value_type(), value_type);
                    assert_eq!(field.normalization_rule(), rule);
                }
                Err(error) => {
                    assert_eq!(error, ExtractionSchemaError::InvalidNormalizationRule);
                }
            }
        }
    }
    Ok(())
}
