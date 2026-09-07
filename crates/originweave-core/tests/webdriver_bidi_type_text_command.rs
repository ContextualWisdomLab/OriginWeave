use std::{error::Error, io};

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, MAX_WEBDRIVER_BIDI_COMMAND_ID, MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES,
    NodeHandleError, Origin, OriginWeaveProtocolVersion, UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
    ValidatedBrowserProtocolUse, WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiTypeTextAuthorityError,
    WebDriverBiDiTypeTextCommand, WebDriverBiDiTypeTextCommandError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

struct AdmittedTextField {
    registry: BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    external_context: String,
    handle: AdmittedNodeHandle,
    remote: WebDriverBiDiRemoteNodeReference,
}

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

fn admitted_text_field(
    external_context: &str,
    shared_id: &str,
) -> Result<AdmittedTextField, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let browsing_context = registry.register_context(browser_session, external_context)?;
    let origin = Origin::parse("https://app.example").map_err(|error| {
        io::Error::other(format!("fixture origin rejected unexpectedly: {error:?}"))
    })?;
    let epoch = registry.bind_context_origin(browser_session, browsing_context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, browsing_context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task name"), 1)?;
    let locate = WebDriverBiDiLocateNodesCommand::new(41, external_context, &query)?;
    let escaped_shared_id = shared_id.replace('\\', "\\\\").replace('"', "\\\"");
    let document = BoundedWebDriverBiDiResponseDocument::new(&format!(
        r#"{{"type":"success","id":41,"result":{{"nodes":[{{"type":"node","sharedId":"{escaped_shared_id}"}}]}}}}"#,
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
    Ok(AdmittedTextField {
        registry,
        browser_session,
        browsing_context,
        external_context: external_context.to_owned(),
        handle,
        remote,
    })
}

#[test]
fn type_text_command_focuses_exact_admitted_node_before_keyboard_input()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let command = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        &fixture.external_context,
        "Az",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    assert_eq!(command.command_id(), 42);
    assert_eq!(command.method(), WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD);
    assert_eq!(command.browsing_context(), "context-a");
    assert_eq!(command.text_bytes(), 2);
    assert_eq!(
        command.as_json(),
        r#"{"id":42,"method":"input.performActions","params":{"context":"context-a","actions":[{"type":"pointer","id":"originweave-mouse","parameters":{"pointerType":"mouse"},"actions":[{"type":"pointerMove","x":0,"y":0,"origin":{"type":"element","element":{"sharedId":"shared-input-42"}}},{"type":"pointerDown","button":0},{"type":"pointerUp","button":0},{"type":"pause"},{"type":"pause"},{"type":"pause"},{"type":"pause"}]},{"type":"key","id":"originweave-keyboard","actions":[{"type":"pause"},{"type":"pause"},{"type":"pause"},{"type":"keyDown","value":"A"},{"type":"keyUp","value":"A"},{"type":"keyDown","value":"z"},{"type":"keyUp","value":"z"}]}]}}"#
    );
    Ok(())
}

#[test]
fn type_text_command_escapes_protocol_identifiers_and_keyboard_characters()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field(r#"context-"quoted"\path"#, r#"node-"quoted"\path"#)?;
    let command = WebDriverBiDiTypeTextCommand::new_for_current_node(
        MAX_WEBDRIVER_BIDI_COMMAND_ID,
        &fixture.external_context,
        r#""\é"#,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    assert!(command.as_json().contains(r#"context-\"quoted\"\\path"#));
    assert!(command.as_json().contains(r#"node-\"quoted\"\\path"#));
    assert!(command.as_json().contains(r#""value":"\"""#));
    assert!(command.as_json().contains(r#""value":"\\""#));
    assert!(command.as_json().contains(r#""value":"é""#));
    assert_eq!(command.text_bytes(), 4);
    Ok(())
}

#[test]
fn type_text_command_rejects_unbounded_or_protocol_dangerous_text() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let overlong = "a".repeat(MAX_WEBDRIVER_BIDI_TYPE_TEXT_BYTES + 1);
    let format_text = format!("a{}b", UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS[0]);

    for (text, expected) in [
        ("", WebDriverBiDiTypeTextCommandError::EmptyText),
        (
            overlong.as_str(),
            WebDriverBiDiTypeTextCommandError::TextTooLong,
        ),
        (
            "line\nbreak",
            WebDriverBiDiTypeTextCommandError::InvalidText,
        ),
        (
            format_text.as_str(),
            WebDriverBiDiTypeTextCommandError::InvalidText,
        ),
    ] {
        assert_eq!(
            WebDriverBiDiTypeTextCommand::new_for_current_node(
                42,
                &fixture.external_context,
                text,
                &fixture.handle,
                &fixture.remote,
                &fixture.registry,
            ),
            Err(WebDriverBiDiTypeTextAuthorityError::Command(expected))
        );
    }

    assert_eq!(
        WebDriverBiDiTypeTextCommand::new_for_current_node(
            MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
            &fixture.external_context,
            "a",
            &fixture.handle,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiTypeTextAuthorityError::Command(
            WebDriverBiDiTypeTextCommandError::InvalidCommandId,
        ))
    );
    Ok(())
}

#[test]
fn type_text_command_rejects_wrong_context_and_unadmitted_node() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let wrong_context = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        "context-b",
        "safe text",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected wrong external context rejection")?;
    assert_eq!(
        wrong_context,
        WebDriverBiDiTypeTextAuthorityError::BrowserAuthority(
            BrowserRegistryError::ContextExternalIdentifierMismatch,
        )
    );
    assert!(wrong_context.source().is_some());
    assert!(wrong_context.to_string().contains("browser authority"));

    let forged = WebDriverBiDiRemoteNodeReference::new("node", Some("unadmitted-node"))?;
    let wrong_node = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        &fixture.external_context,
        "safe text",
        &fixture.handle,
        &forged,
        &fixture.registry,
    )
    .err()
    .ok_or("expected unadmitted sharedId rejection")?;
    assert_eq!(
        wrong_node,
        WebDriverBiDiTypeTextAuthorityError::NodeExternalIdentifierMismatch
    );
    assert!(wrong_node.source().is_none());
    assert!(wrong_node.to_string().contains("wire node identifier"));
    Ok(())
}

#[test]
fn type_text_command_rejects_changed_origin_authority() -> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_text_field("context-a", "shared-input-42")?;
    fixture
        .registry
        .advance_document(fixture.browsing_context)?;
    let changed_origin = Origin::parse("https://changed.example").map_err(|error| {
        io::Error::other(format!(
            "changed fixture origin rejected unexpectedly: {error:?}"
        ))
    })?;
    fixture.registry.bind_context_origin(
        fixture.browser_session,
        fixture.browsing_context,
        &changed_origin,
    )?;

    let error = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        &fixture.external_context,
        "safe text",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected changed-origin rejection")?;
    assert!(matches!(
        error,
        WebDriverBiDiTypeTextAuthorityError::BrowserAuthority(_)
    ));
    assert!(error.source().is_some());
    assert!(error.to_string().contains("browser authority"));
    Ok(())
}

#[test]
fn type_text_command_rejects_stale_document_authority() -> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_text_field("context-a", "shared-input-42")?;
    let observed = fixture.handle.document_epoch();
    let current = fixture
        .registry
        .advance_document(fixture.browsing_context)?;
    let origin = fixture.handle.origin().clone();
    fixture.registry.bind_context_origin(
        fixture.browser_session,
        fixture.browsing_context,
        &origin,
    )?;

    let error = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        &fixture.external_context,
        "safe text",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected stale document rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiTypeTextAuthorityError::NodeHandle(NodeHandleError::StaleDocumentEpoch {
            observed,
            current,
        })
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("node authority"));
    Ok(())
}

#[test]
fn type_text_error_contracts_expose_only_typed_sources() {
    for error in [
        WebDriverBiDiTypeTextCommandError::InvalidCommandId,
        WebDriverBiDiTypeTextCommandError::EmptyText,
        WebDriverBiDiTypeTextCommandError::TextTooLong,
        WebDriverBiDiTypeTextCommandError::InvalidText,
    ] {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
        let authority = WebDriverBiDiTypeTextAuthorityError::Command(error);
        assert!(authority.source().is_some());
        assert!(!authority.to_string().is_empty());
    }
}

#[test]
fn type_text_command_debug_redacts_typed_text_and_wire_payload() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let command = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        &fixture.external_context,
        "buyer-private-marker",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    let debug = format!("{command:?}");
    assert!(!debug.contains("buyer-private-marker"));
    assert!(!debug.contains("originweave-keyboard"));
    assert!(debug.contains("command_id: 42"));
    assert!(debug.contains("text_bytes: 20"));
    Ok(())
}
