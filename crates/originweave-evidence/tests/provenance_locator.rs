use originweave_evidence::{EvidenceError, EvidenceSourceKind, ProvenanceRecord, VerificationResult};

const VALID_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn provenance_rejects_control_characters_in_source_locator() {
    for source_locator in [
        "body\nforged-record",
        "body\rforged-record",
        "body\tforged-record",
        "body\u{0000}forged-record",
        "body\u{001b}[31mforged-record",
        "body\u{007f}forged-record",
    ] {
        assert_eq!(
            ProvenanceRecord::new(
                "https://example.com/item/42",
                source_locator,
                VALID_HASH,
                EvidenceSourceKind::DomTree,
                VerificationResult::Verified,
            ),
            Err(EvidenceError::InvalidLocator),
            "source_locator={source_locator:?}"
        );
    }
}

#[test]
fn provenance_locator_preserves_printable_channel_specific_syntax() {
    let source_locator = "css:main article[data-field='sale price']";
    let record = ProvenanceRecord::new(
        "https://example.com/item/42",
        source_locator,
        VALID_HASH,
        EvidenceSourceKind::DomTree,
        VerificationResult::Verified,
    )
    .expect("printable channel-specific locator");

    assert_eq!(record.source_locator(), source_locator);
}
