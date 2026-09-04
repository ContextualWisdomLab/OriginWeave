use std::{cell::Cell, collections::BTreeSet, error::Error};

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AdmittedNodeAuthorityError, ApprovalEvidence,
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserRegistryError, BrowsingContextId, ExecutionPurpose, InstructionSource, NodeActionKind,
    ObservationChannel, ObservedNodeHandle, Origin, OriginWeaveProtocolVersion, PolicyContext,
    RobotsDecision, SecretDelivery, SemanticNodeActionBinding, SemanticNodeActionTargetError,
    SemanticNodeObservation, SemanticNodeObservationInput, SessionMode,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
};
use originweave_policy::{PolicyAuthorizedSemanticNodeAction, SemanticNodeDispatchValidationError};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";
const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct AuthorizedFixture {
    registry: BrowserAuthorityRegistry,
    context: BrowsingContextId,
    handle: ObservedNodeHandle,
    other_handle: ObservedNodeHandle,
    authorized: PolicyAuthorizedSemanticNodeAction,
}

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

fn observation(
    registry: &BrowserAuthorityRegistry,
    handle: ObservedNodeHandle,
    enabled: bool,
    supported_actions: BTreeSet<NodeActionKind>,
) -> Result<SemanticNodeObservation, Box<dyn Error>> {
    Ok(SemanticNodeObservation::new(
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
    )?)
}

fn authorized_action() -> Result<AuthorizedFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-dispatch-current-observation-session")?;
    let context =
        registry.register_context(session, "semantic-dispatch-current-observation-context")?;
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
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Continue"), 2)?;
    let command = WebDriverBiDiLocateNodesCommand::new(
        91,
        "semantic-dispatch-current-observation-context",
        &query,
    )?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":91,"result":{"nodes":[{"type":"node","sharedId":"semantic-dispatch-current-observation-node"},{"type":"node","sharedId":"semantic-dispatch-current-observation-other-node"}]}}"#,
    )?;
    let mut handles = command
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter();
    let admitted = handles
        .next()
        .ok_or("locateNodes fixture did not bind its primary node")?;
    let other_admitted = handles
        .next()
        .ok_or("locateNodes fixture did not bind its comparison node")?;
    let handle = (*admitted).clone();
    let other_handle = (*other_admitted).clone();

    let request = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT)
            .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?,
    );
    let binding = SemanticNodeActionBinding::new(admitted, NodeActionKind::Click, request)?;
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
) -> Result<Result<(NodeActionKind, ActionKind), &'static str>, SemanticNodeDispatchValidationError>
{
    authorized.dispatch_if_current_observation(registry, current, |binding| {
        called.set(true);
        if adapter_should_fail {
            Err("adapter failed")
        } else {
            Ok((binding.node_action(), binding.request().action()))
        }
    })
}

#[test]
fn exact_current_browser_and_semantic_authority_reaches_dispatch() -> Result<(), Box<dyn Error>> {
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
    )?;

    assert_eq!(
        adapter_result,
        Ok((NodeActionKind::Click, ActionKind::Navigate))
    );
    assert!(called.get());
    Ok(())
}

#[test]
fn stale_registry_authority_never_reaches_semantic_dispatch() -> Result<(), Box<dyn Error>> {
    let mut fixture = authorized_action()?;
    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    fixture.registry.advance_document(fixture.context)?;
    let called = Cell::new(false);

    let error = dispatch_action(
        &fixture.authorized,
        &fixture.registry,
        &current,
        &called,
        false,
    )
    .err()
    .ok_or("stale browser authority unexpectedly dispatched")?;

    assert!(matches!(
        error,
        SemanticNodeDispatchValidationError::BrowserAuthority(
            AdmittedNodeAuthorityError::BrowserAuthority(
                BrowserRegistryError::ContextOriginNotBound
            )
        )
    ));
    assert!(!called.get());
    Ok(())
}

#[test]
fn newly_disabled_node_never_reaches_dispatch() -> Result<(), Box<dyn Error>> {
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
    .ok_or("disabled current observation unexpectedly dispatched")?;

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
fn removed_action_never_reaches_dispatch() -> Result<(), Box<dyn Error>> {
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
    .ok_or("removed semantic action unexpectedly dispatched")?;

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
fn different_same_document_node_never_reaches_dispatch() -> Result<(), Box<dyn Error>> {
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
    .ok_or("different semantic node unexpectedly dispatched")?;

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
fn adapter_failure_remains_separate_after_both_revalidations() -> Result<(), Box<dyn Error>> {
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
    )?;

    assert_eq!(adapter_result, Err("adapter failed"));
    assert!(called.get());
    Ok(())
}

#[test]
fn dispatch_validation_errors_preserve_typed_sources() {
    let browser = SemanticNodeDispatchValidationError::BrowserAuthority(
        AdmittedNodeAuthorityError::BrowserAuthority(BrowserRegistryError::ContextOriginNotBound),
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
