#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, DocumentEpoch, NodeHandleError, ObservedNodeHandle,
    Origin, SameDocumentMutationKind,
};

fn checkout_field_handle() -> ObservedNodeHandle {
    ObservedNodeHandle::new(
        BrowserSessionId::new(3).expect("browser session"),
        BrowsingContextId::new(11).expect("browsing context"),
        Origin::parse("https://shop.example").expect("canonical origin"),
        DocumentEpoch::new(1).expect("initial document epoch"),
        42,
    )
    .expect("observed checkout field")
}

#[test]
fn a_same_document_target_replacement_invalidates_the_previous_checkout_field_handle() {
    let handle = checkout_field_handle();
    let session = handle.browser_session();
    let context = handle.browsing_context();
    let origin = handle.origin().clone();
    let observed_epoch = handle.document_epoch();

    let current_epoch = observed_epoch
        .after_same_document_mutation(SameDocumentMutationKind::TargetReplaced)
        .expect("replacement must rotate the document epoch");

    assert_eq!(current_epoch.value(), 2);
    assert_eq!(
        handle.validate_current(session, context, &origin, current_epoch),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: observed_epoch,
            current: current_epoch,
        })
    );

    let reobserved = ObservedNodeHandle::new(session, context, origin.clone(), current_epoch, 43)
        .expect("re-observed checkout field");
    assert_eq!(
        reobserved.validate_current(session, context, &origin, current_epoch),
        Ok(())
    );
}

#[test]
fn relevant_same_document_mutations_rotate_the_document_epoch() {
    let epoch = DocumentEpoch::new(4).expect("document epoch");
    let cases = [
        SameDocumentMutationKind::TargetRemoved,
        SameDocumentMutationKind::TargetReplaced,
        SameDocumentMutationKind::RoleOrNameChanged,
        SameDocumentMutationKind::AccessibilityTreeInvalidated,
        SameDocumentMutationKind::FrameDocumentReplaced,
        SameDocumentMutationKind::ActionableSubtreeReplaced,
    ];

    for mutation in cases {
        assert!(
            mutation.requires_epoch_increment(),
            "mutation={mutation:?} must rotate the document epoch"
        );
        let next = epoch
            .after_same_document_mutation(mutation)
            .expect("relevant mutation must produce the next epoch");
        assert_eq!(next.value(), 5, "mutation={mutation:?}");
    }
}

#[test]
fn an_unrelated_non_semantic_mutation_may_keep_the_checkout_field_handle() {
    let handle = checkout_field_handle();
    let session = handle.browser_session();
    let context = handle.browsing_context();
    let origin = handle.origin().clone();
    let observed_epoch = handle.document_epoch();

    assert!(!SameDocumentMutationKind::NonSemanticUnrelated.requires_epoch_increment());
    let current_epoch = observed_epoch
        .after_same_document_mutation(SameDocumentMutationKind::NonSemanticUnrelated)
        .expect("unrelated mutation may preserve the epoch");

    assert_eq!(current_epoch, observed_epoch);
    assert_eq!(
        handle.validate_current(session, context, &origin, current_epoch),
        Ok(())
    );
}

#[test]
fn document_epoch_overflow_fails_closed_instead_of_wrapping() {
    let epoch = DocumentEpoch::new(u64::MAX).expect("maximum document epoch");
    assert_eq!(
        epoch.after_same_document_mutation(SameDocumentMutationKind::TargetRemoved),
        Err(NodeHandleError::DocumentEpochOverflow)
    );
    assert_eq!(
        NodeHandleError::DocumentEpochOverflow.to_string(),
        "document epoch cannot wrap after a same-document mutation"
    );
}
