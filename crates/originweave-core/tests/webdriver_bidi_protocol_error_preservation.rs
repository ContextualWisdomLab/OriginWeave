use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, WebDriverBiDiAccessibilityQuery, WebDriverBiDiErrorCode,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseDocumentError,
    WebDriverBiDiLocateNodesResponseEnvelopeError,
};

#[test]
fn correlated_wire_error_remains_typed_through_node_admission() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 4)?;
    let command = WebDriverBiDiLocateNodesCommand::new(42, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"error","id":42,"error":"unavailable network data","message":"retry later"}"#,
    )?;

    assert_eq!(
        command.admit_response_document_nodes(document),
        Err(
            WebDriverBiDiLocateNodesResponseDocumentError::ProtocolError(
                WebDriverBiDiErrorCode::UnavailableNetworkData,
            )
        )
    );
    Ok(())
}

#[test]
fn nullable_wire_error_remains_uncorrelatable_before_protocol_error_admission()
-> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 4)?;
    let command = WebDriverBiDiLocateNodesCommand::new(42, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"error","id":null,"error":"invalid argument","message":"bad request"}"#,
    )?;

    assert_eq!(
        command.admit_response_document_nodes(document),
        Err(WebDriverBiDiLocateNodesResponseDocumentError::Envelope(
            WebDriverBiDiLocateNodesResponseEnvelopeError::UncorrelatableErrorResponse,
        ))
    );
    Ok(())
}
