use std::error::Error;

use originweave_core::{
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, MAX_WEBDRIVER_BIDI_COMMAND_ID,
    UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS, WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD,
    WebDriverBiDiPointerClickCommand, WebDriverBiDiPointerClickCommandError,
    WebDriverBiDiRemoteNodeReference,
};

#[test]
fn pointer_click_command_serializes_exact_bidi_envelope() -> Result<(), Box<dyn Error>> {
    let node = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;
    let command = WebDriverBiDiPointerClickCommand::new(42, "context-a", &node)?;

    assert_eq!(command.command_id(), 42);
    assert_eq!(command.method(), WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD);
    assert_eq!(command.browsing_context(), "context-a");
    assert_eq!(
        command.as_json(),
        r#"{"id":42,"method":"input.performActions","params":{"context":"context-a","actions":[{"type":"pointer","id":"originweave-mouse","parameters":{"pointerType":"mouse"},"actions":[{"type":"pointerMove","x":0,"y":0,"origin":{"type":"element","element":{"sharedId":"shared-node-42"}}},{"type":"pointerDown","button":0},{"type":"pointerUp","button":0}]}]}}"#
    );
    Ok(())
}

#[test]
fn pointer_click_command_rejects_invalid_command_and_context() -> Result<(), Box<dyn Error>> {
    let node = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new(
            MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
            "context-a",
            &node,
        ),
        Err(WebDriverBiDiPointerClickCommandError::InvalidCommandId)
    );

    for invalid in ["", "context with space", "context\nline"] {
        assert_eq!(
            WebDriverBiDiPointerClickCommand::new(1, invalid, &node),
            Err(WebDriverBiDiPointerClickCommandError::InvalidBrowsingContext)
        );
    }

    let overlong = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1);
    assert_eq!(
        WebDriverBiDiPointerClickCommand::new(1, &overlong, &node),
        Err(WebDriverBiDiPointerClickCommandError::InvalidBrowsingContext)
    );
    for character in UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS {
        let context = format!("context{character}");
        assert_eq!(
            WebDriverBiDiPointerClickCommand::new(1, &context, &node),
            Err(WebDriverBiDiPointerClickCommandError::InvalidBrowsingContext)
        );
    }
    Ok(())
}

#[test]
fn pointer_click_command_accepts_maximum_context_and_escaped_shared_id()
-> Result<(), Box<dyn Error>> {
    let context = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES);
    let node = WebDriverBiDiRemoteNodeReference::new("node", Some(r#"node-"quoted"\path"#))?;
    let command =
        WebDriverBiDiPointerClickCommand::new(MAX_WEBDRIVER_BIDI_COMMAND_ID, &context, &node)?;

    assert!(command.as_json().contains(&context));
    assert!(command.as_json().contains(r#"node-\"quoted\"\\path"#));
    Ok(())
}

#[test]
fn pointer_click_command_error_contract_is_source_free() {
    for error in [
        WebDriverBiDiPointerClickCommandError::InvalidCommandId,
        WebDriverBiDiPointerClickCommandError::InvalidBrowsingContext,
    ] {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
}
