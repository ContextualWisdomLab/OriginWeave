use std::{collections::BTreeSet, error::Error};

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BoundedWebDriverBiDiResponseDocument,
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolKind, BrowserRegistryError, BrowsingContextId, ExecutionPurpose, InstructionSource,
    NodeActionKind, Origin, OriginWeaveProtocolVersion, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SessionMode, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
};
use originweave_policy::PolicyAuthorizedSemanticNodeAction;

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion = OriginWeaveProtocolVersion::new(0, 1);
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

fn authorized_fixture(
) -> Result<(BrowserAuthorityRegistry, BrowsingContextId, PolicyAuthorizedSemanticNodeAction), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-policy-current-authority-session")?;
    let context = registry.register_context(session, "semantic-policy-current-authority-context")?;
    let site = Origin::parse("https://app.example")
        .map_err(|error| std::io::Error::other(format!("fixture origin rejected: {error:?}")))?;
    let epoch = registry.bind_context_origin(session, context, &site)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &site,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Continue"), 1)?;
    let command = WebDriverBiDiLocateNodesCommand::new(
        71,
        "semantic-policy-current-authority-context",
        &query,
    )?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":71,"result":{"nodes":[{"type":"node","sharedId":"semantic-policy-current-authority-node"}]}}"#,
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
    let request = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT)
            .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?,
    );
    let binding = SemanticNodeActionBinding::new(handle, NodeActionKind::Click, request)?;
    let policy_context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([ActionKind::Navigate.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &policy_context)?;

    Ok((registry, context, authorized))
}

#[test]
fn policy_authorized_action_revalidates_registry_owned_browser_authority() -> Result<(), Box<dyn Error>> {
    let (mut registry, context, authorized) = authorized_fixture()?;

    authorized.validate_current(&registry)?;
    registry.advance_document(context)?;

    assert_eq!(
        authorized.validate_current(&registry).err(),
        Some(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}
