use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin,
};
use originweave_evidence::{
    CaptureManifest, CaptureManifestError, EvidenceSourceKind, MAX_CAPTURE_IDENTIFIER_BYTES,
    ProvenanceRecord, StructuredValueEvidence, VerificationResult,
};

const VALUE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const SOURCE_HASH: &str = "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const WARC_HASH: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const PROV_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";

fn structured_value() -> Result<StructuredValueEvidence, String> {
    let origin = Origin::parse("https://app.example").map_err(|error| format!("{error:?}"))?;
    let node = ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        origin,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())?;
    let node_provenance = ProvenanceRecord::new(
        "https://app.example/form",
        "ax:submit-result",
        SOURCE_HASH,
        EvidenceSourceKind::AccessibilityTree,
        VerificationResult::Verified,
    )
    .map_err(|error| format!("{error:?}"))?;
    let network_provenance = ProvenanceRecord::new(
        "https://app.example/api/task",
        "response:task-status",
        SOURCE_HASH,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .map_err(|error| format!("{error:?}"))?;

    StructuredValueEvidence::new(
        "task_status",
        VALUE_HASH,
        node,
        node_provenance,
        network_provenance,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn capture_manifest_binds_structured_result_to_durable_artifact_digests() -> Result<(), String> {
    let manifest = CaptureManifest::new(
        "capture_2026_08_21_0001",
        "task_status_v1",
        structured_value()?,
        WARC_HASH,
        PROV_HASH,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(manifest.capture_id(), "capture_2026_08_21_0001");
    assert_eq!(manifest.extraction_schema_id(), "task_status_v1");
    assert_eq!(manifest.structured_value().field_name(), "task_status");
    assert_eq!(manifest.structured_value().value_hash(), VALUE_HASH);
    assert_eq!(manifest.warc_record_set_hash(), WARC_HASH);
    assert_eq!(manifest.provenance_graph_hash(), PROV_HASH);
    Ok(())
}

#[test]
fn capture_and_schema_identifiers_fail_closed() -> Result<(), String> {
    let invalid_identifiers = ["", "---", "capture id", "capture/id"];
    for capture_id in invalid_identifiers {
        let error = CaptureManifest::new(
            capture_id,
            "task_status_v1",
            structured_value()?,
            WARC_HASH,
            PROV_HASH,
        )
        .err()
        .ok_or_else(|| format!("invalid capture id unexpectedly accepted: {capture_id:?}"))?;
        assert_eq!(error, CaptureManifestError::InvalidCaptureId);
    }

    let oversized = "a".repeat(MAX_CAPTURE_IDENTIFIER_BYTES + 1);
    let error = CaptureManifest::new(
        &oversized,
        "task_status_v1",
        structured_value()?,
        WARC_HASH,
        PROV_HASH,
    )
    .err()
    .ok_or_else(|| "oversized capture id unexpectedly accepted".to_owned())?;
    assert_eq!(error, CaptureManifestError::InvalidCaptureId);

    for schema_id in ["", "___", "task status", "task/status"] {
        let error = CaptureManifest::new(
            "capture_2026_08_21_0001",
            schema_id,
            structured_value()?,
            WARC_HASH,
            PROV_HASH,
        )
        .err()
        .ok_or_else(|| format!("invalid schema id unexpectedly accepted: {schema_id:?}"))?;
        assert_eq!(error, CaptureManifestError::InvalidExtractionSchemaId);
    }
    Ok(())
}

#[test]
fn durable_artifact_hashes_must_be_canonical_sha256() -> Result<(), String> {
    let warc_error = CaptureManifest::new(
        "capture_2026_08_21_0001",
        "task_status_v1",
        structured_value()?,
        "sha256:ABCDEF",
        PROV_HASH,
    )
    .err()
    .ok_or_else(|| "malformed WARC digest unexpectedly accepted".to_owned())?;
    assert_eq!(warc_error, CaptureManifestError::InvalidWarcRecordSetHash);

    let provenance_error = CaptureManifest::new(
        "capture_2026_08_21_0001",
        "task_status_v1",
        structured_value()?,
        WARC_HASH,
        "sha256:short",
    )
    .err()
    .ok_or_else(|| "malformed PROV digest unexpectedly accepted".to_owned())?;
    assert_eq!(
        provenance_error,
        CaptureManifestError::InvalidProvenanceGraphHash
    );
    Ok(())
}

#[test]
fn capture_manifest_errors_are_stable_and_source_free() {
    let cases = [
        (
            CaptureManifestError::InvalidCaptureId,
            "invalid capture identifier",
        ),
        (
            CaptureManifestError::InvalidExtractionSchemaId,
            "invalid extraction schema identifier",
        ),
        (
            CaptureManifestError::InvalidWarcRecordSetHash,
            "invalid WARC record-set hash",
        ),
        (
            CaptureManifestError::InvalidProvenanceGraphHash,
            "invalid provenance graph hash",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(std::error::Error::source(&error).is_none());
    }
}
