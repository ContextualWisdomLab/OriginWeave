use std::error::Error;

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserSessionId, BrowsingContextId, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiPointerClickAuthorityError,
    WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

struct AdmittedNodeFixture {
    registry: BrowserAuthorityRegistry,
    handle: AdmittedNodeHandle,
    remote: WebDriverBiDiRemoteNodeReference,
}

fn protocol_proof(
    kind: BrowserProtocolKind,
    capability: BrowserProtocolCapability,
) -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        kind,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[
            BrowserProtocolCapability::SemanticObservation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        kind,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        capability,
    )?)
}

fn admitted_node() -> Result<AdmittedNodeFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session: BrowserSessionId = registry.register_session("webdriver-session")?;
    let browsing_context: BrowsingContextId =
        registry.register_context(browser_session, "context-a")?;
    let origin = Origin::parse("https://app.example").map_err(|error| {
        std::io::Error::other(format!("fixture origin rejected unexpectedly: {error:?}"))
    })?;
    let epoch = registry.bind_context_origin(browser_session, browsing_context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, browsing_context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 1)?;
    let command = WebDriverBiDiLocateNodesCommand::new(41, "context-a", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{"nodes":[{"type":"node","sharedId":"shared-node-42"}]}}"#,
    )?;
    let handles = command.bind_response_document_nodes(
        document,
        protocol_proof(
            BrowserProtocolKind::WebDriverBiDi,
            BrowserProtocolCapability::SemanticObservation,
        )?,
        &mut registry,
        target,
    )?;
    let handle = handles
        .into_iter()
        .next()
        .ok_or("locateNodes fixture did not bind its node")?;
    let remote = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;
    Ok(AdmittedNodeFixture {
        registry,
        handle,
        remote,
    })
}

#[test]
fn pointer_click_requires_webdriver_bidi_typed_input_proof() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let command = WebDriverBiDiPointerClickCommand::new_for_current_node(
        protocol_proof(
            BrowserProtocolKind::WebDriverBiDi,
            BrowserProtocolCapability::TypedInput,
        )?,
        42,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;
    assert_eq!(command.command_id(), 42);

    let wrong_capability = WebDriverBiDiPointerClickCommand::new_for_current_node(
        protocol_proof(
            BrowserProtocolKind::WebDriverBiDi,
            BrowserProtocolCapability::SemanticObservation,
        )?,
        43,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected semantic-observation proof rejection")?;
    assert_eq!(
        wrong_capability,
        WebDriverBiDiPointerClickAuthorityError::UnsupportedCapability(
            BrowserProtocolCapability::SemanticObservation,
        )
    );

    let wrong_protocol = WebDriverBiDiPointerClickCommand::new_for_current_node(
        protocol_proof(
            BrowserProtocolKind::ChromeDevToolsProtocol,
            BrowserProtocolCapability::TypedInput,
        )?,
        44,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected CDP typed-input proof rejection")?;
    assert_eq!(
        wrong_protocol,
        WebDriverBiDiPointerClickAuthorityError::UnsupportedProtocolKind(
            BrowserProtocolKind::ChromeDevToolsProtocol,
        )
    );
    Ok(())
}
