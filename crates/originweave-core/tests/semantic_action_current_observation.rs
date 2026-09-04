use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, NodeActionKind, ObservationChannel,
    ObservedNodeHandle, Origin, OriginWeaveProtocolVersion, SemanticNodeActionTarget,
    SemanticNodeActionTargetError, SemanticNodeObservation, SemanticNodeObservationInput,
    WebDriverBiDiAccessibilityQuery,
};

const PROTOCOL_VERSION: OriginWeaveProtocolVersion = OriginWeaveProtocolVersion::new(0, 1);

struct ObservationAuthorityFixture {
    registry: BrowserAuthorityRegistry,
    handle: ObservedNodeHandle,
    other_handle: ObservedNodeHandle,
}

fn authority_fixture() -> Result<ObservationAuthorityFixture, String> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry
        .register_session("current-observation-session")
        .map_err(|error| error.to_string())?;
    let context = registry
        .register_context(session, "current-observation-context")
        .map_err(|error| error.to_string())?;
    let origin = Origin::parse("https://app.example").map_err(|error| format!("{error:?}"))?;
    let epoch = registry
        .bind_context_origin(session, context, &origin)
        .map_err(|error| error.to_string())?;
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        PROTOCOL_VERSION,
        "originweave-bidi-v1",
        "webdriver-bidi-wd-2026-06-01",
        "chromium-r1639810",
        &[BrowserProtocolCapability::SemanticObservation],
    )
    .map_err(|error| error.to_string())?;
    let proof = descriptor
        .validate_use(
            PROTOCOL_VERSION,
            BrowserProtocolKind::WebDriverBiDi,
            "originweave-bidi-v1",
            "webdriver-bidi-wd-2026-06-01",
            "chromium-r1639810",
            BrowserProtocolCapability::SemanticObservation,
        )
        .map_err(|error| error.to_string())?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &origin,
        ),
        epoch,
    );
    let mut handles = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 2)
        .map_err(|error| error.to_string())?
        .bind_current_nodes(
            proof,
            &mut registry,
            target,
            &[
                ("node", Some("current-observation-node")),
                ("node", Some("other-current-observation-node")),
            ],
        )
        .map_err(|error| error.to_string())?
        .into_iter();
    let handle = handles
        .next()
        .ok_or_else(|| "current observation fixture returned no target node".to_owned())?;
    let other_handle = handles
        .next()
        .ok_or_else(|| "current observation fixture returned no comparison node".to_owned())?;
    Ok(ObservationAuthorityFixture {
        registry,
        handle,
        other_handle,
    })
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

#[test]
fn current_semantic_observation_revalidates_exact_target_action_state() -> Result<(), String> {
    let fixture = authority_fixture()?;
    let initial = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    let target = SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::Click)
        .map_err(|error| error.to_string())?;

    let current = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;

    let disabled = observation(
        &fixture.registry,
        fixture.handle.clone(),
        false,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    assert_eq!(
        target.validate_current_observation(&disabled),
        Err(SemanticNodeActionTargetError::NodeNotEnabled)
    );

    let action_removed = observation(
        &fixture.registry,
        fixture.handle.clone(),
        true,
        BTreeSet::new(),
    )?;
    assert_eq!(
        target.validate_current_observation(&action_removed),
        Err(SemanticNodeActionTargetError::UnsupportedAction)
    );

    let other_node = observation(
        &fixture.registry,
        fixture.other_handle,
        true,
        BTreeSet::from([NodeActionKind::Click]),
    )?;
    assert_eq!(
        target.validate_current_observation(&other_node),
        Err(SemanticNodeActionTargetError::ObservationAuthorityMismatch)
    );
    Ok(())
}

#[test]
fn scroll_revalidation_preserves_non_enabled_scroll_boundary() -> Result<(), String> {
    let fixture = authority_fixture()?;
    let initial = observation(
        &fixture.registry,
        fixture.handle.clone(),
        false,
        BTreeSet::from([NodeActionKind::ScrollIntoView]),
    )?;
    let target =
        SemanticNodeActionTarget::from_observation(&initial, NodeActionKind::ScrollIntoView)
            .map_err(|error| error.to_string())?;
    let current = observation(
        &fixture.registry,
        fixture.handle,
        false,
        BTreeSet::from([NodeActionKind::ScrollIntoView]),
    )?;

    target
        .validate_current_observation(&current)
        .map_err(|error| error.to_string())?;
    Ok(())
}
