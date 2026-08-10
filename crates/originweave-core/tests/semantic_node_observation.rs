use std::collections::BTreeSet;
use std::error::Error;

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeActionKind, ObservationChannel,
    ObservedNodeHandle, Origin, SemanticNodeObservation, SemanticNodeObservationInput,
};

fn observed_node() -> Result<ObservedNodeHandle, Box<dyn Error>> {
    Ok(ObservedNodeHandle::new(
        BrowserSessionId::new(7)?,
        BrowsingContextId::new(11)?,
        Origin::parse("https://example.com")?,
        DocumentEpoch::new(3)?,
        17,
    )?)
}

#[test]
fn semantic_node_preserves_authority_and_bounded_surface() -> Result<(), Box<dyn Error>> {
    let handle = observed_node()?;
    let observation = SemanticNodeObservation::new(SemanticNodeObservationInput {
        handle: handle.clone(),
        role: "textbox".to_owned(),
        accessible_name: "Email address".to_owned(),
        visible_text: Some("name@example.test".to_owned()),
        enabled: true,
        visible: true,
        selected: None,
        supported_actions: BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText]),
        evidence_channels: BTreeSet::from([
            ObservationChannel::Accessibility,
            ObservationChannel::Dom,
        ]),
    })?;

    assert_eq!(observation.handle(), &handle);
    assert_eq!(observation.role(), "textbox");
    assert_eq!(observation.accessible_name(), "Email address");
    assert_eq!(observation.visible_text(), Some("name@example.test"));
    assert!(observation.is_enabled());
    assert!(observation.is_visible());
    assert_eq!(observation.is_selected(), None);
    assert_eq!(
        observation.supported_actions(),
        &BTreeSet::from([NodeActionKind::Click, NodeActionKind::TypeText])
    );
    assert_eq!(
        observation.evidence_channels(),
        &BTreeSet::from([ObservationChannel::Accessibility, ObservationChannel::Dom,])
    );
    Ok(())
}
