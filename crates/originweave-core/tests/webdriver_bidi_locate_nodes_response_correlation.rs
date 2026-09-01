use std::error::Error;

use originweave_core::{
    MAX_WEBDRIVER_BIDI_COMMAND_ID, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseCorrelationError,
};

fn locate_nodes_command(
    command_id: u64,
) -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 4)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        command_id,
        "context-a",
        &query,
    )?)
}

#[test]
fn locate_nodes_response_requires_exact_command_id() -> Result<(), Box<dyn Error>> {
    let correlated = locate_nodes_command(42)?.correlate_response_id(42)?;

    assert_eq!(correlated.command_id(), 42);
    assert_eq!(correlated.browsing_context(), "context-a");
    Ok(())
}

#[test]
fn locate_nodes_response_rejects_mismatched_command_id() -> Result<(), Box<dyn Error>> {
    let error = locate_nodes_command(42)?.correlate_response_id(41);

    assert_eq!(
        error,
        Err(
            WebDriverBiDiLocateNodesResponseCorrelationError::ResponseIdMismatch {
                expected: 42,
                actual: 41,
            }
        )
    );
    Ok(())
}

#[test]
fn locate_nodes_response_rejects_out_of_range_id_before_correlation() -> Result<(), Box<dyn Error>>
{
    let error = locate_nodes_command(1)?.correlate_response_id(MAX_WEBDRIVER_BIDI_COMMAND_ID + 1);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResponseCorrelationError::InvalidResponseId)
    );
    Ok(())
}

#[test]
fn response_correlation_error_contract_is_source_free() {
    let errors = [
        WebDriverBiDiLocateNodesResponseCorrelationError::InvalidResponseId,
        WebDriverBiDiLocateNodesResponseCorrelationError::ResponseIdMismatch {
            expected: 2,
            actual: 1,
        },
    ];

    for error in errors {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
}
