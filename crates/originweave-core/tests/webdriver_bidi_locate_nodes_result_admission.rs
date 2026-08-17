use std::error::Error;

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiAccessibilityQueryError,
    WebDriverBiDiCommandResponseKind, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResultAdmissionError, WebDriverBiDiRemoteNodeReferenceError,
};

fn correlated_success(
    max_node_count: u16,
) -> Result<originweave_core::ValidatedWebDriverBiDiLocateNodesResponse, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(
        Some("button"),
        Some("Submit task"),
        max_node_count,
    )?;
    Ok(WebDriverBiDiLocateNodesCommand::new(42, "context-a", &query)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, Some(42))?
        .into_validated_success()?)
}

#[test]
fn correlated_result_admission_retains_exact_command_and_normalized_nodes(
) -> Result<(), Box<dyn Error>> {
    let result = correlated_success(2)?.admit_result_nodes(&[
        ("node", Some("shared-node-a")),
        ("node", Some("shared-node-b")),
    ])?;

    assert_eq!(result.command_id(), 42);
    assert_eq!(result.browsing_context(), "context-a");
    assert_eq!(result.max_node_count(), 2);
    assert_eq!(result.nodes().len(), 2);
    assert_eq!(result.nodes()[0].remote_type(), "node");
    assert_eq!(result.nodes()[0].shared_id(), "shared-node-a");
    assert_eq!(result.nodes()[1].shared_id(), "shared-node-b");
    Ok(())
}

#[test]
fn correlated_result_admission_rejects_over_budget_batch_before_node_normalization(
) -> Result<(), Box<dyn Error>> {
    let error = correlated_success(1)?.admit_result_nodes(&[
        ("not-a-node", None),
        ("not-a-node", None),
    ]);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResultAdmissionError::Query(
            WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
        ))
    );
    Ok(())
}

#[test]
fn correlated_result_admission_rejects_invalid_remote_node_shape() -> Result<(), Box<dyn Error>> {
    let error = correlated_success(1)?.admit_result_nodes(&[("string", Some("shared-node-a"))]);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResultAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType,
        ))
    );
    Ok(())
}

#[test]
fn correlated_result_admission_error_preserves_typed_source() {
    let query_error = WebDriverBiDiLocateNodesResultAdmissionError::Query(
        WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
    );
    assert!(query_error.source().is_some());
    assert!(!query_error.to_string().is_empty());

    let remote_error = WebDriverBiDiLocateNodesResultAdmissionError::RemoteNode(
        WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
    );
    assert!(remote_error.source().is_some());
    assert!(!remote_error.to_string().is_empty());
}
