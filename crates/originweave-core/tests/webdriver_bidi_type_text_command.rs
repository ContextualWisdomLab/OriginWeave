use std::{error::Error, io};

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse, WEBDRIVER_BIDI_PERFORM_ACTIONS_METHOD,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiTypeTextCommand,
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

fn admitted_text_field() -> Result<
    (
        BrowserAuthorityRegistry,
        AdmittedNodeHandle,
        WebDriverBiDiRemoteNodeReference,
    ),
    Box<dyn Error>,
> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(browser_session, "context-a")?;
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
    Ok((registry, handle, remote))
}

#[test]
fn type_text_command_focuses_exact_admitted_node_before_keyboard_input()
-> Result<(), Box<dyn Error>> {
    let (registry, handle, node) = admitted_text_field()?;
    let command = WebDriverBiDiTypeTextCommand::new_for_current_node(
        42,
        "context-a",
        "Az",
        &handle,
        &node,
        &registry,
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
