use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, Capability, DocumentEpoch, ExecutionPurpose, InstructionSource,
    NodeActionKind, ObservationChannel, ObservedNodeHandle, Origin, PolicyContext, RiskClass,
    RobotsDecision, SecretDelivery, SemanticNodeActionBinding, SemanticNodeActionTarget,
    SemanticNodeObservation, SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::{
    DenialReason, PolicyAuthorizedSemanticNodeAction, SemanticNodePolicyAuthorizationError,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn binding(
    action: ActionKind,
    instruction_source: InstructionSource,
) -> Result<SemanticNodeActionBinding, String> {
    let site = origin("https://app.example")?;
    let handle = ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        site.clone(),
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())?;
    let observation = SemanticNodeObservation::new(SemanticNodeObservationInput {
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
    })
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
    SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())
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
    let binding = binding(ActionKind::Navigate, InstructionSource::User)?;
    let context = context(ActionKind::Navigate)?;

    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding.clone(), &context)
        .map_err(|error| error.to_string())?;

    assert_eq!(authorized.binding(), &binding);
    assert_eq!(
        authorized.binding().request().action(),
        ActionKind::Navigate
    );
    Ok(())
}

#[test]
fn semantic_node_action_preserves_approval_required_as_non_authorized() -> Result<(), String> {
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
fn semantic_node_action_preserves_policy_denial_as_non_authorized() -> Result<(), String> {
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
fn policy_authorized_semantic_node_action_still_revalidates_browser_authority() -> Result<(), String>
{
    let binding = binding(ActionKind::Navigate, InstructionSource::User)?;
    let context = context(ActionKind::Navigate)?;
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &context)
        .map_err(|error| error.to_string())?;

    let error = authorized
        .validate_current(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            &origin("https://app.example")?,
            DocumentEpoch::new(4).map_err(|error| error.to_string())?,
        )
        .err()
        .ok_or_else(|| "stale document epoch unexpectedly authorized".to_owned())?;

    assert!(error.to_string().contains("stale"));
    Ok(())
}

#[test]
fn semantic_node_policy_authorization_errors_are_credential_free() {
    assert_eq!(
        SemanticNodePolicyAuthorizationError::Denied(DenialReason::UntrustedInstructionSource)
            .to_string(),
        "semantic node action denied by deterministic policy: untrusted instruction source"
    );
    assert_eq!(
        SemanticNodePolicyAuthorizationError::ApprovalRequired(RiskClass::R4).to_string(),
        "semantic node action requires R4 approval before policy authorization"
    );
}
