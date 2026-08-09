#![allow(clippy::expect_used)]

use originweave_core::{DocumentEpoch, NodeHandleError, ObservedNodeHandle, Origin};

fn assert_standard_error<T: std::error::Error>() {}

#[test]
fn document_epochs_and_node_handles_reject_invalid_identifiers() {
    assert_eq!(
        DocumentEpoch::new(0),
        Err(NodeHandleError::InvalidDocumentEpoch)
    );

    let epoch = DocumentEpoch::new(7).expect("nonzero document epoch");
    let origin = Origin::parse("https://example.com").expect("canonical origin");
    assert_eq!(
        ObservedNodeHandle::new(origin, epoch, 0),
        Err(NodeHandleError::InvalidNodeId)
    );
}

#[test]
fn a_node_handle_is_valid_only_for_its_exact_origin_and_document_epoch() {
    let origin = Origin::parse("https://example.com").expect("canonical origin");
    let other_origin = Origin::parse("https://other.example").expect("canonical origin");
    let observed_epoch = DocumentEpoch::new(7).expect("document epoch");
    let current_epoch = DocumentEpoch::new(8).expect("document epoch");
    let handle = ObservedNodeHandle::new(origin.clone(), observed_epoch, 42)
        .expect("valid observed node handle");

    assert_eq!(handle.origin(), &origin);
    assert_eq!(handle.document_epoch(), observed_epoch);
    assert_eq!(handle.node_id(), 42);
    assert_eq!(observed_epoch.value(), 7);
    assert_eq!(handle.validate_current(&origin, observed_epoch), Ok(()));
    assert_eq!(
        handle.validate_current(&origin, current_epoch),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: observed_epoch,
            current: current_epoch,
        })
    );
    assert_eq!(
        handle.validate_current(&other_origin, observed_epoch),
        Err(NodeHandleError::OriginMismatch)
    );
}

#[test]
fn node_handle_errors_are_standard_and_deterministic_for_adapters() {
    assert_standard_error::<NodeHandleError>();

    let observed = DocumentEpoch::new(7).expect("document epoch");
    let current = DocumentEpoch::new(8).expect("document epoch");
    let cases = [
        (
            NodeHandleError::InvalidDocumentEpoch,
            "document epoch must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::InvalidNodeId,
            "observed node identifier must be nonzero".to_owned(),
        ),
        (
            NodeHandleError::OriginMismatch,
            "observed node origin does not match the current origin".to_owned(),
        ),
        (
            NodeHandleError::StaleDocumentEpoch { observed, current },
            "observed node document epoch 7 is stale; current epoch is 8".to_owned(),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        let standard: &dyn std::error::Error = &error;
        assert!(standard.source().is_none());
    }
}
