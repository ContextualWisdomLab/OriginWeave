use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiCommandResponseKind, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResponseCorrelationError,
    WebDriverBiDiLocateNodesResponseDocumentError, WebDriverBiDiLocateNodesResponseEnvelopeError,
    WebDriverBiDiResponseEnvelopeParseError,
};

fn locate_nodes_command(command_id: u64) -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 4)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        command_id,
        "context-a",
        &query,
    )?)
}

#[test]
fn bounded_success_document_is_parsed_and_correlated_in_one_consuming_boundary()
-> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[]}}"#,
    )?;
    let correlated = locate_nodes_command(42)?.correlate_response_document(document)?;

    assert_eq!(correlated.kind(), WebDriverBiDiCommandResponseKind::Success);
    assert_eq!(correlated.command_id(), 42);
    assert_eq!(correlated.browsing_context(), "context-a");
    Ok(())
}

#[test]
fn malformed_bounded_document_preserves_the_parser_failure() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{},}"#,
    )?;
    let result = locate_nodes_command(42)?.correlate_response_document(document);

    assert_eq!(
        result,
        Err(WebDriverBiDiLocateNodesResponseDocumentError::Parse(
            WebDriverBiDiResponseEnvelopeParseError::InvalidJson,
        ))
    );
    Ok(())
}

#[test]
fn parsed_response_id_mismatch_preserves_exact_correlation_failure() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{}}"#,
    )?;
    let result = locate_nodes_command(42)?.correlate_response_document(document);

    assert_eq!(
        result,
        Err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope(
            WebDriverBiDiLocateNodesResponseEnvelopeError::Correlation(
                WebDriverBiDiLocateNodesResponseCorrelationError::ResponseIdMismatch {
                    expected: 42,
                    actual: 41,
                },
            ),
        ))
    );
    Ok(())
}

#[test]
fn nullable_error_document_remains_explicitly_uncorrelatable() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"error","id":null,"error":"invalid argument","message":"bad request"}"#,
    )?;
    let result = locate_nodes_command(42)?.correlate_response_document(document);

    assert_eq!(
        result,
        Err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope(
            WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse,
        ))
    );
    Ok(())
}

#[test]
fn document_correlation_error_preserves_nested_error_sources() {
    let parse = WebDriverBiDiLocateNodesResponseDocumentError::Parse(
        WebDriverBiDiResponseEnvelopeParseError::InvalidJson,
    );
    assert!(parse.source().is_some());
    assert!(!parse.to_string().is_empty());

    let envelope = WebDriverBiDiLocateNodesResponseDocumentError::Envelope(
        WebDriverBiDiLocateNodesResponseEnvelopeError::MissingResponseId,
    );
    assert!(envelope.source().is_some());
    assert!(!envelope.to_string().is_empty());
}
