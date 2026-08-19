use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, WebDriverBiDiAccessibilityQuery, WebDriverBiDiErrorCode,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseDocumentError,
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
