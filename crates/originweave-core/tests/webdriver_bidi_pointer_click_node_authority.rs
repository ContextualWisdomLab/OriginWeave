use std::error::Error;

use originweave_core::{
    AdmittedNodeHandle, BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, MAX_WEBDRIVER_BIDI_COMMAND_ID, NodeHandleError, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiPointerClickAuthorityError,
    WebDriverBiDiPointerClickCommand, WebDriverBiDiPointerClickCommandError,
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

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &fixture.handle,
        &forged,
        &fixture.registry,
    )
    .err()
    .ok_or("expected unadmitted sharedId rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch
    );
    assert!(error.source().is_none());
    assert!(error.to_string().contains("wire node identifier"));
    Ok(())
}

#[test]
fn pointer_click_rejects_the_right_node_under_the_wrong_external_context()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-b",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected external context rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
            BrowserRegistryError::ContextExternalIdentifierMismatch,
        )
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("browser authority"));
    Ok(())
}

#[test]
fn pointer_click_rejects_a_pre_navigation_node_before_new_origin_binding()
-> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_node()?;
    fixture.registry.advance_document(fixture.browsing_context)?;

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected missing current origin rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
            BrowserRegistryError::ContextOriginNotBound,
        )
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("browser authority"));
    Ok(())
}

#[test]
fn pointer_click_rejects_a_stale_node_after_new_origin_is_rebound() -> Result<(), Box<dyn Error>> {
    let mut fixture = admitted_node()?;
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

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected stale document rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::NodeHandle(NodeHandleError::StaleDocumentEpoch {
            observed,
            current,
        })
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("node authority"));
    Ok(())
}

#[test]
fn pointer_click_rejects_an_admitted_node_from_another_registry_even_when_public_fields_match()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let foreign = admitted_node()?;

    assert_eq!(fixture.browser_session, foreign.handle.browser_session());
    assert_eq!(fixture.browsing_context, foreign.handle.browsing_context());
    assert_eq!(fixture.handle.origin(), foreign.handle.origin());
    assert_eq!(
        fixture.handle.document_epoch(),
        foreign.handle.document_epoch()
    );
    assert_eq!(fixture.handle.node_id(), foreign.handle.node_id());
    assert_eq!(fixture.remote.shared_id(), foreign.remote.shared_id());

    assert_eq!(
        WebDriverBiDiPointerClickCommand::new_for_current_node(
            42,
            "context-a",
            &foreign.handle,
            &fixture.remote,
            &fixture.registry,
        ),
        Err(WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch)
    );
    Ok(())
}

#[test]
fn pointer_click_rejects_a_matching_context_bound_to_a_different_origin()
-> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;
    let mut foreign_registry = BrowserAuthorityRegistry::new();
    let session = foreign_registry.register_session("webdriver-session")?;
    let context = foreign_registry.register_context(session, "context-a")?;
    let other_origin = Origin::parse("https://other.example").map_err(|error| {
        std::io::Error::other(format!("fixture origin rejected unexpectedly: {error:?}"))
    })?;
    foreign_registry.bind_context_origin(session, context, &other_origin)?;

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        42,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &foreign_registry,
    )
    .err()
    .ok_or("expected origin mismatch rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::BrowserAuthority(
            BrowserRegistryError::OriginChangedWithoutDocumentAdvance,
        )
    );
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn pointer_click_reports_bounded_command_serialization_failure() -> Result<(), Box<dyn Error>> {
    let fixture = admitted_node()?;

    let error = WebDriverBiDiPointerClickCommand::new_for_current_node(
        MAX_WEBDRIVER_BIDI_COMMAND_ID + 1,
        "context-a",
        &fixture.handle,
        &fixture.remote,
        &fixture.registry,
    )
    .err()
    .ok_or("expected command identifier rejection")?;
    assert_eq!(
        error,
        WebDriverBiDiPointerClickAuthorityError::Command(
            WebDriverBiDiPointerClickCommandError::InvalidCommandId,
        )
    );
    assert!(error.source().is_some());
    assert!(error.to_string().contains("command rejected input"));
    Ok(())
}

#[test]
fn authority_registry_rejects_document_advance_for_a_foreign_context()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("local-session")?;
    let _local_context = registry.register_context(session, "local-context")?;

    let mut foreign_registry = BrowserAuthorityRegistry::new();
    let foreign_session = foreign_registry.register_session("foreign-session")?;
    let _first_foreign_context =
        foreign_registry.register_context(foreign_session, "foreign-context-a")?;
    let second_foreign_context =
        foreign_registry.register_context(foreign_session, "foreign-context-b")?;

    assert_eq!(
        registry.advance_document(second_foreign_context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    Ok(())
}
