#![allow(clippy::expect_used)]

use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionSourceChannel, ExtractionValueType,
};

#[test]
fn every_five_channel_permutation_has_the_same_reviewed_identity() {
    let canonical = [
        ExtractionSourceChannel::SemanticNode,
        ExtractionSourceChannel::StructuredData,
        ExtractionSourceChannel::TableCell,
        ExtractionSourceChannel::NetworkResponse,
        ExtractionSourceChannel::ModelInterpretation,
    ];
    let expected = ExtractionField::new(
        "all_sources",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &canonical,
    )
    .expect("all five reviewed channels must be valid");
    assert_eq!(expected.source_channels(), &canonical);

    let mut permutations = 0;
    for a in 0..5 {
        for b in 0..5 {
            if b == a {
                continue;
            }
            for c in 0..5 {
                if c == a || c == b {
                    continue;
                }
                for d in 0..5 {
                    if d == a || d == b || d == c {
                        continue;
                    }
                    for e in 0..5 {
                        if e == a || e == b || e == c || e == d {
                            continue;
                        }
                        let input = [
                            canonical[a],
                            canonical[b],
                            canonical[c],
                            canonical[d],
                            canonical[e],
                        ];
                        let field = ExtractionField::new(
                            "all_sources",
                            ExtractionValueType::Text,
                            ExtractionCardinality::One,
                            true,
                            &input,
                        )
                        .expect("each five-channel permutation must be valid");
                        assert_eq!(
                            field.source_channels(),
                            &canonical,
                            "channel order must match the independent reviewed order for {input:?}",
                        );
                        assert_eq!(
                            field, expected,
                            "channel order must not alter field identity"
                        );
                        permutations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(permutations, 120, "all five-channel permutations must run");
}
