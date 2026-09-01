use std::cell::Cell;
use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserAuthorityRegistry,
    BrowserRegistryError, BrowsingContextId, ExecutionPurpose, InstructionSource, NodeActionKind,
    ObservationChannel, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SemanticNodeActionTarget, SemanticNodeObservation,
    SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::PolicyAuthorizedSemanticNodeAction;

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct AuthorizedFixture {
    registry: BrowserAuthorityRegistry,
    context: BrowsingContextId,
    authorized: PolicyAuthorizedSemanticNodeAction,
}

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn authorized_action() -> Result<AuthorizedFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-dispatch-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-dispatch-context")
        .map_err(|error| error.to_string())?;
    let site = origin("https://app.example")?;
    let handle = registry
        .bind_node(session, context, &site, "semantic-dispatch-node")
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
        ActionKind::Navigate,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT).map_err(|error| format!("{error:?}"))?,
    );
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;
    let policy_context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([ActionKind::Navigate.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &policy_context)
        .map_err(|error| error.to_string())?;

    Ok(AuthorizedFixture {
        registry,
        context,
        authorized,
    })
}

fn dispatch_unit_callback(
    authorized: &PolicyAuthorizedSemanticNodeAction,
    registry: &BrowserAuthorityRegistry,
    called: &Cell<bool>,
) -> Result<(), BrowserRegistryError> {
    authorized.dispatch_if_current(registry, |_binding| called.set(true))
}

#[test]
fn dispatch_callback_runs_only_after_registry_owned_browser_revalidation() -> Result<(), String> {
    let fixture = authorized_action()?;
    let called = Cell::new(false);

    let result = fixture
        .authorized
        .dispatch_if_current(&fixture.registry, |binding| {
            called.set(true);
            (binding.target().action(), binding.request().action())
        })
        .map_err(|error| error.to_string())?;

    assert!(called.get());
    assert_eq!(result, (NodeActionKind::Click, ActionKind::Navigate));
    Ok(())
}

#[test]
fn stale_registry_authority_never_reaches_dispatch_callback() -> Result<(), String> {
    let mut fixture = authorized_action()?;
    let called = Cell::new(false);

    dispatch_unit_callback(&fixture.authorized, &fixture.registry, &called)
        .map_err(|error| error.to_string())?;
    assert!(called.replace(false));

    fixture
        .registry
        .advance_document(fixture.context)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        fixture
            .authorized
            .dispatch_if_current(&fixture.registry, |_binding| called.set(true))
            .err(),
        Some(BrowserRegistryError::UnknownNodeAuthority)
    );
    assert!(!called.get());
    Ok(())
}

#[test]
fn adapter_failure_remains_separate_after_successful_revalidation() -> Result<(), String> {
    let fixture = authorized_action()?;

    let adapter_result = fixture
        .authorized
        .dispatch_if_current(&fixture.registry, |_binding| -> Result<(), &'static str> {
            Err("adapter failed")
        })
        .map_err(|error| error.to_string())?;

    assert_eq!(adapter_result, Err("adapter failed"));
    Ok(())
}
