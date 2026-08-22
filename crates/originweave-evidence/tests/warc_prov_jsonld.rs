#![allow(clippy::expect_used)]

use originweave_evidence::{
    EvidenceSourceKind, MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES, ProvenanceRecord, VerificationResult,
    WarcProvBundle, WarcProvBundleError, WarcResourceRecord,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-22T12:00:00Z";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn resource_record() -> WarcResourceRecord {
    let provenance = ProvenanceRecord::new(
        "https://example.com/item",
        "body",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        "https://example.com/item",
        "text/plain",
        b"hello".to_vec(),
        provenance,
    )
    .expect("WARC resource record")
}

fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn warc_prov_bundle_exposes_stable_capture_identities_and_standard_errors() {
    assert_standard_error_contract::<WarcProvBundleError>();
    assert_eq!(
        WarcProvBundleError::InvalidSoftwareCommitSha.to_string(),
        "invalid OriginWeave software commit SHA"
    );
    assert_eq!(
        WarcProvBundleError::LimitExceeded.to_string(),
        "WARC PROV bundle limit exceeded"
    );

    let bundle = WarcProvBundle::new(&resource_record(), SOFTWARE_COMMIT_SHA)
        .expect("PROV bundle over a validated WARC record");
    assert_eq!(bundle.record_entity_id(), RECORD_ID);
    assert_eq!(
        bundle.source_entity_id(),
        "urn:uuid:123e4567-e89b-12d3-a456-426614174000#source"
    );
    assert_eq!(
        bundle.capture_activity_id(),
        "urn:uuid:123e4567-e89b-12d3-a456-426614174000#capture"
    );
    assert_eq!(
        bundle.software_agent_id(),
        "https://github.com/ContextualWisdomLab/OriginWeave/commit/0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(bundle.software_commit_sha(), SOFTWARE_COMMIT_SHA);
}

#[test]
fn warc_prov_bundle_emits_deterministic_prov_o_json_ld_without_raw_payload() {
    let bundle = WarcProvBundle::new(&resource_record(), SOFTWARE_COMMIT_SHA)
        .expect("PROV bundle over a validated WARC record");
    let json_ld = bundle.to_json_ld();

    assert_eq!(
        json_ld,
        concat!(
            "{\"@context\":{\"prov\":\"http://www.w3.org/ns/prov#\",\"xsd\":\"http://www.w3.org/2001/XMLSchema#\"},\"@graph\":[",
            "{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000#source\",\"@type\":\"prov:Entity\",\"prov:atLocation\":{\"@id\":\"https://example.com/item\"},\"prov:value\":\"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"},",
            "{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000#capture\",\"@type\":\"prov:Activity\",\"prov:startedAtTime\":{\"@value\":\"2026-08-22T12:00:00Z\",\"@type\":\"xsd:dateTime\"},\"prov:used\":{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000#source\"},\"prov:wasAssociatedWith\":{\"@id\":\"https://github.com/ContextualWisdomLab/OriginWeave/commit/0123456789abcdef0123456789abcdef01234567\"}},",
            "{\"@id\":\"https://github.com/ContextualWisdomLab/OriginWeave/commit/0123456789abcdef0123456789abcdef01234567\",\"@type\":\"prov:SoftwareAgent\"},",
            "{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000\",\"@type\":\"prov:Entity\",\"prov:value\":\"sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\",\"prov:wasDerivedFrom\":{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000#source\"},\"prov:wasGeneratedBy\":{\"@id\":\"urn:uuid:123e4567-e89b-12d3-a456-426614174000#capture\"}}",
            "]}"
        )
    );
    assert!(!json_ld.contains("hello"));
}

#[test]
fn warc_prov_bundle_rejects_noncanonical_or_oversized_software_revisions() {
    let record = resource_record();
    for software_commit_sha in [
        "",
        "0123456789abcdef0123456789abcdef0123456",
        "0123456789abcdef0123456789abcdef0123456G",
        "0123456789ABCDEF0123456789ABCDEF01234567",
        "0123456789abcdef0123456789abcdef0123456 ",
    ] {
        assert_eq!(
            WarcProvBundle::new(&record, software_commit_sha),
            Err(WarcProvBundleError::InvalidSoftwareCommitSha),
            "software_commit_sha={software_commit_sha:?}"
        );
    }

    assert_eq!(
        MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES,
        SOFTWARE_COMMIT_SHA.len()
    );
    assert_eq!(
        WarcProvBundle::new(&record, &"a".repeat(MAX_PROV_SOFTWARE_COMMIT_SHA_BYTES + 1)),
        Err(WarcProvBundleError::LimitExceeded)
    );
}
