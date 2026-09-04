#![allow(clippy::expect_used)]

use originweave_evidence::{
    ExtractionCardinality, ExtractionField, ExtractionSourceChannel, ExtractionValueType,
};

#[test]
fn equivalent_source_channel_sets_have_canonical_identity() {
    let semantic_then_network = ExtractionField::new(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[
            ExtractionSourceChannel::SemanticNode,
            ExtractionSourceChannel::NetworkResponse,
        ],
    )
    .expect("reviewed source set must be valid");
    let network_then_semantic = ExtractionField::new(
        "product_name",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[
            ExtractionSourceChannel::NetworkResponse,
            ExtractionSourceChannel::SemanticNode,
        ],
    )
    .expect("equivalent reviewed source set must be valid");

    assert_eq!(semantic_then_network, network_then_semantic);
    assert_eq!(
        network_then_semantic.source_channels(),
        &[
            ExtractionSourceChannel::SemanticNode,
            ExtractionSourceChannel::NetworkResponse,
        ]
    );
}
