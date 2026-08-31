use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserRegistryError, BrowserSessionId, BrowsingContextId, NodeHandleError, ObservedNodeHandle,
    Origin, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiPointerClickAuthorityError, WebDriverBiDiPointerClickCommand,
    WebDriverBiDiRemoteNodeReference,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

struct AdmittedNodeFixture {
    registry: BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    handle: ObservedNodeHandle,
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

fn admitted_node() -> Result<AdmittedNodeFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let browser_session = registry.register_session("webdriver-session")?;
    let browsing_context = registry.register_context(browser_session, "context-a")?;
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
        semantic_observation_proof()?,
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
        browser_session,
        browsing_context,
        handle,
        remote,
    })
}

#[test]
fn pointer_click_serialization_requires_the_exact_current_admitted_wire_node()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let command = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )?;

    assert_eq!(command.command_id(), 42);
    assert!(command.as_json().contains(r#""sharedId":"shared-node-42""#));
    Ok(())
}

#[test]
fn pointer_click_rejects_a_caller_selected_unadmitted_shared_id() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let forged = WebDriverBiDiRemoteNodeReference::new("node", Some("caller-selected-node"))?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-a",
            &fixture.handle,
            &forged,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch)
    );
    Ok(())
}

#[test]
fn pointer_click_rejects_the_right_node_under_the_wrong_external_context()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-b",
            &fixture.handle,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
            BrowserRegistryError::ContextExternalIdentifierMismatch,
        ))
    );
    Ok(())
}

#[test]
fn pointer_click_rejects_a_pre_navigation_node_after_document_advance() -> Result<(), Box<dyn Error>>
{
    let mut fixture = admitted_node()?;
    let observed = fixture.handle.document_epoch();
    let current = fixture
        .registry
        .advance_document(fixture.browsing_context)?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-a",
            &fixture.handle,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::NodeHandle(
            NodeHandleError::StaleDocumentEpoch { observed, current },
        ))
    );
    Ok(())
}

#[test]
fn pointer_click_rejects_a_fabricated_current_epoch_handle_without_registry_node_authority()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let fabricated = ObservedNodeHandle::new(
        fixture.browser_session,
        fixture.browsing_context,
        fixture.handle.origin().clone(),
        fixture.handle.document_epoch(),
        fixture.handle.node_id() + 1,
    )?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-a",
            &fabricated,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch)
    );
    Ok(())
}

#[test]
fn pointer_click_rejects_a_publicly_fabricated_handle_that_copies_the_exact_admitted_tuple()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let fabricated = ObservedNodeHandle::new(
        fixture.handle.browser_session(),
        fixture.handle.browsing_context(),
        fixture.handle.origin().clone(),
        fixture.handle.document_epoch(),
        fixture.handle.node_id(),
    )?;

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-a",
            &fabricated,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch)
    );
    Ok(())
}
