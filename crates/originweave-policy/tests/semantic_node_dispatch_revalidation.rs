use std::cell::Cell;
use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, ExecutionPurpose, InstructionSource, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SemanticNodeActionTarget, SemanticNodeObservation,
    SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::PolicyAuthorizedSemanticNodeAction;

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn authorized_action() -> Result<PolicyAuthorizedSemanticNodeAction, String> {
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

fn dispatch_unit_callback(
    authorized: &PolicyAuthorizedSemanticNodeAction,
    document_epoch: u64,
    called: &Cell<bool>,
) -> Result<(), String> {
    authorized
        .dispatch_if_current(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            &origin("https://app.example")?,
            DocumentEpoch::new(document_epoch).map_err(|error| error.to_string())?,
            |_binding| called.set(true),
        )
        .map_err(|error| error.to_string())
}

#[test]
fn dispatch_callback_runs_only_after_exact_browser_revalidation() -> Result<(), String> {
    let authorized = authorized_action()?;
    let called = Cell::new(false);

    let result = authorized
        .dispatch_if_current(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            &origin("https://app.example")?,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
            |binding| {
                called.set(true);
                (binding.target().action(), binding.request().action())
            },
        )
        .map_err(|error| error.to_string())?;

    assert!(called.get());
    assert_eq!(result, (NodeActionKind::Click, ActionKind::Navigate));
    Ok(())
}

#[test]
fn stale_browser_authority_never_reaches_dispatch_callback() -> Result<(), String> {
    let authorized = authorized_action()?;
    let called = Cell::new(false);

    dispatch_unit_callback(&authorized, 3, &called)?;
    assert!(called.replace(false));

    let error = dispatch_unit_callback(&authorized, 4, &called)
        .err()
        .ok_or_else(|| "stale browser authority unexpectedly reached dispatch".to_owned())?;

    assert!(!called.get());
    assert!(error.contains("stale"));
    Ok(())
}

#[test]
fn adapter_failure_remains_separate_after_successful_revalidation() -> Result<(), String> {
    let authorized = authorized_action()?;

    let adapter_result = authorized
        .dispatch_if_current(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            &origin("https://app.example")?,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
            |_binding| -> Result<(), &'static str> { Err("adapter failed") },
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(adapter_result, Err("adapter failed"));
    Ok(())
}
