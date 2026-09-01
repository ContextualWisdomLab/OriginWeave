use std::collections::BTreeSet;
use std::error::Error;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AdmittedNodeHandle, ApprovalEvidence,
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind, Capability,
    ExecutionPurpose, InstructionSource, Origin, OriginWeaveProtocolVersion, PolicyContext,
    RiskClass, RobotsDecision, SecretDelivery, SemanticNodeActionBinding, SessionMode,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
};
use originweave_policy::{
    DenialReason, PolicyAuthorizedSemanticNodeAction, SemanticNodePolicyAuthorizationError,
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

fn admitted_node() -> Result<AdmittedNodeHandle, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-policy-session")?;
    let context = registry.register_context(session, "semantic-policy-context")?;
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
    let command = WebDriverBiDiLocateNodesCommand::new(51, "semantic-policy-context", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":51,"result":{"nodes":[{"type":"node","sharedId":"semantic-policy-node"}]}}"#,
    )?;
    command
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter()
        .next()
        .ok_or_else(|| "locateNodes fixture did not bind its node".into())
}

fn binding(
    action: ActionKind,
    instruction_source: InstructionSource,
) -> Result<SemanticNodeActionBinding, Box<dyn Error>> {
    let handle = admitted_node()?;
    let source_origin = handle.origin().clone();
    let request = ActionRequest::new(
        action,
        source_origin.clone(),
        source_origin,
        instruction_source,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT)
            .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?,
    );
    Ok(SemanticNodeActionBinding::new(handle, request)?)
}

fn context(action: ActionKind) -> Result<PolicyContext, Box<dyn Error>> {
    let site = Origin::parse("https://app.example")
        .map_err(|error| std::io::Error::other(format!("context origin rejected: {error:?}")))?;
    Ok(PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([action.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    ))
}

#[test]
fn semantic_node_action_becomes_policy_authorized_only_after_allow() -> Result<(), Box<dyn Error>> {
    let binding = binding(ActionKind::Navigate, InstructionSource::User)?;
    let node_id = binding.handle().node_id();
    let context = context(ActionKind::Navigate)?;

    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &context)?;

    assert_eq!(authorized.binding().handle().node_id(), node_id);
    assert_eq!(
        authorized.binding().request().action(),
        ActionKind::Navigate
    );
    Ok(())
}

#[test]
fn semantic_node_action_preserves_approval_required_as_non_authorized() -> Result<(), Box<dyn Error>> {
    let binding = binding(ActionKind::Purchase, InstructionSource::User)?;
    let context = context(ActionKind::Purchase)?;

    assert_eq!(
        PolicyAuthorizedSemanticNodeAction::authorize(binding, &context).err(),
        Some(SemanticNodePolicyAuthorizationError::ApprovalRequired(
            RiskClass::R4
        ))
    );
    Ok(())
}

#[test]
fn semantic_node_action_preserves_policy_denial_as_non_authorized() -> Result<(), Box<dyn Error>> {
    let binding = binding(ActionKind::Navigate, InstructionSource::WebContent)?;
    let context = context(ActionKind::Navigate)?;

    assert_eq!(
        PolicyAuthorizedSemanticNodeAction::authorize(binding, &context).err(),
        Some(SemanticNodePolicyAuthorizationError::Denied(
            DenialReason::UntrustedInstructionSource
        ))
    );
    Ok(())
}

#[test]
fn semantic_node_policy_authorization_errors_are_credential_free() {
    let denial_cases = [
        (
            DenialReason::HumanModeNotAgentControlled,
            "human mode is not agent controlled",
        ),
        (
            DenialReason::ModePurposeMismatch,
            "execution mode and purpose mismatch",
        ),
        (
            DenialReason::UntrustedInstructionSource,
            "untrusted instruction source",
        ),
        (
            DenialReason::MissingCapability(Capability::Navigate),
            "required capability is missing",
        ),
        (
            DenialReason::OriginNotReadable,
            "target origin is not readable",
        ),
        (
            DenialReason::CrawlerMutation,
            "crawler mutation is forbidden",
        ),
        (
            DenialReason::CrossOriginMutation,
            "cross-origin mutation is forbidden",
        ),
        (
            DenialReason::OriginNotWritable,
            "target origin is not writable",
        ),
        (
            DenialReason::RobotsDisallowed,
            "robots policy disallows the crawl",
        ),
        (DenialReason::RobotsUnknown, "robots policy is unknown"),
        (
            DenialReason::RobotsNotApplicable,
            "robots policy was not evaluated",
        ),
        (
            DenialReason::SecretBrokerRequired,
            "secret broker handle is required",
        ),
        (
            DenialReason::UnexpectedSecretMaterial,
            "unexpected secret material",
        ),
        (DenialReason::ForbiddenRisk, "risk class is not delegable"),
        (
            DenialReason::ApprovalScopeMismatch,
            "approval scope does not match",
        ),
    ];

    for (reason, expected_reason) in denial_cases {
        assert_eq!(
            SemanticNodePolicyAuthorizationError::Denied(reason).to_string(),
            format!("semantic node action denied by deterministic policy: {expected_reason}")
        );
    }
    assert_eq!(
        SemanticNodePolicyAuthorizationError::ApprovalRequired(RiskClass::R4).to_string(),
        "semantic node action requires R4 approval before policy authorization"
    );
}
