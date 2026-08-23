use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserAuthorityRegistry,
    BrowserRegistryError, BrowsingContextId, Capability, ExecutionPurpose, InstructionSource,
    NodeActionKind, ObservationChannel, Origin, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SemanticNodeActionBinding, SemanticNodeActionTarget, SemanticNodeObservation,
    SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::{
    DenialReason, PolicyAuthorizedSemanticNodeAction, SemanticNodePolicyAuthorizationError,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct BindingFixture {
    registry: BrowserAuthorityRegistry,
    context: BrowsingContextId,
    binding: SemanticNodeActionBinding,
}

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn binding(
    action: ActionKind,
    instruction_source: InstructionSource,
) -> Result<BindingFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-policy-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-policy-context")
        .map_err(|error| error.to_string())?;
    let site = origin("https://app.example")?;
    let handle = registry
        .bind_node(session, context, &site, "semantic-policy-node")
        .map_err(|error| error.to_string())?;
    let observation = SemanticNodeObservation::new(
        SemanticNodeObservationInput {
            handle,
            parent: None,
            children: Vec::new(),
            role: "button".to_owned(),
            accessible_name: "Continue".to_owned(),
            visible_text: Some("Continue".to_owned()),
            enabled: true,
            visible: true,
            selected: None,
            supported_actions: BTreeSet::from([NodeActionKind::Click]),
            evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
        },
        &registry,
    )
    .map_err(|error| error.to_string())?;
    let target = SemanticNodeActionTarget::from_observation(&observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = ActionRequest::new(
        action,
        site.clone(),
        site,
        instruction_source,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT).map_err(|error| format!("{error:?}"))?,
    );
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;

    Ok(BindingFixture {
        registry,
        context,
        binding,
    })
}

fn context(action: ActionKind) -> Result<PolicyContext, String> {
    let site = origin("https://app.example")?;
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
fn semantic_node_action_becomes_policy_authorized_only_after_allow() -> Result<(), String> {
    let fixture = binding(ActionKind::Navigate, InstructionSource::User)?;
    let context = context(ActionKind::Navigate)?;

    let authorized =
        PolicyAuthorizedSemanticNodeAction::authorize(fixture.binding.clone(), &context)
            .map_err(|error| error.to_string())?;

    assert_eq!(authorized.binding(), &fixture.binding);
    assert_eq!(
        authorized.binding().request().action(),
        ActionKind::Navigate
    );
    authorized
        .validate_current(&fixture.registry)
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn semantic_node_action_preserves_approval_required_as_non_authorized() -> Result<(), String> {
    let fixture = binding(ActionKind::Purchase, InstructionSource::User)?;
    let context = context(ActionKind::Purchase)?;

    assert_eq!(
        PolicyAuthorizedSemanticNodeAction::authorize(fixture.binding, &context).err(),
        Some(SemanticNodePolicyAuthorizationError::ApprovalRequired(
            RiskClass::R4
        ))
    );
    Ok(())
}

#[test]
fn semantic_node_action_preserves_policy_denial_as_non_authorized() -> Result<(), String> {
    let fixture = binding(ActionKind::Navigate, InstructionSource::WebContent)?;
    let context = context(ActionKind::Navigate)?;

    assert_eq!(
        PolicyAuthorizedSemanticNodeAction::authorize(fixture.binding, &context).err(),
        Some(SemanticNodePolicyAuthorizationError::Denied(
            DenialReason::UntrustedInstructionSource
        ))
    );
    Ok(())
}

#[test]
fn policy_authorized_semantic_node_action_still_revalidates_browser_authority() -> Result<(), String>
{
    let mut fixture = binding(ActionKind::Navigate, InstructionSource::User)?;
    let context = context(ActionKind::Navigate)?;
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(fixture.binding, &context)
        .map_err(|error| error.to_string())?;

    fixture
        .registry
        .advance_document(fixture.context)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        authorized.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownNodeAuthority)
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
