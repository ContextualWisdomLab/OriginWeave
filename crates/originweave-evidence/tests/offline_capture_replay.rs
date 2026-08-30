#![allow(clippy::expect_used)]

use originweave_evidence::{
    CaptureManifest, CaptureManifestError, CaptureManifestValueBinding,
    CaptureManifestVerificationError, EvidenceSourceKind, ExtractionCardinality, ExtractionField,
    ExtractionSchema, ExtractionSourceChannel, ExtractionValueType, OfflineReplayVerificationError,
    ProvenanceRecord, VerificationResult, WarcProvBundle, WarcResourceRecord,
    verify_offline_capture_package,
};
use sha2::{Digest, Sha256};

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
    let payload = b"captured-payload";
    let source_hash = format!("sha256:{:x}", Sha256::digest(payload));
    let provenance = ProvenanceRecord::new(
        "https://example.com/item",
        "body",
        &source_hash,
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )
    .expect("verified provenance");
    WarcResourceRecord::new(
        RECORD_ID,
        DATE,
        "https://example.com/item",
        "text/plain",
        payload.to_vec(),
        provenance,
    )
    .expect("WARC record")
}

#[test]
fn offline_replay_verifies_exact_persisted_manifest_warc_prov_and_structured_result() {
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
    let persisted_warc = record.to_warc_bytes();
    let persisted_prov = bundle.to_json_ld();

    let verification = verify_offline_capture_package(
        &manifest,
        serialized_manifest.as_bytes(),
        &schema,
        &[(&record, &bundle)],
        &[(&persisted_warc, persisted_prov.as_bytes())],
        std::slice::from_ref(&value),
    )
    .expect("offline replay verification");

    assert_eq!(verification.manifest_digest(), manifest.manifest_digest());
    assert_eq!(verification.record_count(), 1);
    assert_eq!(verification.value_count(), 1);
}

#[test]
fn offline_replay_rejects_persisted_warc_byte_drift() {
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
    let mut persisted_warc = record.to_warc_bytes();
    persisted_warc.push(b' ');
    let persisted_prov = bundle.to_json_ld();

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            serialized_manifest.as_bytes(),
            &schema,
            &[(&record, &bundle)],
            &[(&persisted_warc, persisted_prov.as_bytes())],
            std::slice::from_ref(&value),
        ),
        Err(OfflineReplayVerificationError::WarcBytes { record_index: 0 })
    );
}

#[test]
fn offline_replay_rejects_persisted_prov_byte_drift() {
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
    let persisted_warc = record.to_warc_bytes();
    let mut persisted_prov = bundle.to_json_ld().into_bytes();
    persisted_prov.push(b' ');

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            serialized_manifest.as_bytes(),
            &schema,
            &[(&record, &bundle)],
            &[(&persisted_warc, &persisted_prov)],
            std::slice::from_ref(&value),
        ),
        Err(OfflineReplayVerificationError::ProvBytes { record_index: 0 })
    );
}

#[test]
fn offline_replay_rejects_persisted_record_count_mismatch() {
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
            &[(&record, &bundle)],
            &[],
            std::slice::from_ref(&value),
        ),
        Err(OfflineReplayVerificationError::PersistedRecordCountMismatch)
    );
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
    let persisted_warc = record.to_warc_bytes();
    let persisted_prov = bundle.to_json_ld();

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            &serialized_manifest,
            &schema,
            &[(&record, &bundle)],
            &[(&persisted_warc, persisted_prov.as_bytes())],
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
    let persisted_warc = record.to_warc_bytes();
    let persisted_prov = bundle.to_json_ld();

    assert_eq!(
        verify_offline_capture_package(
            &manifest,
            serialized_manifest.as_bytes(),
            &schema,
            &[(&record, &bundle)],
            &[(&persisted_warc, persisted_prov.as_bytes())],
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
    let count_error = OfflineReplayVerificationError::PersistedRecordCountMismatch;
    let warc_error = OfflineReplayVerificationError::WarcBytes { record_index: 2 };
    let prov_error = OfflineReplayVerificationError::ProvBytes { record_index: 3 };

    assert_eq!(
        manifest_error.to_string(),
        "offline replay persisted manifest bytes failed verification: capture manifest identity does not match"
    );
    assert_eq!(
        evidence_error.to_string(),
        "offline replay capture evidence failed verification: capture manifest identity does not match"
    );
    assert_eq!(
        count_error.to_string(),
        "offline replay persisted WARC/PROV record count does not match typed evidence"
    );
    assert_eq!(
        warc_error.to_string(),
        "offline replay persisted WARC bytes failed verification at record 2"
    );
    assert_eq!(
        prov_error.to_string(),
        "offline replay persisted PROV bytes failed verification at record 3"
    );
    assert_eq!(
        std::error::Error::source(&manifest_error).map(ToString::to_string),
        Some("capture manifest identity does not match".to_owned())
    );
    assert_eq!(
        std::error::Error::source(&evidence_error).map(ToString::to_string),
        Some("capture manifest identity does not match".to_owned())
    );
    assert!(std::error::Error::source(&count_error).is_none());
    assert!(std::error::Error::source(&warc_error).is_none());
    assert!(std::error::Error::source(&prov_error).is_none());
}
