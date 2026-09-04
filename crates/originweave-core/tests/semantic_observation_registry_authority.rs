use std::collections::BTreeSet;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, NodeActionKind, ObservationChannel,
    ObservedNodeHandle, Origin, OriginWeaveProtocolVersion, SemanticNodeObservation,
    SemanticNodeObservationError, SemanticNodeObservationInput, WebDriverBiDiAccessibilityQuery,
};

const PROTOCOL_VERSION: OriginWeaveProtocolVersion = OriginWeaveProtocolVersion::new(0, 1);

fn bound_observation_fixture()
-> Result<(BrowserAuthorityRegistry, ObservedNodeHandle), Box<dyn std::error::Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-session")?;
    let context = registry.register_context(session, "semantic-context")?;
    let origin = Origin::parse("https://example.com").map_err(|error| format!("{error:?}"))?;
    let epoch = registry.bind_context_origin(session, context, &origin)?;
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        PROTOCOL_VERSION,
        "originweave-bidi-v1",
        "webdriver-bidi-wd-2026-06-01",
        "chromium-r1639810",
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    let proof = descriptor.validate_use(
        PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        "originweave-bidi-v1",
        "webdriver-bidi-wd-2026-06-01",
        "chromium-r1639810",
        BrowserProtocolCapability::SemanticObservation,
    )?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &origin,
        ),
        epoch,
    );
    let handle = WebDriverBiDiAccessibilityQuery::new(Some("generic"), None, 1)?
        .bind_current_nodes(
            proof,
            &mut registry,
            target,
            &[("node", Some("semantic-node"))],
        )?
        .into_iter()
        .next()
        .ok_or("semantic observation fixture returned no node")?;
    Ok((registry, handle))
}

fn input(handle: ObservedNodeHandle) -> SemanticNodeObservationInput {
    SemanticNodeObservationInput {
        handle,
        parent: None,
        children: Vec::new(),
        role: "button".to_owned(),
        accessible_name: "Submit".to_owned(),
        visible_text: None,
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click]),
        evidence_channels: BTreeSet::from([ObservationChannel::Accessibility]),
    }
}

#[test]
fn semantic_observation_rejects_forged_primary_node_authority()
-> Result<(), Box<dyn std::error::Error>> {
    let (registry, bound) = bound_observation_fixture()?;
    let forged = ObservedNodeHandle::new(
        bound.browser_session(),
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id() + 10_000,
    )?;

    assert_eq!(
        SemanticNodeObservation::new(input(forged), &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn semantic_observation_rejects_a_context_claimed_by_another_session()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut registry, bound) = bound_observation_fixture()?;
    let other_session = registry.register_session("other-semantic-session")?;
    let forged = ObservedNodeHandle::new(
        other_session,
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id(),
    )?;

    assert_eq!(
        SemanticNodeObservation::new(input(forged), &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn semantic_observation_rejects_forged_parent_node_authority()
-> Result<(), Box<dyn std::error::Error>> {
    let (registry, bound) = bound_observation_fixture()?;
    let forged_parent = ObservedNodeHandle::new(
        bound.browser_session(),
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id() + 10_000,
    )?;
    let mut observation_input = input(bound);
    observation_input.parent = Some(forged_parent);

    assert_eq!(
        SemanticNodeObservation::new(observation_input, &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}

#[test]
fn semantic_observation_rejects_forged_related_node_authority()
-> Result<(), Box<dyn std::error::Error>> {
    let (registry, bound) = bound_observation_fixture()?;
    let forged_child = ObservedNodeHandle::new(
        bound.browser_session(),
        bound.browsing_context(),
        bound.origin().clone(),
        bound.document_epoch(),
        bound.node_id() + 10_000,
    )?;
    let mut observation_input = input(bound);
    observation_input.children.push(forged_child);

    assert_eq!(
        SemanticNodeObservation::new(observation_input, &registry).err(),
        Some(SemanticNodeObservationError::UnknownNodeAuthority)
    );
    Ok(())
}
