use std::collections::BTreeSet;
use std::fmt;

use crate::{BrowserAuthorityRegistry, ObservedNodeHandle};

/// Maximum UTF-8 byte length retained for one semantic node role.
pub const MAX_SEMANTIC_ROLE_BYTES: usize = 64;
/// Maximum UTF-8 byte length retained for one semantic node accessible name.
pub const MAX_ACCESSIBLE_NAME_BYTES: usize = 512;
/// Maximum UTF-8 byte length retained for one semantic node visible-text excerpt.
pub const MAX_VISIBLE_TEXT_BYTES: usize = 4_096;
/// Maximum number of child relationships retained for one semantic node observation.
pub const MAX_SEMANTIC_CHILDREN: usize = 128;

/// A node-local typed action advertised by an observation adapter.
///
/// This is descriptive evidence only and never grants execution authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeActionKind {
    /// Activate the node using browser-native click semantics.
    Click,
    /// Insert bounded non-secret text using browser-native input semantics.
    TypeText,
    /// Select one option using browser-native selection semantics.
    SelectOption,
    /// Set a checkable control to an explicit checked state.
    SetChecked,
    /// Scroll the node into the viewport without activating it.
    ScrollIntoView,
}

/// A structured evidence channel that contributed to a semantic observation.
///
/// Channel provenance never converts page-provided content into trusted instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservationChannel {
    /// Experimental structured browser tool metadata, such as WebMCP when available.
    WebMcp,
    /// Structured data interpreted by a versioned adapter.
    StructuredData,
    /// Browser accessibility-tree evidence.
    Accessibility,
    /// Browser DOM evidence used through a bounded adapter.
    Dom,
    /// Browser layout evidence used through a bounded adapter.
    Layout,
    /// Bounded visual evidence used when structured channels are insufficient.
    Visual,
}

/// Caller-owned fields used to construct one bounded semantic node observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNodeObservationInput {
    /// Exact OriginWeave authority handle for the observed node.
    pub handle: ObservedNodeHandle,
    /// Optional exact-authority parent relationship.
    pub parent: Option<ObservedNodeHandle>,
    /// Bounded exact-authority child relationships in adapter-observed order.
    pub children: Vec<ObservedNodeHandle>,
    /// Bounded semantic or accessibility role.
    pub role: String,
    /// Bounded accessible name; an empty name is valid.
    pub accessible_name: String,
    /// Optional bounded visible-text excerpt.
    pub visible_text: Option<String>,
    /// Whether the adapter observed the node as enabled.
    pub enabled: bool,
    /// Whether the adapter observed the node as visible.
    pub visible: bool,
    /// Optional selected state when that concept applies.
    pub selected: Option<bool>,
    /// Finite typed actions the adapter reports as meaningful for this node.
    pub supported_actions: BTreeSet<NodeActionKind>,
    /// Finite evidence channels that contributed to this observation.
    pub evidence_channels: BTreeSet<ObservationChannel>,
}

/// A bounded semantic view of one authority-bound browser node.
///
/// The value carries no raw HTML, protocol-local identifier, or independent authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNodeObservation {
    handle: ObservedNodeHandle,
    parent: Option<ObservedNodeHandle>,
    children: Vec<ObservedNodeHandle>,
    role: String,
    accessible_name: String,
    visible_text: Option<String>,
    enabled: bool,
    visible: bool,
    selected: Option<bool>,
    supported_actions: BTreeSet<NodeActionKind>,
    evidence_channels: BTreeSet<ObservationChannel>,
}

impl SemanticNodeObservation {
    /// Validate reviewed text, relationship, authority, and provenance bounds.
    ///
    /// Every primary or related node handle must still be live authority owned by
    /// `registry`. Caller-constructed, retired, stale, or otherwise unbound handles
    /// fail closed before page-derived semantic metadata can become an observation.
    pub fn new(
        input: SemanticNodeObservationInput,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<Self, SemanticNodeObservationError> {
        if input.role.is_empty() {
            return Err(SemanticNodeObservationError::EmptyRole);
        }
        if input.role.len() > MAX_SEMANTIC_ROLE_BYTES {
            return Err(SemanticNodeObservationError::RoleTooLong);
        }
        if input.accessible_name.len() > MAX_ACCESSIBLE_NAME_BYTES {
            return Err(SemanticNodeObservationError::AccessibleNameTooLong);
        }
        if input
            .visible_text
            .as_ref()
            .is_some_and(|text| text.len() > MAX_VISIBLE_TEXT_BYTES)
        {
            return Err(SemanticNodeObservationError::VisibleTextTooLong);
        }
        if input.evidence_channels.is_empty() {
            return Err(SemanticNodeObservationError::MissingEvidenceChannel);
        }
        if input.children.len() > MAX_SEMANTIC_CHILDREN {
            return Err(SemanticNodeObservationError::TooManyChildren);
        }
        validate_live_node(registry, &input.handle)?;
        if let Some(parent) = input.parent.as_ref() {
            validate_live_node(registry, parent)?;
            validate_relationship(&input.handle, parent)?;
        }
        for (index, child) in input.children.iter().enumerate() {
            validate_live_node(registry, child)?;
            validate_relationship(&input.handle, child)?;
            if input.children[..index].contains(child) {
                return Err(SemanticNodeObservationError::DuplicateChild);
            }
        }
        Ok(Self {
            handle: input.handle,
            parent: input.parent,
            children: input.children,
            role: input.role,
            accessible_name: input.accessible_name,
            visible_text: input.visible_text,
            enabled: input.enabled,
            visible: input.visible,
            selected: input.selected,
            supported_actions: input.supported_actions,
            evidence_channels: input.evidence_channels,
        })
    }

    /// Return the exact authority-bound node handle.
    #[must_use]
    pub const fn handle(&self) -> &ObservedNodeHandle {
        &self.handle
    }

    /// Return the optional exact-authority parent relationship.
    #[must_use]
    pub const fn parent(&self) -> Option<&ObservedNodeHandle> {
        self.parent.as_ref()
    }

    /// Return the bounded exact-authority child relationships in observed order.
    #[must_use]
    pub fn children(&self) -> &[ObservedNodeHandle] {
        &self.children
    }

    /// Return the bounded semantic role.
    #[must_use]
    pub fn role(&self) -> &str {
        &self.role
    }

    /// Return the bounded accessible name.
    #[must_use]
    pub fn accessible_name(&self) -> &str {
        &self.accessible_name
    }

    /// Return the optional bounded visible-text excerpt.
    #[must_use]
    pub fn visible_text(&self) -> Option<&str> {
        self.visible_text.as_deref()
    }

    /// Return whether the node was observed as enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Return whether the node was observed as visible.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Return the optional selected state.
    #[must_use]
    pub const fn is_selected(&self) -> Option<bool> {
        self.selected
    }

    /// Return the adapter-advertised node action set.
    #[must_use]
    pub const fn supported_actions(&self) -> &BTreeSet<NodeActionKind> {
        &self.supported_actions
    }

    /// Return the non-empty evidence-channel provenance set.
    #[must_use]
    pub const fn evidence_channels(&self) -> &BTreeSet<ObservationChannel> {
        &self.evidence_channels
    }
}

fn validate_live_node(
    registry: &BrowserAuthorityRegistry,
    handle: &ObservedNodeHandle,
) -> Result<(), SemanticNodeObservationError> {
    registry
        .validate_node_handle(handle)
        .map_err(|_error| SemanticNodeObservationError::UnknownNodeAuthority)
}

fn validate_relationship(
    handle: &ObservedNodeHandle,
    related: &ObservedNodeHandle,
) -> Result<(), SemanticNodeObservationError> {
    if handle == related {
        return Err(SemanticNodeObservationError::SelfRelationship);
    }
    if handle.browser_session() != related.browser_session()
        || handle.browsing_context() != related.browsing_context()
        || handle.origin() != related.origin()
        || handle.document_epoch() != related.document_epoch()
    {
        return Err(SemanticNodeObservationError::RelationshipAuthorityMismatch);
    }
    Ok(())
}

/// A bounded validation failure for one semantic node observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticNodeObservationError {
    /// The semantic role was empty.
    EmptyRole,
    /// The role exceeded [`MAX_SEMANTIC_ROLE_BYTES`].
    RoleTooLong,
    /// The accessible name exceeded [`MAX_ACCESSIBLE_NAME_BYTES`].
    AccessibleNameTooLong,
    /// The visible-text excerpt exceeded [`MAX_VISIBLE_TEXT_BYTES`].
    VisibleTextTooLong,
    /// No evidence channel was supplied for the observation.
    MissingEvidenceChannel,
    /// The child relationship list exceeded [`MAX_SEMANTIC_CHILDREN`].
    TooManyChildren,
    /// A supplied node handle is not current authority owned by the active browser registry.
    UnknownNodeAuthority,
    /// A relationship crossed the observation's session, context, origin, or document authority.
    RelationshipAuthorityMismatch,
    /// The observation attempted to relate the node to itself.
    SelfRelationship,
    /// The child relationship list contained the same exact handle more than once.
    DuplicateChild,
}

impl fmt::Display for SemanticNodeObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRole => formatter.write_str("semantic node role must not be empty"),
            Self::RoleTooLong => formatter.write_str("semantic node role exceeds 64 UTF-8 bytes"),
            Self::AccessibleNameTooLong => {
                formatter.write_str("semantic node accessible name exceeds 512 UTF-8 bytes")
            }
            Self::VisibleTextTooLong => {
                formatter.write_str("semantic node visible text exceeds 4096 UTF-8 bytes")
            }
            Self::MissingEvidenceChannel => formatter
                .write_str("semantic node observation requires at least one evidence channel"),
            Self::TooManyChildren => {
                formatter.write_str("semantic node observation exceeds 128 child relationships")
            }
            Self::UnknownNodeAuthority => formatter.write_str(
                "semantic node observation contains node authority not owned by the active browser registry",
            ),
            Self::RelationshipAuthorityMismatch => formatter.write_str(
                "semantic node relationship crosses its session, context, origin, or document authority",
            ),
            Self::SelfRelationship => {
                formatter.write_str("semantic node observation cannot relate the node to itself")
            }
            Self::DuplicateChild => formatter
                .write_str("semantic node observation contains a duplicate child relationship"),
        }
    }
}

impl std::error::Error for SemanticNodeObservationError {}
