use std::error::Error;

use originweave_core::{
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, MAX_WEBDRIVER_BIDI_COMMAND_ID,
    UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS, WEBDRIVER_BIDI_LOCATE_NODES_METHOD,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesCommandError,
};

#[test]
fn locate_nodes_command_serializes_exact_bidi_envelope() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(
        Some("textbox"),
        Some(r#"Task "quoted" \ review 작업"#),
        32,
    )?;
    let command = WebDriverBiDiLocateNodesCommand::new(42, r#"context-"quoted"\path"#, &query)?;

    assert_eq!(command.command_id(), 42);
    assert_eq!(command.method(), WEBDRIVER_BIDI_LOCATE_NODES_METHOD);
    assert_eq!(command.browsing_context(), r#"context-"quoted"\path"#);
    assert_eq!(
        command.as_json(),
        r#"{"id":42,"method":"browsingContext.locateNodes","params":{"context":"context-\"quoted\"\\path","locator":{"type":"accessibility","value":{"role":"textbox","name":"Task \"quoted\" \\ review 작업"}},"maxNodeCount":32,"serializationOptions":{"maxDomDepth":0,"maxObjectDepth":0,"includeShadowTree":"none"}}}"#
    );
    Ok(())
}

#[test]
fn locate_nodes_command_serializes_role_only_and_name_only_locators() -> Result<(), Box<dyn Error>>
{
    let role_only = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;
    let role_command = WebDriverBiDiLocateNodesCommand::new(0, "context-a", &role_only)?;
    assert_eq!(
        role_command.as_json(),
        r#"{"id":0,"method":"browsingContext.locateNodes","params":{"context":"context-a","locator":{"type":"accessibility","value":{"role":"button"}},"maxNodeCount":1,"serializationOptions":{"maxDomDepth":0,"maxObjectDepth":0,"includeShadowTree":"none"}}}"#
    );

    let name_only = WebDriverBiDiAccessibilityQuery::new(None, Some("Submit task"), 2)?;
    let name_command = WebDriverBiDiLocateNodesCommand::new(
        MAX_WEBDRIVER_BIDI_COMMAND_ID,
        "context-b",
        &name_only,
    )?;
    assert_eq!(name_command.command_id(), MAX_WEBDRIVER_BIDI_COMMAND_ID);
    assert_eq!(
        name_command.as_json(),
        r#"{"id":9007199254740991,"method":"browsingContext.locateNodes","params":{"context":"context-b","locator":{"type":"accessibility","value":{"name":"Submit task"}},"maxNodeCount":2,"serializationOptions":{"maxDomDepth":0,"maxObjectDepth":0,"includeShadowTree":"none"}}}"#
    );
    Ok(())
}

#[test]
fn locate_nodes_command_rejects_out_of_range_command_id() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        WebDriverBiDiLocateNodesCommand::new(
            MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
            "context-a",
            &query,
        ),
        Err(WebDriverBiDiLocateNodesCommandError::InvalidCommandId)
    );
    assert_eq!(
        WebDriverBiDiLocateNodesCommand::new(u64::MAX, "context-a", &query),
        Err(WebDriverBiDiLocateNodesCommandError::InvalidCommandId)
    );
    Ok(())
}

#[test]
fn locate_nodes_command_rejects_invalid_browsing_context_text() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    for invalid_context in ["", "context with space", "context\nline"] {
        assert_eq!(
            WebDriverBiDiLocateNodesCommand::new(1, invalid_context, &query),
            Err(WebDriverBiDiLocateNodesCommandError::InvalidBrowsingContext)
        );
    }

    let overlong = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1);
    assert_eq!(
        WebDriverBiDiLocateNodesCommand::new(1, &overlong, &query),
        Err(WebDriverBiDiLocateNodesCommandError::InvalidBrowsingContext)
    );

    for character in UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS {
        let context = format!("context{character}");
        assert_eq!(
            WebDriverBiDiLocateNodesCommand::new(1, &context, &query),
            Err(WebDriverBiDiLocateNodesCommandError::InvalidBrowsingContext)
        );
    }
    Ok(())
}

#[test]
fn locate_nodes_command_accepts_maximum_bounded_context() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;
    let context = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES);
    let command = WebDriverBiDiLocateNodesCommand::new(1, &context, &query)?;

    assert_eq!(command.browsing_context(), context);
    assert!(command.as_json().contains(&context));
    Ok(())
}

#[test]
fn locate_nodes_command_error_contract_is_source_free() {
    let errors = [
        WebDriverBiDiLocateNodesCommandError::InvalidCommandId,
        WebDriverBiDiLocateNodesCommandError::InvalidBrowsingContext,
    ];

    for error in errors {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
}
