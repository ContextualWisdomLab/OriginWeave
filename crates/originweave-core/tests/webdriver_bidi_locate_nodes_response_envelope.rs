use std::error::Error;

use originweave_core::{
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiCommandResponseKind,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseCorrelationError,
    WebDriverBiDiLocateNodesResponseEnvelopeError,
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
fn success_envelope_requires_and_retains_exact_response_id() -> Result<(), Box<dyn Error>> {
    let correlated = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, Some(42))?;

    assert_eq!(correlated.kind(), WebDriverBiDiCommandResponseKind::Success);
    assert_eq!(correlated.command_id(), 42);
    assert_eq!(correlated.browsing_context(), "context-a");
    Ok(())
}

#[test]
fn correlated_success_can_be_consumed_as_success_evidence() -> Result<(), Box<dyn Error>> {
    let validated = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, Some(42))?
        .into_validated_success()?;

    assert_eq!(validated.command_id(), 42);
    assert_eq!(validated.browsing_context(), "context-a");
    Ok(())
}

#[test]
fn error_envelope_with_id_is_correlated_but_remains_error_kind() -> Result<(), Box<dyn Error>> {
    let correlated = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Error, Some(42))?;

    assert_eq!(correlated.kind(), WebDriverBiDiCommandResponseKind::Error);
    assert_eq!(correlated.command_id(), 42);
    assert_eq!(correlated.browsing_context(), "context-a");
    Ok(())
}

#[test]
fn correlated_error_cannot_become_success_evidence() -> Result<(), Box<dyn Error>> {
    let result = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Error, Some(42))?
        .into_validated_success();

    assert_eq!(
        result,
        Err(WebDriverBiDiLocateNodesResponseEnvelopeError::CorrelatedErrorResponse)
    );
    Ok(())
}

#[test]
fn success_envelope_rejects_missing_id() -> Result<(), Box<dyn Error>> {
    let error = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, None);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResponseEnvelopeError::MissingResponseId)
    );
    Ok(())
}

#[test]
fn null_error_id_is_explicitly_uncorrelatable() -> Result<(), Box<dyn Error>> {
    let error = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Error, None);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse)
    );
    Ok(())
}

#[test]
fn envelope_preserves_exact_correlation_failures() -> Result<(), Box<dyn Error>> {
    let error = locate_nodes_command(42)?
        .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, Some(41));

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResponseEnvelopeError::Correlation(
            WebDriverBiDiLocateNodesResponseCorrelationError::ResponseIdMismatch {
                expected: 42,
                actual: 41,
            }
        ))
    );
    Ok(())
}

#[test]
fn envelope_error_sources_distinguish_protocol_shape_from_correlation() {
    let direct_errors = [
        WebDriverBiDiLocateNodesResponseEnvelopeError::MissingResponseId,
        WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse,
        WebDriverBiDiLocateNodesResponseEnvelopeError::CorrelatedErrorResponse,
    ];
    for error in direct_errors {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }

    let correlation = WebDriverBiDiLocateNodesResponseEnvelopeError::Correlation(
        WebDriverBiDiLocateNodesResponseCorrelationError::InvalidResponseId,
    );
    assert!(correlation.source().is_some());
    assert!(!correlation.to_string().is_empty());
}
