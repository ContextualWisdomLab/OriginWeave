use std::error::Error;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AdmittedNodeHandle,
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry,
    BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, InstructionSource, Origin,
    OriginWeaveProtocolVersion, SecretDelivery, SemanticNodeActionBinding,
    SemanticNodeActionBindingError, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";
const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

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

fn admitted_node() -> Result<(BrowserAuthorityRegistry, AdmittedNodeHandle), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("policy-binding-session")?;
    let context = registry.register_context(session, "policy-binding-context")?;
    let source_origin = Origin::parse("https://app.example")
        .map_err(|error| std::io::Error::other(format!("fixture origin rejected: {error:?}")))?;
    let epoch = registry.bind_context_origin(session, context, &source_origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &source_origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task title"), 1)?;
    let command = WebDriverBiDiLocateNodesCommand::new(41, "policy-binding-context", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":41,"result":{"nodes":[{"type":"node","sharedId":"shared-node-42"}]}}"#,
    )?;
    let handle = command
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter()
        .next()
        .ok_or("locateNodes fixture did not bind its node")?;
    Ok((registry, handle))
}

fn action_request(source: Origin, target: Origin) -> Result<ActionRequest, Box<dyn Error>> {
    let intent = ActionIntentDigest::parse(VALID_INTENT)
        .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?;
    Ok(ActionRequest::new(
        ActionKind::Draft,
        source,
        target,
        InstructionSource::User,
        SecretDelivery::None,
        intent,
    ))
}

#[test]
fn action_binding_keeps_admitted_node_and_business_target_separate() -> Result<(), Box<dyn Error>> {
    let (_registry, handle) = admitted_node()?;
    let node_origin = handle.origin().clone();
    let node_id = handle.node_id();
    let business_target = Origin::parse("https://destination.example")
        .map_err(|error| std::io::Error::other(format!("target origin rejected: {error:?}")))?;
    let request = action_request(node_origin, business_target.clone())?;

    let binding = SemanticNodeActionBinding::new(handle, request)?;

    assert_eq!(binding.handle().node_id(), node_id);
    assert_eq!(binding.request().target_origin(), &business_target);
    assert_eq!(binding.request().action(), ActionKind::Draft);
    Ok(())
}

#[test]
fn action_binding_rejects_business_request_from_another_document_origin()
-> Result<(), Box<dyn Error>> {
    let (_registry, handle) = admitted_node()?;
    let other_origin = Origin::parse("https://other.example")
        .map_err(|error| std::io::Error::other(format!("other origin rejected: {error:?}")))?;
    let target_origin = Origin::parse("https://destination.example")
        .map_err(|error| std::io::Error::other(format!("target origin rejected: {error:?}")))?;
    let request = action_request(other_origin, target_origin)?;

    let error = SemanticNodeActionBinding::new(handle, request)
        .err()
        .ok_or("mismatched source origin unexpectedly admitted")?;
    assert_eq!(error, SemanticNodeActionBindingError::SourceOriginMismatch);
    assert_eq!(
        error.to_string(),
        "admitted node origin does not match action request source origin"
    );
    assert!(error.source().is_none());
    Ok(())
}
