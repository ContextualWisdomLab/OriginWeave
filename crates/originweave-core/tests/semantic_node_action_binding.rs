use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, BrowserSessionId, BrowsingContextId,
    DocumentEpoch, InstructionSource, NodeActionKind, NodeHandleError, ObservationChannel,
    ObservedNodeHandle, Origin, SecretDelivery, SemanticNodeActionBinding,
    SemanticNodeActionBindingError, SemanticNodeActionTarget, SemanticNodeObservation,
    SemanticNodeObservationInput,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin(value: &str) -> Result<Origin, String> {
    Origin::parse(value).map_err(|error| format!("{error:?}"))
}

fn observation() -> Result<SemanticNodeObservation, String> {
    let handle = ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        origin("https://app.example")?,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())?;

    SemanticNodeObservation::new(SemanticNodeObservationInput {
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
    .map_err(|error| error.to_string())
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
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
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
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
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
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let destination = origin("https://destination.example")?;
    let request = action_request(origin("https://app.example")?, destination.clone())?;

    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;

    assert_eq!(binding.request().target_origin(), &destination);
    Ok(())
}

#[test]
fn node_action_binding_revalidates_exact_browser_authority_before_dispatch() -> Result<(), String> {
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let request = action_request(
        origin("https://app.example")?,
        origin("https://next.example")?,
    )?;
    let binding =
        SemanticNodeActionBinding::new(target, request).map_err(|error| error.to_string())?;
    let observed_epoch = DocumentEpoch::new(3).map_err(|error| error.to_string())?;
    let current_epoch = DocumentEpoch::new(4).map_err(|error| error.to_string())?;

    assert_eq!(
        binding
            .validate_current(
                BrowserSessionId::new(7).map_err(|error| error.to_string())?,
                BrowsingContextId::new(11).map_err(|error| error.to_string())?,
                &origin("https://app.example")?,
                current_epoch,
            )
            .err(),
        Some(NodeHandleError::StaleDocumentEpoch {
            observed: observed_epoch,
            current: current_epoch,
        })
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
