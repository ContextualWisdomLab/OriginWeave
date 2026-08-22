use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, BrowserAuthorityRegistry, BrowserRegistryError,
    BrowserSessionId, BrowsingContextId, InstructionSource, NodeActionKind, ObservationChannel,
    Origin, SecretDelivery, SemanticNodeActionBinding, SemanticNodeActionBindingError,
    SemanticNodeActionTarget, SemanticNodeObservation, SemanticNodeObservationInput,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct ObservationFixture {
    registry: BrowserAuthorityRegistry,
    session: BrowserSessionId,
    context: BrowsingContextId,
    observation: SemanticNodeObservation,
}

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn observation_fixture() -> Result<ObservationFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("semantic-binding-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "semantic-binding-context")
        .map_err(|error| error.to_string())?;
    let source_origin = origin("https://app.example")?;
    let handle = registry
        .bind_node(session, context, &source_origin, "semantic-binding-node")
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

    Ok(ObservationFixture {
        registry,
        session,
        context,
        observation,
    })
}

fn action_request(source: Origin, target: Origin) -> Result<ActionRequest, String> {
    let intent = ActionIntentDigest::parse(VALID_INTENT).map_err(|error| format!("{error:?}"))?;
    Ok(ActionRequest::new(
        ActionKind::Navigate,
        source,
        target,
        InstructionSource::User,
        SecretDelivery::None,
        intent,
    ))
}

#[test]
fn node_action_binding_preserves_node_target_and_business_request() -> Result<(), String> {
    let fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://app.example")?,
        origin("https://next.example")?,
    )?;

    let binding = SemanticNodeActionBinding::new(target.clone(), request.clone())
        .map_err(|error| error.to_string())?;

    assert_eq!(binding.target(), &target);
    assert_eq!(binding.request(), &request);
    Ok(())
}

#[test]
fn node_action_binding_rejects_request_from_another_document_origin() -> Result<(), String> {
    let fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://other.example")?,
        origin("https://next.example")?,
    )?;

    assert_eq!(
        SemanticNodeActionBinding::new(target, request).err(),
        Some(SemanticNodeActionBindingError::SourceOriginMismatch)
    );
    Ok(())
}

#[test]
fn node_action_binding_does_not_conflate_source_node_with_navigation_target() -> Result<(), String>
{
    let fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let destination = origin("https://destination.example")?;
    let request = action_request(origin("https://app.example")?, destination.clone())?;

    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;

    assert_eq!(binding.request().target_origin(), &destination);
    Ok(())
}

#[test]
fn node_action_binding_revalidates_registry_owned_authority_before_dispatch() -> Result<(), String> {
    let fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://app.example")?,
        origin("https://next.example")?,
    )?;
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;

    binding
        .validate_current(&fixture.registry)
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn node_action_binding_rejects_stale_document_before_dispatch() -> Result<(), String> {
    let mut fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://app.example")?,
        origin("https://next.example")?,
    )?;
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;
    fixture
        .registry
        .advance_document(fixture.context)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        binding.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn node_action_binding_rejects_retired_session_before_dispatch() -> Result<(), String> {
    let mut fixture = observation_fixture()?;
    let target = SemanticNodeActionTarget::from_observation(&fixture.observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://app.example")?,
        origin("https://next.example")?,
    )?;
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;
    fixture
        .registry
        .remove_session(fixture.session)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        binding.validate_current(&fixture.registry).err(),
        Some(BrowserRegistryError::UnknownBrowserSession)
    );
    Ok(())
}

#[test]
fn node_action_binding_error_is_stable_and_credential_free() {
    assert_eq!(
        SemanticNodeActionBindingError::SourceOriginMismatch.to_string(),
        "semantic node origin does not match action request source origin"
    );
}
