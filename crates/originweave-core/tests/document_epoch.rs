#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError, ObservedNodeHandle, Origin,
};

fn assert_standard_error<T: std::error::Error>() {}

#[test]
fn sessions_contexts_epochs_and_node_handles_reject_invalid_identifiers() {
    assert_eq!(
        BrowserSessionId::new(0),
        Err(NodeHandleError::InvalidBrowserSessionId)
    );
    assert_eq!(
        BrowsingContextId::new(0),
        Err(NodeHandleError::InvalidBrowsingContextId)
    );
    assert_eq!(
        DocumentEpoch::new(0),
        Err(NodeHandleError::InvalidDocumentEpoch)
    );

    let browser_session = BrowserSessionId::new(3).expect("nonzero browser session");
    let browsing_context = BrowsingContextId::new(11).expect("nonzero browsing context");
    let epoch = DocumentEpoch::new(7).expect("nonzero document epoch");
    let origin = Origin::parse("https://example.com").expect("canonical origin");
    assert_eq!(
        ObservedNodeHandle::new(browser_session, browsing_context, origin, epoch, 0),
        Err(NodeHandleError::InvalidNodeId)
    );
}

#[test]
fn a_node_handle_is_valid_only_for_its_exact_session_context_origin_and_document_epoch() {
    let origin = Origin::parse("https://example.com").expect("canonical origin");
    let other_origin = Origin::parse("https://other.example").expect("canonical origin");
    let observed_session = BrowserSessionId::new(3).expect("browser session");
    let other_session = BrowserSessionId::new(4).expect("browser session");
    let observed_context = BrowsingContextId::new(11).expect("browsing context");
    let other_context = BrowsingContextId::new(12).expect("browsing context");
    let observed_epoch = DocumentEpoch::new(7).expect("document epoch");
    let current_epoch = DocumentEpoch::new(8).expect("document epoch");
    let handle = ObservedNodeHandle::new(
        observed_session,
        observed_context,
        origin.clone(),
        observed_epoch,
        42,
    )
    .expect("valid observed node handle");

    assert_eq!(handle.browser_session(), observed_session);
    assert_eq!(handle.browsing_context(), observed_context);
    assert_eq!(handle.origin(), &origin);
    assert_eq!(handle.document_epoch(), observed_epoch);
    assert_eq!(handle.node_id(), 42);
    assert_eq!(observed_session.value(), 3);
    assert_eq!(observed_context.value(), 11);
    assert_eq!(observed_epoch.value(), 7);
    assert_eq!(
        handle.validate_current(observed_session, observed_context, &origin, observed_epoch,),
        Ok(())
    );
    assert_eq!(
        handle.validate_current(other_session, observed_context, &origin, observed_epoch,),
        Err(NodeHandleError::BrowserSessionMismatch {
            observed: observed_session,
            current: other_session,
        })
    );
    assert_eq!(
        handle.validate_current(observed_session, other_context, &origin, observed_epoch,),
        Err(NodeHandleError::BrowsingContextMismatch {
            observed: observed_context,
            current: other_context,
        })
    );
    assert_eq!(
        handle.validate_current(observed_session, observed_context, &origin, current_epoch,),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: observed_epoch,
            current: current_epoch,
        })
    );
    assert_eq!(
        handle.validate_current(
            observed_session,
            observed_context,
            &other_origin,
            observed_epoch,
        ),
        Err(NodeHandleError::OriginMismatch)
    );
}

#[test]
fn node_handle_errors_are_standard_and_deterministic_for_adapters() {
    assert_standard_error::<NodeHandleError>();

    let observed_session = BrowserSessionId::new(3).expect("browser session");
    let current_session = BrowserSessionId::new(4).expect("browser session");
    let observed_context = BrowsingContextId::new(11).expect("browsing context");
    let current_context = BrowsingContextId::new(12).expect("browsing context");
    let observed_epoch = DocumentEpoch::new(7).expect("document epoch");
    let current_epoch = DocumentEpoch::new(8).expect("document epoch");
    let cases = [
        (
            NodeHandleError::InvalidBrowserSessionId,
            "browser session identifier must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::InvalidBrowsingContextId,
            "browsing context identifier must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::InvalidDocumentEpoch,
            "document epoch must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::InvalidNodeId,
            "observed node identifier must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::BrowserSessionMismatch {
                observed: observed_session,
                current: current_session,
            },
            "observed node browser session 3 does not match current session 4".to_owned(),
        ),
        (
            NodeHandleError::BrowsingContextMismatch {
                observed: observed_context,
                current: current_context,
            },
            "observed node browsing context 11 does not match current context 12".to_owned(),
        ),
        (
            NodeHandleError::OriginMismatch,
            "observed node origin does not match the current origin".to_owned(),
        ),
        (
            NodeHandleError::StaleDocumentEpoch {
                observed: observed_epoch,
                current: current_epoch,
            },
            "observed node document epoch 7 is stale; current epoch is 8".to_owned(),
        ),
        (
            NodeHandleError::DocumentEpochOverflow,
            "document epoch cannot wrap after a same-document mutation".to_owned(),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        let standard: &dyn std::error::Error = &error;
        assert!(standard.source().is_none());
    }
}
