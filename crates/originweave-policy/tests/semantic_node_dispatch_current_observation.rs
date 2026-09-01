use std::cell::Cell;
use std::collections::BTreeSet;
use std::error::Error;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, BrowserAuthorityRegistry,
    BrowserRegistryError, BrowsingContextId, ExecutionPurpose, InstructionSource, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SemanticNodeActionTarget, SemanticNodeActionTargetError,
    SemanticNodeObservation, SemanticNodeObservationInput, SessionMode,
};
use originweave_policy::{
    PolicyAuthorizedSemanticNodeAction, SemanticNodeDispatchValidationError,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct AuthorizedFixture {
    registry: BrowserAuthorityRegistry,
    context: BrowsingContextId,
    handle: ObservedNodeHandle,
    other_handle: ObservedNodeHandle,
    authorized: PolicyAuthorizedSemanticNodeAction,
}

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn observation(
    registry: &BrowserAuthorityRegistry,
    handle: ObservedNodeHandle,
    enabled: bool,
    supported_actions: BTreeSet<NodeActionKind>,
) -> Result<SemanticNodeObservation, String> {
    SemanticNodeObservation::new(
        SemanticNodeObservationInput {
            handle,
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
        },
        registry,
    )
    .map_err(|error| error.to_string())
}

fn authorized_action() -> Result<AuthorizedFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-dispatch-current-observation-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-dispatch-current-observation-context")
        .map_err(|error| error.to_string())?;
    let site = origin("https://app.example")?;
    let handle = registry
        .bind_node(session, context, &site, "semantic-dispatch-current-observation-node")
        .map_err(|error| error.to_string())?;
    let other_handle = registry
        .bind_node(session, context, &site, "semantic-dispatch-current-observation-other-node")
        .map_err(|error| error.to_string())?;
    let initial_observation = observation(
        &registry,
        handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
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
    let context_policy = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([ActionKind::Navigate.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &context_policy)
        .map_err(|error| error.to_string())?;

    Ok(AuthorizedFixture {
        registry,
        context,
        handle,
        other_handle,
        authorized,
    })
}

fn dispatch_action(
    authorized: &PolicyAuthorizedSemanticNodeAction,
    registry: &BrowserAuthorityRegistry,
    current: &SemanticNodeObservation,
    called: &Cell<bool>,
    adapter_should_fail: bool,
) -> Result<Result<(NodeActionKind, ActionKind), &'static str>, SemanticNodeDispatchValidationError> {
    authorized.dispatch_if_current_observation(registry, current, |binding| {
        called.set(true);
        if adapter_should_fail {
            Err("adapter failed")
        } else {
            Ok((binding.target().action(), binding.request().action()))
        }
    })
}

#[test]
fn exact_current_browser_and_semantic_authority_reaches_dispatch() -> Result<(), String> {
    let fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let called = Cell::new(false);

    let adapter_result = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        adapter_result,
        Ok((NodeActionKind::Click, ActionKind::Navigate))
    );
    assert!(called.get());
    Ok(())
}

#[test]
fn stale_registry_authority_never_reaches_semantic_dispatch() -> Result<(), String> {
    let mut fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    fixture
        .registry
        .advance_document(fixture.context)
        .map_err(|error| error.to_string())?;
    let called = Cell::new(false);

    let error = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .err()
    .ok_or_else(|| "stale browser authority unexpectedly dispatched".to_owned())?;

    assert!(matches!(
        error,
        SemanticNodeDispatchValidationError::BrowserAuthority(
            BrowserRegistryError::UnknownNodeAuthority
        )
    ));
    assert!(!called.get());
    Ok(())
}

#[test]
fn newly_disabled_node_never_reaches_dispatch() -> Result<(), String> {
    let fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        false,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let called = Cell::new(false);

    let error = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .err()
    .ok_or_else(|| "disabled current observation unexpectedly dispatched".to_owned())?;

    assert!(matches!(
        error,
        SemanticNodeDispatchValidationError::SemanticState(
            SemanticNodeActionTargetError::NodeNotEnabled
        )
    ));
    assert!(!called.get());
    Ok(())
}

#[test]
fn removed_action_never_reaches_dispatch() -> Result<(), String> {
    let fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::ScrollIntoView]),
    )?;
    let called = Cell::new(false);

    let error = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .err()
    .ok_or_else(|| "removed semantic action unexpectedly dispatched".to_owned())?;

    assert!(matches!(
        error,
        SemanticNodeDispatchValidationError::SemanticState(
            SemanticNodeActionTargetError::UnsupportedAction
        )
    ));
    assert!(!called.get());
    Ok(())
}

#[test]
fn different_same_document_node_never_reaches_dispatch() -> Result<(), String> {
    let fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.other_handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let called = Cell::new(false);

    let error = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .err()
    .ok_or_else(|| "different semantic node unexpectedly dispatched".to_owned())?;

    assert!(matches!(
        error,
        SemanticNodeDispatchValidationError::SemanticState(
            SemanticNodeActionTargetError::ObservationAuthorityMismatch
        )
    ));
    assert!(!called.get());
    Ok(())
}

#[test]
fn adapter_failure_remains_separate_after_both_revalidations() -> Result<(), String> {
    let fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let called = Cell::new(false);

    let adapter_result = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        true,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(adapter_result, Err("adapter failed"));
    assert!(called.get());
    Ok(())
}

#[test]
fn dispatch_validation_errors_preserve_typed_sources() {
    let browser = SemanticNodeDispatchValidationError::BrowserAuthority(
        BrowserRegistryError::UnknownNodeAuthority,
    );
    assert_eq!(
        browser.to_string(),
        "semantic node dispatch browser authority revalidation failed"
    );
    assert!(browser.source().is_some());

    let semantic = SemanticNodeDispatchValidationError::SemanticState(
        SemanticNodeActionTargetError::UnsupportedAction,
    );
    assert_eq!(
        semantic.to_string(),
        "semantic node dispatch semantic-state revalidation failed"
    );
    assert!(semantic.source().is_some());
}
