use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand,
};

fn locate_nodes_command(
    command_id: u64,
    max_node_count: u16,
) -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query =
        WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), max_node_count)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        command_id,
        "context-a",
        &query,
    )?)
}

#[test]
fn bounded_locate_nodes_document_admits_exact_wire_nodes_without_caller_selected_payload()
-> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node","sharedId":"node-a"},{"type":"node","sharedId":"node-b","value":{"nodeType":1}}]}}"#,
    )?;
    let admitted = locate_nodes_command(42, 2)?.admit_response_document_nodes(document)?;

    assert_eq!(admitted.command_id(), 42);
    assert_eq!(admitted.browsing_context(), "context-a");
    assert_eq!(admitted.max_node_count(), 2);
    assert_eq!(admitted.nodes().len(), 2);
    assert_eq!(admitted.nodes()[0].remote_type(), "node");
    assert_eq!(admitted.nodes()[0].shared_id(), "node-a");
    assert_eq!(admitted.nodes()[1].shared_id(), "node-b");
    Ok(())
}

#[test]
fn wire_result_preserves_exact_command_node_budget() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node","sharedId":"node-a"},{"type":"node","sharedId":"node-b"}]}}"#,
    )?;

    assert!(
        locate_nodes_command(42, 1)?
            .admit_response_document_nodes(document)
            .is_err()
    );
    Ok(())
}

#[test]
fn wire_result_rejects_missing_shared_id() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node"}]}}"#,
    )?;

    assert!(
        locate_nodes_command(42, 1)?
            .admit_response_document_nodes(document)
            .is_err()
    );
    Ok(())
}

#[test]
fn wire_result_rejects_non_node_remote_value() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"window","sharedId":"node-a"}]}}"#,
    )?;

    assert!(
        locate_nodes_command(42, 1)?
            .admit_response_document_nodes(document)
            .is_err()
    );
    Ok(())
}

#[test]
fn wire_result_requires_nodes_array() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":42,"result":{}}"#,
        r#"{"type":"success","id":42,"result":{"nodes":{}}}"#,
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
        assert!(
            locate_nodes_command(42, 1)?
                .admit_response_document_nodes(document)
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn wire_result_rejects_ambiguous_duplicate_result_or_node_fields() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":42,"result":{"nodes":[],"nodes":[]}}"#,
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node","type":"node","sharedId":"node-a"}]}}"#,
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node","sharedId":"node-a","sharedId":"node-a"}]}}"#,
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
        assert!(
            locate_nodes_command(42, 1)?
                .admit_response_document_nodes(document)
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn wire_result_decodes_json_escaped_protocol_fields_before_admission() -> Result<(), Box<dyn Error>>
{
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"res\u0075lt":{"no\u0064es":[{"ty\u0070e":"no\u0064e","shared\u0049d":"node-\u03b1"}]}}"#,
    )?;
    let admitted = locate_nodes_command(42, 1)?.admit_response_document_nodes(document)?;

    assert_eq!(admitted.nodes().len(), 1);
    assert_eq!(admitted.nodes()[0].shared_id(), "node-α");
    Ok(())
}
