use std::error::Error;
use std::fmt::{self, Display, Formatter};

use originweave_core::{ActionIntentDigest, ActionKind, ObservedNodeHandle, Origin};

use crate::{ProvenanceRecord, VerificationResult};

/// A bounded observable state transition that may prove a browser action completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PostConditionKind {
    /// The canonical browser URL changed to the expected resulting location.
    UrlChanged,
    /// A governed semantic node reached the expected state after the action.
    NodeStateChanged,
    /// A browser dialog entered the expected visible or closed state.
    DialogStateChanged,
    /// A bounded network-side mutation attributable to the action was observed.
    NetworkMutationObserved,
}

/// A failure to construct successful action evidence from post-condition proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedActionOutcomeError {
    /// The supplied provenance did not independently verify the post-condition.
    PostConditionNotVerified,
    /// The claimed post-condition observation happened before action dispatch.
    PostConditionPredatesDispatch {
        /// Monotonic timestamp recorded when the governed action was dispatched.
        dispatched_at_milliseconds: u64,
        /// Monotonic timestamp recorded when the post-condition was observed.
        observed_at_milliseconds: u64,
    },
    /// Node-state success used the generic constructor without exact node authority.
    NodeStateTargetRequired,
    /// Node-state provenance belongs to an origin other than the governed action target.
    PostConditionOriginMismatch,
    /// The observed node differs from the exact governed action target node.
    PostConditionNodeMismatch,
}

impl Display for VerifiedActionOutcomeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostConditionNotVerified => formatter
                .write_str("action success requires an independently verified post-condition"),
            Self::PostConditionPredatesDispatch {
                dispatched_at_milliseconds,
                observed_at_milliseconds,
            } => write!(
                formatter,
                "post-condition observation at {observed_at_milliseconds} ms predates action dispatch at {dispatched_at_milliseconds} ms"
            ),
            Self::NodeStateTargetRequired => formatter.write_str(
                "node-state post-condition requires the exact governed action target node",
            ),
            Self::PostConditionOriginMismatch => formatter.write_str(
                "node-state post-condition provenance must match the governed action target origin",
            ),
            Self::PostConditionNodeMismatch => formatter.write_str(
                "node-state post-condition must observe the exact governed action target node",
            ),
        }
    }
}

impl Error for VerifiedActionOutcomeError {}

/// Credential-safe evidence that a typed action completed its verified post-condition.
///
/// Construction is intentionally fail-closed: a command acknowledgement, an
/// unverified observation, rejected provenance, an observation timestamp earlier
/// than the action dispatch, or a node-state observation that is not bound to the
/// exact governed [`ObservedNodeHandle`] cannot be represented by this type as
/// successful action completion. Equal dispatch and observation timestamps are
/// permitted because a bounded adapter may use a coarse monotonic clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedActionOutcomeEvidence {
    action: ActionKind,
    target_origin: Origin,
    target_node: Option<ObservedNodeHandle>,
    intent_digest: ActionIntentDigest,
    post_condition: PostConditionKind,
    dispatched_at_milliseconds: u64,
    observed_at_milliseconds: u64,
    provenance: ProvenanceRecord,
}

impl VerifiedActionOutcomeEvidence {
    /// Create successful non-node action evidence from verified, ordered provenance.
    ///
    /// Both timestamps must come from the same monotonic clock domain. The
    /// observation may share the dispatch tick, but it may never predate it.
    /// `NodeStateChanged` is deliberately rejected here because origin-only
    /// provenance cannot identify the exact node that was governed; callers must
    /// use [`Self::new_node_state`] for that post-condition.
    pub fn new(
        action: ActionKind,
        target_origin: Origin,
        intent_digest: ActionIntentDigest,
        post_condition: PostConditionKind,
        dispatched_at_milliseconds: u64,
        observed_at_milliseconds: u64,
        provenance: ProvenanceRecord,
    ) -> Result<Self, VerifiedActionOutcomeError> {
        validate_common_post_condition(
            dispatched_at_milliseconds,
            observed_at_milliseconds,
            &provenance,
        )?;
        if post_condition == PostConditionKind::NodeStateChanged {
            return Err(VerifiedActionOutcomeError::NodeStateTargetRequired);
        }
        Ok(Self {
            action,
            target_origin,
            target_node: None,
            intent_digest,
            post_condition,
            dispatched_at_milliseconds,
            observed_at_milliseconds,
            provenance,
        })
    }

    /// Create node-state success evidence bound to one exact governed node.
    ///
    /// `target_node` is the node authority used by the governed action;
    /// `observed_node` is the independently observed node whose post-condition
    /// was verified. Both must be exactly equal across browser session, browsing
    /// context, canonical origin, document epoch, and node identifier. Provenance
    /// must also originate from that canonical target origin.
    pub fn new_node_state(
        action: ActionKind,
        target_node: ObservedNodeHandle,
        intent_digest: ActionIntentDigest,
        dispatched_at_milliseconds: u64,
        observed_at_milliseconds: u64,
        observed_node: ObservedNodeHandle,
        provenance: ProvenanceRecord,
    ) -> Result<Self, VerifiedActionOutcomeError> {
        validate_common_post_condition(
            dispatched_at_milliseconds,
            observed_at_milliseconds,
            &provenance,
        )?;
        if provenance.source_origin() != target_node.origin() {
            return Err(VerifiedActionOutcomeError::PostConditionOriginMismatch);
        }
        if observed_node != target_node {
            return Err(VerifiedActionOutcomeError::PostConditionNodeMismatch);
        }
        Ok(Self {
            action,
            target_origin: target_node.origin().clone(),
            target_node: Some(target_node),
            intent_digest,
            post_condition: PostConditionKind::NodeStateChanged,
            dispatched_at_milliseconds,
            observed_at_milliseconds,
            provenance,
        })
    }

    /// Return the typed action whose completion was verified.
    #[must_use]
    pub const fn action(&self) -> ActionKind {
        self.action
    }

    /// Return the canonical origin affected by the verified action.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the exact governed node for node-state evidence, when applicable.
    #[must_use]
    pub const fn target_node(&self) -> Option<&ObservedNodeHandle> {
        self.target_node.as_ref()
    }

    /// Return the digest of the complete canonical action intent.
    #[must_use]
    pub const fn intent_digest(&self) -> &ActionIntentDigest {
        &self.intent_digest
    }

    /// Return the bounded post-condition that was independently verified.
    #[must_use]
    pub const fn post_condition(&self) -> PostConditionKind {
        self.post_condition
    }

    /// Return the monotonic timestamp recorded when action dispatch began.
    #[must_use]
    pub const fn dispatched_at_milliseconds(&self) -> u64 {
        self.dispatched_at_milliseconds
    }

    /// Return the monotonic timestamp recorded when the post-condition was observed.
    #[must_use]
    pub const fn observed_at_milliseconds(&self) -> u64 {
        self.observed_at_milliseconds
    }

    /// Return the exact provenance record that verified the post-condition.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceRecord {
        &self.provenance
    }
}

fn validate_common_post_condition(
    dispatched_at_milliseconds: u64,
    observed_at_milliseconds: u64,
    provenance: &ProvenanceRecord,
) -> Result<(), VerifiedActionOutcomeError> {
    if provenance.verification_result() != VerificationResult::Verified {
        return Err(VerifiedActionOutcomeError::PostConditionNotVerified);
    }
    if observed_at_milliseconds < dispatched_at_milliseconds {
        return Err(VerifiedActionOutcomeError::PostConditionPredatesDispatch {
            dispatched_at_milliseconds,
            observed_at_milliseconds,
        });
    }
    Ok(())
}
