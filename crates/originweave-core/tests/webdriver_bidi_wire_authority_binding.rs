use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserRegistryError, Origin, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesAdmissionError,
    WebDriverBiDiLocateNodesCommand, WebDriverBiDiLocateNodesResponseDocumentError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn locate_nodes_command() -> Result<WebDriverBiDiLocateNodesCommand, Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 1)?;
    Ok(WebDriverBiDiLocateNodesCommand::new(
        42,
        "context-a",
        &query,
    )?)
}

fn controlled_origin() -> Result<Origin, Box<dyn Error>> {
    Origin::parse("https://app.example").map_err(|_error| "valid controlled fixture origin".into())
}

fn current_target<'a>(
    registry: &mut BrowserAuthorityRegistry,
    origin: &'a Origin,
    external_context: &str,
) -> Result<BrowserContextOriginEpochDispatchTarget<'a>, Box<dyn Error>> {
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, external_context)?;
    let epoch = registry.bind_context_origin(session, context, origin)?;
    Ok(BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            origin,
        ),
        epoch,
    ))
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

fn successful_wire_document() -> Result<BoundedWebDriverBiDiResponseDocument, Box<dyn Error>> {
    Ok(BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":42,"result":{"nodes":[{"type":"node","sharedId":"node-a"}]}}"#,
    )?)
}

#[test]
fn wire_response_binds_nodes_to_exact_current_authority_without_caller_selected_intermediate_result()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;

    let handles = locate_nodes_command()?.bind_response_document_nodes(
        successful_wire_document()?,
        semantic_observation_proof()?,
        &mut registry,
        target,
    )?;

    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].origin(), &origin);
    assert_eq!(handles[0].document_epoch(), target.expected_epoch());
    Ok(())
}

#[test]
fn wire_response_binding_preserves_current_context_authority_failure() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-b")?;

    let error = locate_nodes_command()?.bind_response_document_nodes(
        successful_wire_document()?,
        semantic_observation_proof()?,
        &mut registry,
        target,
    );

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResponseDocumentError::NodeBinding(
            WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
                BrowserRegistryError::ContextExternalIdentifierMismatch,
            ),
        ))
    );
    let error = error
        .err()
        .ok_or("expected exact current context failure")?;
    assert!(error.source().is_some());
    assert!(!error.to_string().is_empty());
    Ok(())
}
