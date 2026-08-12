use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin,
};
use originweave_evidence::{
    EvidenceSourceKind, MAX_STRUCTURED_FIELD_NAME_BYTES, ProvenanceRecord,
    StructuredValueEvidence, StructuredValueEvidenceError, VerificationResult,
};

const VALID_VALUE_HASH: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALID_SOURCE_HASH: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn node(source_origin: &Origin) -> Result<ObservedNodeHandle, String> {
    ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        source_origin.clone(),
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())
}

fn provenance(
    source_url: &str,
    locator: &str,
    source_kind: EvidenceSourceKind,
    verification_result: VerificationResult,
) -> Result<ProvenanceRecord, String> {
    ProvenanceRecord::new(
        source_url,
        locator,
        VALID_SOURCE_HASH,
        source_kind,
        verification_result,
    )
    .map_err(|error| format!("{error:?}"))
}

fn valid_evidence() -> Result<StructuredValueEvidence, String> {
    let source_origin = origin("https://app.example")?;
    StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        node(&source_origin)?,
        provenance(
            "https://app.example/form",
            "ax:submit-result",
            EvidenceSourceKind::AccessibilityTree,
            VerificationResult::Verified,
        )?,
        provenance(
            "https://app.example/api/task",
            "response:task-status",
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Verified,
        )?,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn structured_value_binds_digest_exact_node_and_verified_network_source() -> Result<(), String> {
    let evidence = valid_evidence()?;

    assert_eq!(evidence.field_name(), "task_status");
    assert_eq!(evidence.value_hash(), VALID_VALUE_HASH);
    assert_eq!(evidence.source_node().node_id(), 17);
    assert_eq!(
        evidence.node_provenance().source_kind(),
        EvidenceSourceKind::AccessibilityTree
    );
    assert_eq!(
        evidence.network_provenance().source_kind(),
        EvidenceSourceKind::NetworkResponse
    );
    assert_eq!(
        evidence.node_provenance().verification_result(),
        VerificationResult::Verified
    );
    assert_eq!(
        evidence.network_provenance().verification_result(),
        VerificationResult::Verified
    );
    Ok(())
}

#[test]
fn malformed_field_identifiers_fail_closed() -> Result<(), String> {
    let source_origin = origin("https://app.example")?;
    let node_source = provenance(
        "https://app.example/form",
        "dom:#status",
        EvidenceSourceKind::DomTree,
        VerificationResult::Verified,
    )?;
    let network_source = provenance(
        "https://app.example/api/task",
        "response:task-status",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )?;

    for field_name in ["", "task status", "task/status"] {
        let error = StructuredValueEvidence::new(
            field_name,
            VALID_VALUE_HASH,
            node(&source_origin)?,
            node_source.clone(),
            network_source.clone(),
        )
        .err()
        .ok_or_else(|| format!("invalid field name unexpectedly accepted: {field_name:?}"))?;
        assert_eq!(error, StructuredValueEvidenceError::InvalidFieldName);
    }

    let oversized = "a".repeat(MAX_STRUCTURED_FIELD_NAME_BYTES + 1);
    let error = StructuredValueEvidence::new(
        &oversized,
        VALID_VALUE_HASH,
        node(&source_origin)?,
        node_source,
        network_source,
    )
    .err()
    .ok_or_else(|| "oversized field name unexpectedly accepted".to_owned())?;
    assert_eq!(error, StructuredValueEvidenceError::InvalidFieldName);
    Ok(())
}

#[test]
fn malformed_value_digest_fails_closed() -> Result<(), String> {
    let source_origin = origin("https://app.example")?;
    let error = StructuredValueEvidence::new(
        "task_status",
        "sha256:ABCDEF",
        node(&source_origin)?,
        provenance(
            "https://app.example/form",
            "dom:#status",
            EvidenceSourceKind::DomTree,
            VerificationResult::Verified,
        )?,
        provenance(
            "https://app.example/api/task",
            "response:task-status",
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Verified,
        )?,
    )
    .err()
    .ok_or_else(|| "malformed value digest unexpectedly accepted".to_owned())?;

    assert_eq!(error, StructuredValueEvidenceError::InvalidValueHash);
    Ok(())
}

#[test]
fn node_provenance_must_be_verified_node_evidence() -> Result<(), String> {
    let source_origin = origin("https://app.example")?;
    let network_source = provenance(
        "https://app.example/api/task",
        "response:task-status",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )?;

    let unverified_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        node(&source_origin)?,
        provenance(
            "https://app.example/form",
            "ax:submit-result",
            EvidenceSourceKind::AccessibilityTree,
            VerificationResult::Unverified,
        )?,
        network_source.clone(),
    )
    .err()
    .ok_or_else(|| "unverified node provenance unexpectedly accepted".to_owned())?;
    assert_eq!(
        unverified_error,
        StructuredValueEvidenceError::NodeProvenanceNotVerified
    );

    let wrong_kind_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        node(&source_origin)?,
        provenance(
            "https://app.example/api/task",
            "response:other",
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Verified,
        )?,
        network_source,
    )
    .err()
    .ok_or_else(|| "network evidence unexpectedly accepted as node provenance".to_owned())?;
    assert_eq!(
        wrong_kind_error,
        StructuredValueEvidenceError::NodeProvenanceKindMismatch
    );
    Ok(())
}

#[test]
fn network_provenance_must_be_verified_network_evidence() -> Result<(), String> {
    let source_origin = origin("https://app.example")?;
    let node_source = provenance(
        "https://app.example/form",
        "dom:#status",
        EvidenceSourceKind::DomTree,
        VerificationResult::Verified,
    )?;

    let unverified_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        node(&source_origin)?,
        node_source.clone(),
        provenance(
            "https://app.example/api/task",
            "response:task-status",
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Rejected,
        )?,
    )
    .err()
    .ok_or_else(|| "rejected network provenance unexpectedly accepted".to_owned())?;
    assert_eq!(
        unverified_error,
        StructuredValueEvidenceError::NetworkProvenanceNotVerified
    );

    let wrong_kind_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        node(&source_origin)?,
        node_source.clone(),
        provenance(
            "https://app.example/form",
            "dom:#network-lookalike",
            EvidenceSourceKind::DomTree,
            VerificationResult::Verified,
        )?,
    )
    .err()
    .ok_or_else(|| "DOM evidence unexpectedly accepted as network provenance".to_owned())?;
    assert_eq!(
        wrong_kind_error,
        StructuredValueEvidenceError::NetworkProvenanceKindMismatch
    );
    Ok(())
}

#[test]
fn provenance_from_another_origin_cannot_prove_the_node_value() -> Result<(), String> {
    let source_origin = origin("https://app.example")?;
    let source_node = node(&source_origin)?;
    let valid_node_source = provenance(
        "https://app.example/form",
        "dom:#status",
        EvidenceSourceKind::DomTree,
        VerificationResult::Verified,
    )?;
    let valid_network_source = provenance(
        "https://app.example/api/task",
        "response:task-status",
        EvidenceSourceKind::NetworkResponse,
        VerificationResult::Verified,
    )?;

    let node_origin_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        source_node.clone(),
        provenance(
            "https://other.example/form",
            "dom:#status",
            EvidenceSourceKind::DomTree,
            VerificationResult::Verified,
        )?,
        valid_network_source.clone(),
    )
    .err()
    .ok_or_else(|| "cross-origin node provenance unexpectedly accepted".to_owned())?;
    assert_eq!(
        node_origin_error,
        StructuredValueEvidenceError::SourceOriginMismatch
    );

    let network_origin_error = StructuredValueEvidence::new(
        "task_status",
        VALID_VALUE_HASH,
        source_node,
        valid_node_source,
        provenance(
            "https://other.example/api/task",
            "response:task-status",
            EvidenceSourceKind::NetworkResponse,
            VerificationResult::Verified,
        )?,
    )
    .err()
    .ok_or_else(|| "cross-origin network provenance unexpectedly accepted".to_owned())?;
    assert_eq!(
        network_origin_error,
        StructuredValueEvidenceError::SourceOriginMismatch
    );
    Ok(())
}
