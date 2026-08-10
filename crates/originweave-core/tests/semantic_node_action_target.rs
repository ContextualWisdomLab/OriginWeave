use std::collections::BTreeSet;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeActionKind, NodeHandleError,
    ObservationChannel, ObservedNodeHandle, Origin, SemanticNodeActionTarget,
    SemanticNodeActionTargetError, SemanticNodeObservation, SemanticNodeObservationInput,
};

fn observation() -> Result<SemanticNodeObservation, String> {
    let handle = ObservedNodeHandle::new(
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        17,
    )
    .map_err(|error| error.to_string())?;

    SemanticNodeObservation::new(SemanticNodeObservationInput {
        handle,
        parent: None,
        children: Vec::new(),
        role: "button".to_owned(),
        accessible_name: "Save draft".to_owned(),
        visible_text: Some("Save draft".to_owned()),
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click]),
        evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
    })
    .map_err(|error| error.to_string())
}

#[test]
fn advertised_node_action_becomes_an_authority_bound_target() -> Result<(), String> {
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;

    assert_eq!(target.handle(), observed.handle());
    assert_eq!(target.action(), NodeActionKind::Click);
    Ok(())
}

#[test]
fn unsupported_node_action_fails_closed_without_minting_authority() -> Result<(), String> {
    let observed = observation()?;
    assert_eq!(
        SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::TypeText).err(),
        Some(SemanticNodeActionTargetError::UnsupportedAction)
    );
    Ok(())
}

#[test]
fn node_action_target_revalidates_exact_browser_authority() -> Result<(), String> {
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let current_origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;

    target
        .validate_current(
            BrowserSessionId::new(7).map_err(|error| error.to_string())?,
            BrowsingContextId::new(11).map_err(|error| error.to_string())?,
            &current_origin,
            DocumentEpoch::new(3).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn node_action_target_rejects_stale_document_authority() -> Result<(), String> {
    let observed = observation()?;
    let target = SemanticNodeActionTarget::from_observation(&observed, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;
    let current_origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let observed_epoch = DocumentEpoch::new(3).map_err(|error| error.to_string())?;
    let current_epoch = DocumentEpoch::new(4).map_err(|error| error.to_string())?;

    assert_eq!(
        target
            .validate_current(
                BrowserSessionId::new(7).map_err(|error| error.to_string())?,
                BrowsingContextId::new(11).map_err(|error| error.to_string())?,
                &current_origin,
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
fn node_action_target_error_is_stable_and_credential_free() {
    assert_eq!(
        SemanticNodeActionTargetError::UnsupportedAction.to_string(),
        "semantic node action is not advertised by the observation"
    );
}
