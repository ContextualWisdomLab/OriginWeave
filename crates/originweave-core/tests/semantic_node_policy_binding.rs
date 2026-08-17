use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, BrowserSessionId, BrowsingContextId,
    DocumentEpoch, InstructionSource, NodeActionKind, ObservationChannel, ObservedNodeHandle,
    Origin, SecretDelivery, SemanticNodeActionTarget, SemanticNodeObservation,
    SemanticNodeObservationInput, SemanticNodePolicyBinding, SemanticNodePolicyBindingError,
};

fn node_target() -> Result<SemanticNodeActionTarget, String> {
    let handle = ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())?;
    let observation = SemanticNodeObservation::new(SemanticNodeObservationInput {
        handle,
        parent: None,
        children: Vec::new(),
        role: "button".to_owned(),
        accessible_name: "Submit request".to_owned(),
        visible_text: Some("Submit request".to_owned()),
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click]),
        evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
    })
    .map_err(|error| error.to_string())?;
    SemanticNodeActionTarget::from_observation(&observation, NodeActionKind::Click)
        .map_err(|error| error.to_string())
}

fn request(source: &str, target: &str) -> Result<ActionRequest, String> {
    Ok(ActionRequest::new(
        ActionKind::Submit,
        Origin::parse(source).map_err(|error| format!("{error:?}"))?,
        Origin::parse(target).map_err(|error| format!("{error:?}"))?,
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .map_err(|error| format!("{error:?}"))?,
    ))
}

#[test]
fn semantic_target_binds_to_explicit_same_origin_policy_request() -> Result<(), String> {
    let target = node_target()?;
    let request = request("https://example.com", "https://example.com")?;
    let binding = SemanticNodePolicyBinding::new(target.clone(), request.clone())
        .map_err(|error| error.to_string())?;

    assert_eq!(binding.target(), &target);
    assert_eq!(binding.request(), &request);
    Ok(())
}

#[test]
fn semantic_target_rejects_policy_request_from_another_browser_origin() -> Result<(), String> {
    let target = node_target()?;
    let request = request("https://other.example", "https://example.com")?;

    assert_eq!(
        SemanticNodePolicyBinding::new(target, request).err(),
        Some(SemanticNodePolicyBindingError::SourceOriginMismatch)
    );
    Ok(())
}

#[test]
fn semantic_target_rejects_policy_request_for_another_target_origin() -> Result<(), String> {
    let target = node_target()?;
    let request = request("https://example.com", "https://other.example")?;

    assert_eq!(
        SemanticNodePolicyBinding::new(target, request).err(),
        Some(SemanticNodePolicyBindingError::TargetOriginMismatch)
    );
    Ok(())
}

#[test]
fn semantic_policy_binding_errors_are_stable_and_credential_free() {
    assert_eq!(
        SemanticNodePolicyBindingError::SourceOriginMismatch.to_string(),
        "semantic node origin does not match the policy request source origin"
    );
    assert_eq!(
        SemanticNodePolicyBindingError::TargetOriginMismatch.to_string(),
        "semantic node origin does not match the policy request target origin"
    );
}
