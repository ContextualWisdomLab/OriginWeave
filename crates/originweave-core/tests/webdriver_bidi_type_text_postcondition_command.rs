use std::{error::Error, io};

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, MAX_WEBDRIVER_BIDI_COMMAND_ID, NodeHandleError, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
    WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD, WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION,
    WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiTextValueObservationAuthorityError, WebDriverBiDiTextValueObservationCommand,
    WebDriverBiDiTextValueObservationCommandError,
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
fn text_value_postcondition_is_a_fixed_sandboxed_node_observation() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let command = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    assert_eq!(command.command_id(), 43);
    assert_eq!(command.method(), WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD);
    assert_eq!(command.browsing_context(), "context-a");
    assert_eq!(command.sandbox(), WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX);
    assert_eq!(
        command.function_declaration(),
        WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION
    );
    assert_eq!(
        command.as_json(),
        r#"{"id":43,"method":"script.callFunction","params":{"functionDeclaration":"node => node.value","awaitPromise":false,"target":{"context":"context-a","sandbox":"originweave-postcondition-v1"},"arguments":[{"sharedId":"shared-input-42"}],"resultOwnership":"none"}}"#
    );
    Ok(())
}

#[test]
fn text_value_postcondition_escapes_only_protocol_identifiers() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field(r#"context-"quoted"\path"#, r#"node-"quoted"\path"#)?;
    let command = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    assert!(command.as_json().contains(r#"context-\"quoted\"\\path"#));
    assert!(command.as_json().contains(r#"node-\"quoted\"\\path"#));
    assert_eq!(
        command.function_declaration(),
        "node => node.value",
        "page/model text must never become executable source"
    );
    Ok(())
}

#[test]
fn text_value_postcondition_rejects_invalid_command_id() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let error = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected oversized command id rejection")?;

    assert_eq!(
        error,
        WebDriverBiDiTextValueObservationAuthorityError::Command(
            WebDriverBiDiTextValueObservationCommandError::InvalidCommandId,
        )
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("rejected input"));
    assert_eq!(
        WebDriverBiDiTextValueObservationCommandError::InvalidCommandId.to_string(),
        "WebDriver BiDi command id is outside the js-uint range"
    );
    Ok(())
}

#[test]
fn text_value_postcondition_rejects_wrong_context_and_unadmitted_node()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("context-a", "shared-input-42")?;
    let wrong_context = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        "context-b",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected wrong external context rejection")?;
    assert_eq!(
        wrong_context,
        WebDriverBiDiTextValueObservationAuthorityError::BrowserAuthority(
            BrowserRegistryError::ContextExternalIdentifierMismatch,
        )
    );
    assert!(wrong_context.source().is_some());
    assert!(wrong_context.to_string().contains("browser authority"));

    let forged = WebDriverBiDiRemoteNodeReference::new("node", Some("unadmitted-node"))?;
    let wrong_node = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &forged,
        &fixture.registry,
    )
    .err()
    .ok_or("expected unadmitted sharedId rejection")?;
    assert_eq!(
        wrong_node,
        WebDriverBiDiTextValueObservationAuthorityError::NodeExternalIdentifierMismatch
    );
    assert!(wrong_node.source().is_none());
    assert!(wrong_node.to_string().contains("wire node identifier"));
    Ok(())
}

#[test]
fn text_value_postcondition_rejects_cross_registry_handle() -> Result<(), Box<dyn Error>> {
    let local = admitted_text_field("context-a", "shared-input-42")?;
    let foreign = admitted_text_field("context-a", "shared-input-42")?;

    let error = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &local.external_context,
        &foreign.handle,
        &local.remote,
        &local.registry,
    )
    .err()
    .ok_or("expected cross-registry node rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiTextValueObservationAuthorityError::NodeExternalIdentifierMismatch
    );
    Ok(())
}

#[test]
fn text_value_postcondition_rejects_changed_origin_authority() -> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_text_field("context-a", "shared-input-42")?;
    fixture.registry.advance_document(fixture.browsing_context)?;
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

    let error = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected changed-origin rejection")?;
    assert!(matches!(
        error,
        WebDriverBiDiTextValueObservationAuthorityError::BrowserAuthority(_)
    ));
    assert!(error.source().is_some());
    assert!(error.to_string().contains("browser authority"));
    Ok(())
}

#[test]
fn text_value_postcondition_rejects_stale_document_authority() -> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_text_field("context-a", "shared-input-42")?;
    let observed = fixture.handle.document_epoch();
    let current = fixture.registry.advance_document(fixture.browsing_context)?;
    let origin = fixture.handle.origin().clone();
    fixture.registry.bind_context_origin(
        fixture.browser_session,
        fixture.browsing_context,
        &origin,
    )?;

    let error = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected stale document rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiTextValueObservationAuthorityError::NodeHandle(
            NodeHandleError::StaleDocumentEpoch { observed, current },
        )
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("node authority"));
    Ok(())
}

#[test]
fn text_value_postcondition_debug_omits_wire_identifiers() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_text_field("buyer-private-context", "buyer-private-node")?;
    let command = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        &fixture.external_context,
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    let debug = format!("{command:?}");
    assert!(!debug.contains("buyer-private-context"));
    assert!(!debug.contains("buyer-private-node"));
    assert!(debug.contains("command_id: 43"));
    assert!(debug.contains(WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD));
    assert!(debug.contains(WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX));
    Ok(())
}
