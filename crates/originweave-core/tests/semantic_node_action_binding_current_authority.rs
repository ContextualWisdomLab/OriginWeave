use std::error::Error;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AdmittedNodeHandle,
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserRegistryError, BrowsingContextId, InstructionSource, NodeActionKind, Origin,
    OriginWeaveProtocolVersion, SecretDelivery, SemanticNodeActionBinding,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
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

fn admitted_node(
) -> Result<(BrowserAuthorityRegistry, BrowsingContextId, AdmittedNodeHandle), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("binding-current-authority-session")?;
    let context = registry.register_context(session, "binding-current-authority-context")?;
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
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Continue"), 1)?;
    let command = WebDriverBiDiLocateNodesCommand::new(
        51,
        "binding-current-authority-context",
        &query,
    )?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":51,"result":{"nodes":[{"type":"node","sharedId":"binding-current-authority-node"}]}}"#,
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
    Ok((registry, context, handle))
}

fn binding(handle: AdmittedNodeHandle) -> Result<SemanticNodeActionBinding, Box<dyn Error>> {
    let source_origin = handle.origin().clone();
    let target_origin = Origin::parse("https://destination.example")
        .map_err(|error| std::io::Error::other(format!("target origin rejected: {error:?}")))?;
    let intent = ActionIntentDigest::parse(VALID_INTENT)
        .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?;
    let request = ActionRequest::new(
        ActionKind::Draft,
        source_origin,
        target_origin,
        InstructionSource::User,
        SecretDelivery::None,
        intent,
    );
    Ok(SemanticNodeActionBinding::new(
        handle,
        NodeActionKind::Click,
        request,
    )?)
}

#[test]
fn action_binding_revalidates_exact_registry_issued_node_authority() -> Result<(), Box<dyn Error>> {
    let (registry, _context, handle) = admitted_node()?;
    let binding = binding(handle)?;

    binding.validate_current(&registry)?;
    Ok(())
}

#[test]
fn action_binding_rejects_stale_document_authority() -> Result<(), Box<dyn Error>> {
    let (mut registry, context, handle) = admitted_node()?;
    let binding = binding(handle)?;
    registry.advance_document(context)?;

    assert_eq!(
        binding.validate_current(&registry),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn action_binding_rejects_foreign_registry_even_for_reproducible_descriptive_tuple(
) -> Result<(), Box<dyn Error>> {
    let (_registry, _context, handle) = admitted_node()?;
    let binding = binding(handle)?;
    let foreign_registry = BrowserAuthorityRegistry::new();

    assert_eq!(
        binding.validate_current(&foreign_registry),
        Err(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}
