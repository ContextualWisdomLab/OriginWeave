use std::{error::Error, io};

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError,
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, MAX_WEBDRIVER_BIDI_COMMAND_ID, Origin,
    OriginWeaveProtocolVersion, UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
    ValidatedBrowserProtocolUse, WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiPointerClickAuthorityError, WebDriverBiDiPointerClickCommand,
    WebDriverBiDiPointerClickCommandError, WebDriverBiDiRemoteNodeReference,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::SemanticObservation,
    )?)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn admitted_fixture(
    browsing_context: &str,
    shared_id: &str,
) -> Result<
    (
        BrowserAuthorityRegistry,
        AdmittedNodeHandle,
        WebDriverBiDiRemoteNodeReference,
    ),
    Box<dyn Error>,
> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(browser_session, browsing_context)?;
    let origin = Origin::parse("https://app.example").map_err(|error| {
        io::Error::other(format!("fixture origin rejected unexpectedly: {error:?}"))
    })?;
    let epoch = registry.bind_context_origin(browser_session, context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 1)?;
    let locate = WebDriverBiDiLocateNodesCommand::new(41, browsing_context, &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(&format!(
        r#"{{"type":"success","id":41,"result":{{"nodes":[{{"type":"node","sharedId":"{}"}}]}}}}"#,
        json_escape(shared_id)
    ))?;
    let handle = locate
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("locateNodes fixture did not bind its node"))?;
    let remote = WebDriverBiDiRemoteNodeReference::new("node", Some(shared_id))?;
    Ok((registry, handle, remote))
}

#[test]
fn pointer_click_command_serializes_exact_bidi_envelope() -> Result<(), Box<dyn Error>> {
    let (registry, handle, node) = admitted_fixture("context-a", "shared-node-42")?;
    let command = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &handle,
        &node,
        &registry,
    )?;

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
    let (registry, handle, node) = admitted_fixture("context-a", "shared-node-42")?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
            "context-a",
            &handle,
            &node,
            &registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::Command(
            WebDriverBiDiPointerClickCommandError::InvalidCommandId
        ))
    );

    for invalid in ["", "context with space", "context\nline"] {
        assert_eq!(
            WebDriverBiDiPointerClickCommand::new_for_current_node(
                1, invalid, &handle, &node, &registry,
            ),
            Err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
                BrowserRegistryError::InvalidExternalIdentifier,
            ))
        );
    }

    let overlong = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1);
    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            1, &overlong, &handle, &node, &registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
            BrowserRegistryError::InvalidExternalIdentifier,
        ))
    );
    for character in UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS {
        let context = format!("context{character}");
        assert_eq!(
            WebDriverBiDiPointerClickCommand::new_for_current_node(
                1, &context, &handle, &node, &registry,
            ),
            Err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
                BrowserRegistryError::InvalidExternalIdentifier,
            ))
        );
    }
    Ok(())
}

#[test]
fn pointer_click_command_accepts_maximum_context_and_escaped_shared_id()
-> Result<(), Box<dyn Error>> {
    let context = "c".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES);
    let shared_id = r#"node-"quoted"\path"#;
    let (registry, handle, node) = admitted_fixture(&context, shared_id)?;
    let command = WebDriverBiDiPointerClickCommand::new_for_current_node(
        MAX_WEBDRIVER_BIDI_COMMAND_ID,
        &context,
        &handle,
        &node,
        &registry,
    )?;

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
