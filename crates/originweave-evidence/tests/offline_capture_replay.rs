#![allow(clippy::expect_used)]

use originweave_evidence::{
    CaptureManifest, CaptureManifestError, CaptureManifestValueBinding,
    CaptureManifestVerificationError, EvidenceSourceKind, ExtractionCardinality, ExtractionField,
    ExtractionSchema, ExtractionSourceChannel, ExtractionValueType, OfflineReplayVerificationError,
    ProvenanceRecord, VerificationResult, WarcProvBundle, WarcResourceRecord,
    verify_offline_capture_package,
};

const SOURCE_HASH: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALUE_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DRIFTED_VALUE_HASH: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RECORD_ID: &str = "urn:uuid:123e4567-e89b-12d3-a456-426614174000";
const DATE: &str = "2026-08-26T00:00:00Z";
const SOFTWARE_COMMIT_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn schema() -> ExtractionSchema {
    let title = ExtractionField::new(
        "title",
        ExtractionValueType::Text,
        ExtractionCardinality::One,
        true,
        &[ExtractionSourceChannel::NetworkResponse],
    )
    .expect("title field");
    ExtractionSchema::new("catalog-v3", vec![title]).expect("schema")
}

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
        b"captured-payload".to_vec(),
        provenance,
    )
    .expect("WARC record")
}

#[test]
fn offline_replay_verifies_exact_manifest_evidence_and_structured_result() {
    let schema = schema();
    let record = resource_record();
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let value =
        CaptureManifestValueBinding::new("title", VALUE_HASH, RECORD_ID).expect("value binding");
    let manifest = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&value),
    )
    .expect("manifest");
    let serialized_manifest = manifest.to_json();

    let verification = verify_offline_capture_package(
        &manifest,
        serialized_manifest.as_bytes(),
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&value),
    )
    .expect("offline replay verification");

    assert_eq!(verification.manifest_digest(), manifest.manifest_digest());
    assert_eq!(verification.record_count(), 1);
    assert_eq!(verification.value_count(), 1);
}

#[test]
fn offline_replay_rejects_persisted_manifest_byte_drift_before_evidence_replay() {
    let schema = schema();
    let record = resource_record();
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let value =
        CaptureManifestValueBinding::new("title", VALUE_HASH, RECORD_ID).expect("value binding");
    let manifest = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&value),
    )
    .expect("manifest");
    let mut serialized_manifest = manifest.to_json().into_bytes();
    serialized_manifest.push(b' ');

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            &serialized_manifest,
            &schema,
            &[(&record, &bundle)],
            std::slice::from_ref(&value),
        ),
        Err(OfflineReplayVerificationError::ManifestBytes(
            CaptureManifestVerificationError::IdentityMismatch,
        ))
    );
}

#[test]
fn offline_replay_rejects_structured_result_identity_drift() {
    let schema = schema();
    let record = resource_record();
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let value =
        CaptureManifestValueBinding::new("title", VALUE_HASH, RECORD_ID).expect("value binding");
    let drifted_value = CaptureManifestValueBinding::new("title", DRIFTED_VALUE_HASH, RECORD_ID)
        .expect("drifted value binding");
    let manifest = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&value),
    )
    .expect("manifest");
    let serialized_manifest = manifest.to_json();

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            serialized_manifest.as_bytes(),
            &schema,
            &[(&record, &bundle)],
            std::slice::from_ref(&drifted_value),
        ),
        Err(OfflineReplayVerificationError::Evidence(
            CaptureManifestVerificationError::IdentityMismatch,
        ))
    );
}

#[test]
fn offline_replay_rejects_missing_warc_evidence() {
    let schema = schema();
    let record = resource_record();
    let bundle = WarcProvBundle::new(&record, SOFTWARE_COMMIT_SHA).expect("PROV bundle");
    let value =
        CaptureManifestValueBinding::new("title", VALUE_HASH, RECORD_ID).expect("value binding");
    let manifest = CaptureManifest::new_with_warc_values(
        &schema,
        &[(&record, &bundle)],
        std::slice::from_ref(&value),
    )
    .expect("manifest");
    let serialized_manifest = manifest.to_json();

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            serialized_manifest.as_bytes(),
            &schema,
            &[],
            std::slice::from_ref(&value),
        ),
        Err(OfflineReplayVerificationError::Evidence(
            CaptureManifestVerificationError::InvalidCandidate(CaptureManifestError::MissingRecord),
        ))
    );
}

#[test]
fn offline_replay_errors_preserve_typed_diagnostics_and_sources() {
    let manifest_error = OfflineReplayVerificationError::ManifestBytes(
        CaptureManifestVerificationError::IdentityMismatch,
    );
    let evidence_error = OfflineReplayVerificationError::Evidence(
        CaptureManifestVerificationError::IdentityMismatch,
    );

    assert_eq!(
        manifest_error.to_string(),
        "offline replay persisted manifest bytes failed verification: capture manifest identity does not match"
    );
    assert_eq!(
        evidence_error.to_string(),
        "offline replay capture evidence failed verification: capture manifest identity does not match"
    );
    assert_eq!(
        std::error::Error::source(&manifest_error).map(ToString::to_string),
        Some("capture manifest identity does not match".to_owned())
    );
    assert_eq!(
        std::error::Error::source(&evidence_error).map(ToString::to_string),
        Some("capture manifest identity does not match".to_owned())
    );
}
