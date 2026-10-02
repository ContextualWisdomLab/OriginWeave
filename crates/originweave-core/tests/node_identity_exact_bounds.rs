#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin,
};

#[test]
fn one_is_preserved_in_every_observed_node_identity_component() {
    let session = BrowserSessionId::new(1).expect("one is a valid session identity");
    let context = BrowsingContextId::new(1).expect("one is a valid context identity");
    let epoch = DocumentEpoch::new(1).expect("one is a valid document identity");
    let origin = Origin::parse("https://minimum.example").expect("canonical origin");
    let handle = ObservedNodeHandle::new(session, context, origin.clone(), epoch, 1)
        .expect("minimum nonzero node identity must be admitted");

    assert_eq!(session.value(), 1);
    assert_eq!(context.value(), 1);
    assert_eq!(epoch.value(), 1);
    assert_eq!(handle.browser_session().value(), 1);
    assert_eq!(handle.browsing_context().value(), 1);
    assert_eq!(handle.document_epoch().value(), 1);
    assert_eq!(handle.node_id(), 1);
    assert_eq!(handle.origin(), &origin);
    assert_eq!(
        handle.validate_current(session, context, &origin, epoch),
        Ok(())
    );
}

#[test]
fn maximum_u64_is_preserved_in_every_observed_node_identity_component() {
    let session = BrowserSessionId::new(u64::MAX).expect("maximum session identity is nonzero");
    let context = BrowsingContextId::new(u64::MAX).expect("maximum context identity is nonzero");
    let epoch = DocumentEpoch::new(u64::MAX).expect("maximum document identity is nonzero");
    let origin = Origin::parse("https://maximum.example").expect("canonical origin");
    let handle = ObservedNodeHandle::new(session, context, origin.clone(), epoch, u64::MAX)
        .expect("maximum nonzero node identity must be admitted");

    assert_eq!(session.value(), u64::MAX);
    assert_eq!(context.value(), u64::MAX);
    assert_eq!(epoch.value(), u64::MAX);
    assert_eq!(handle.browser_session().value(), u64::MAX);
    assert_eq!(handle.browsing_context().value(), u64::MAX);
    assert_eq!(handle.document_epoch().value(), u64::MAX);
    assert_eq!(handle.node_id(), u64::MAX);
    assert_eq!(handle.origin(), &origin);
    assert_eq!(
        handle.validate_current(session, context, &origin, epoch),
        Ok(())
    );
}
