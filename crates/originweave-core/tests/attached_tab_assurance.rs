use originweave_core::{
    BrowserAttachmentKind, ExtensionInfluenceEvidence, ReducedAssuranceReason,
    classify_reduced_assurance,
};

#[test]
fn attached_human_tab_with_extension_influence_is_explicitly_reduced_assurance() {
    assert_eq!(
        classify_reduced_assurance(
            BrowserAttachmentKind::AttachedHumanTab,
            ExtensionInfluenceEvidence::CanInfluencePageState,
        ),
        Some(ReducedAssuranceReason::AttachedTabExtensionInfluence)
    );
}

#[test]
fn attached_human_tab_without_known_extension_influence_has_no_extension_reduction() {
    assert_eq!(
        classify_reduced_assurance(
            BrowserAttachmentKind::AttachedHumanTab,
            ExtensionInfluenceEvidence::NoKnownExtensionInfluence,
        ),
        None
    );
}

#[test]
fn isolated_profile_is_not_relabelled_by_attached_tab_rule() {
    assert_eq!(
        classify_reduced_assurance(
            BrowserAttachmentKind::IsolatedProfile,
            ExtensionInfluenceEvidence::CanInfluencePageState,
        ),
        None
    );
}
