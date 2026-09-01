use std::{error::Error, io};

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WEBDRIVER_BIDI_SCRIPT_CALL_FUNCTION_METHOD,
    WEBDRIVER_BIDI_TEXT_VALUE_FUNCTION_DECLARATION, WEBDRIVER_BIDI_TEXT_VALUE_SANDBOX,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiTextValueObservationCommand,
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

#[test]
fn text_value_postcondition_is_a_fixed_sandboxed_node_observation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let browsing_context = registry.register_context(browser_session, "context-a")?;
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
    let locate = WebDriverBiDiLocateNodesCommand::new(41, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{"nodes":[{"type":"node","sharedId":"shared-input-42"}]}}"#,
    )?;
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
    let remote = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-input-42"))?;

    let command = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        43,
        "context-a",
        &handle,
        &remote,
        &registry,
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
