use std::cell::Cell;
use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, ExecutionPurpose, InstructionSource, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SemanticNodeActionTarget, SemanticNodeActionTargetError,
    SemanticNodeObservation, SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::PolicyAuthorizedSemanticNodeAction;

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn observation(
    node_id: u64,
    enabled: bool,
    supported_actions: BTreeSet<NodeActionKind>,
) -> Result<SemanticNodeObservation, String> {
    SemanticNodeObservation::new(SemanticNodeObservationInput {
        handle: ObservedNodeHandle::new(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            origin("https://app.example")?,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
            node_id,
        )
        .map_err(|error| error.to_string())?,
        parent: None,
        children: Vec::new(),
        role: "button".to_owned(),
        accessible_name: "Continue".to_owned(),
        visible_text: Some("Continue".to_owned()),
        enabled,
        visible: true,
        selected: None,
        supported_actions,
        evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
    })
    .map_err(|error| error.to_string())
}

fn authorized_action() -> Result<PolicyAuthorizedSemanticNodeAction, String> {
    let site = origin("https://app.example")?;
    let initial_observation = observation(17, true, BTreeSet::from([NodeActionKind::Click]))?;
    let target =
        SemanticNodeActionTarget::from_observation(&initial_observation, NodeActionKind::Click)
            .map_err(|error| error.to_string())?;
    let request = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT).map_err(|error| format!("{error:?}"))?,
    );
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;
    let context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([ActionKind::Navigate.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );

    PolicyAuthorizedSemanticNodeAction::authorize(binding, &context)
        .map_err(|error| error.to_string())
}

fn dispatch_action(
    authorized: &PolicyAuthorizedSemanticNodeAction,
    current: &SemanticNodeObservation,
    called: &Cell<bool>,
    adapter_should_fail: bool,
) -> Result<Result<(NodeActionKind, ActionKind), &'static str>, SemanticNodeActionTargetError> {
    authorized.dispatch_if_current_observation(current, |binding| {
        called.set(true);
        if adapter_should_fail {
            Err("adapter failed")
        } else {
            Ok((binding.target().action(), binding.request().action()))
        }
    })
}

#[test]
fn exact_current_semantic_observation_reaches_dispatch() -> Result<(), String> {
    let authorized = authorized_action()?;
    let current = observation(17, true, BTreeSet::from([NodeActionKind::Click]))?;
    let called = Cell::new(false);

    let adapter_result = dispatch_action(&authorized, &current, &called, false)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        adapter_result,
        Ok((NodeActionKind::Click, ActionKind::Navigate))
    );
    assert!(called.get());
    Ok(())
}

#[test]
fn newly_disabled_node_never_reaches_dispatch() -> Result<(), String> {
    let authorized = authorized_action()?;
    let current = observation(17, false, BTreeSet::from([NodeActionKind::Click]))?;
    let called = Cell::new(false);

    let error = dispatch_action(&authorized, &current, &called, false)
        .err()
        .ok_or_else(|| "disabled current observation unexpectedly dispatched".to_owned())?;

    assert_eq!(error, SemanticNodeActionTargetError::NodeNotEnabled);
    assert!(!called.get());
    Ok(())
}

#[test]
fn removed_action_never_reaches_dispatch() -> Result<(), String> {
    let authorized = authorized_action()?;
    let current = observation(17, true, BTreeSet::from([NodeActionKind::ScrollIntoView]))?;
    let called = Cell::new(false);

    let error = dispatch_action(&authorized, &current, &called, false)
        .err()
        .ok_or_else(|| "removed semantic action unexpectedly dispatched".to_owned())?;

    assert_eq!(error, SemanticNodeActionTargetError::UnsupportedAction);
    assert!(!called.get());
    Ok(())
}

#[test]
fn different_same_document_node_never_reaches_dispatch() -> Result<(), String> {
    let authorized = authorized_action()?;
    let current = observation(18, true, BTreeSet::from([NodeActionKind::Click]))?;
    let called = Cell::new(false);

    let error = dispatch_action(&authorized, &current, &called, false)
        .err()
        .ok_or_else(|| "different semantic node unexpectedly dispatched".to_owned())?;

    assert_eq!(
        error,
        SemanticNodeActionTargetError::ObservationAuthorityMismatch
    );
    assert!(!called.get());
    Ok(())
}

#[test]
fn adapter_failure_remains_separate_after_semantic_revalidation() -> Result<(), String> {
    let authorized = authorized_action()?;
    let current = observation(17, true, BTreeSet::from([NodeActionKind::Click]))?;
    let called = Cell::new(false);

    let adapter_result =
        dispatch_action(&authorized, &current, &called, true).map_err(|error| error.to_string())?;

    assert_eq!(adapter_result, Err("adapter failed"));
    assert!(called.get());
    Ok(())
}
