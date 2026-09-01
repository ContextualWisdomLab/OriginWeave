/// Browser control surface represented by assurance evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserAttachmentKind {
    /// OriginWeave is attached to an existing person-controlled browser tab.
    AttachedHumanTab,
    /// OriginWeave operates in a task-isolated browser profile.
    IsolatedProfile,
}

/// Trusted adapter evidence about extension influence on page state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionInfluenceEvidence {
    /// The trusted adapter established that an extension can influence page state.
    CanInfluencePageState,
    /// This bounded rule has no trusted evidence of extension influence.
    ///
    /// Absence of known influence is not proof that extensions are absent or unable
    /// to interfere.
    NoKnownExtensionInfluence,
}

/// A specific reason that one browser context has reduced assurance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducedAssuranceReason {
    /// An attached human tab can be influenced by an existing browser extension.
    AttachedTabExtensionInfluence,
}

/// Classify the narrow attached-tab extension-influence assurance reduction.
///
/// `None` means only that this specific rule found no such reduction; it is not
/// proof of full trust, extension absence, inability to interfere, or high
/// assurance.
#[must_use]
pub const fn classify_reduced_assurance(
    attachment: BrowserAttachmentKind,
    extension_influence: ExtensionInfluenceEvidence,
) -> Option<ReducedAssuranceReason> {
    if matches!(
        (attachment, extension_influence),
        (
            BrowserAttachmentKind::AttachedHumanTab,
            ExtensionInfluenceEvidence::CanInfluencePageState
        )
    ) {
        Some(ReducedAssuranceReason::AttachedTabExtensionInfluence)
    } else {
        None
    }
}
