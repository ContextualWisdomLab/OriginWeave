use std::error::Error;
use std::fmt::{self, Display, Formatter};

use originweave_core::{ActionIntentDigest, ActionKind, Origin};

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
    /// The caller supplied no positive freshness budget for the observation.
    ZeroObservationDelayBudget,
    /// The verified observation arrived after the caller-selected freshness budget.
    PostConditionObservationExpired {
        /// Elapsed time between dispatch and the verified observation.
        elapsed_milliseconds: u64,
        /// Maximum dispatch-to-observation delay accepted by the caller's policy.
        maximum_observation_delay_milliseconds: u64,
    },
    /// Node-state provenance belongs to an origin other than the governed action target.
    PostConditionOriginMismatch,
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
            Self::ZeroObservationDelayBudget => formatter
                .write_str("post-condition observation delay budget must be greater than zero"),
            Self::PostConditionObservationExpired {
                elapsed_milliseconds,
                maximum_observation_delay_milliseconds,
            } => write!(
                formatter,
                "post-condition observation delay {elapsed_milliseconds} ms exceeds the configured maximum of {maximum_observation_delay_milliseconds} ms"
            ),
            Self::PostConditionOriginMismatch => formatter.write_str(
                "node-state post-condition provenance must match the governed action target origin",
            ),
        }
    }
}

impl Error for VerifiedActionOutcomeError {}

/// A caller-bounded monotonic observation window for one action post-condition.
///
/// Construction fails closed when an observation predates dispatch, when the
/// freshness budget is zero, or when the observation arrives after that budget.
/// Dispatch and observation timestamps must come from the same monotonic clock
/// domain; equal values are allowed for coarse clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostConditionObservation {
    dispatched_at_milliseconds: u64,
    observed_at_milliseconds: u64,
    maximum_observation_delay_milliseconds: u64,
}

impl PostConditionObservation {
    /// Validate one monotonic post-condition observation against its freshness budget.
    pub fn new(
        dispatched_at_milliseconds: u64,
        observed_at_milliseconds: u64,
        maximum_observation_delay_milliseconds: u64,
    ) -> Result<Self, VerifiedActionOutcomeError> {
        if observed_at_milliseconds < dispatched_at_milliseconds {
            return Err(VerifiedActionOutcomeError::PostConditionPredatesDispatch {
                dispatched_at_milliseconds,
                observed_at_milliseconds,
            });
        }
        if maximum_observation_delay_milliseconds == 0 {
            return Err(VerifiedActionOutcomeError::ZeroObservationDelayBudget);
        }
        let elapsed_milliseconds = observed_at_milliseconds - dispatched_at_milliseconds;
        if elapsed_milliseconds > maximum_observation_delay_milliseconds {
            return Err(
                VerifiedActionOutcomeError::PostConditionObservationExpired {
                    elapsed_milliseconds,
                    maximum_observation_delay_milliseconds,
                },
            );
        }
        Ok(Self {
            dispatched_at_milliseconds,
            observed_at_milliseconds,
            maximum_observation_delay_milliseconds,
        })
    }

    /// Return the monotonic timestamp recorded when action dispatch began.
    #[must_use]
    pub const fn dispatched_at_milliseconds(self) -> u64 {
        self.dispatched_at_milliseconds
    }

    /// Return the monotonic timestamp recorded when the post-condition was observed.
    #[must_use]
    pub const fn observed_at_milliseconds(self) -> u64 {
        self.observed_at_milliseconds
    }

    /// Return the caller-selected maximum accepted dispatch-to-observation delay.
    #[must_use]
    pub const fn maximum_observation_delay_milliseconds(self) -> u64 {
        self.maximum_observation_delay_milliseconds
    }
}

/// Credential-safe evidence that a typed action completed its verified post-condition.
///
/// Construction is intentionally fail-closed: a command acknowledgement,
/// unverified or rejected provenance, an invalid temporal observation, or
/// node-state provenance from a different origin cannot be represented by this
/// type as successful action completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedActionOutcomeEvidence {
    action: ActionKind,
    target_origin: Origin,
    intent_digest: ActionIntentDigest,
    post_condition: PostConditionKind,
    observation: PostConditionObservation,
    provenance: ProvenanceRecord,
}

impl VerifiedActionOutcomeEvidence {
    /// Create successful action evidence from verified, temporally bounded provenance.
    ///
    /// The observation has already established caller-selected freshness and
    /// monotonic ordering. `NodeStateChanged` provenance must also originate from
    /// the canonical action target. This constructor does not independently prove
    /// clock provenance, browser dispatch, or causal attribution between the
    /// action and the observed post-condition.
    pub fn new(
        action: ActionKind,
        target_origin: Origin,
        intent_digest: ActionIntentDigest,
        post_condition: PostConditionKind,
        observation: PostConditionObservation,
        provenance: ProvenanceRecord,
    ) -> Result<Self, VerifiedActionOutcomeError> {
        if provenance.verification_result() != VerificationResult::Verified {
            return Err(VerifiedActionOutcomeError::PostConditionNotVerified);
        }
        if post_condition == PostConditionKind::NodeStateChanged
            && provenance.source_origin() != &target_origin
        {
            return Err(VerifiedActionOutcomeError::PostConditionOriginMismatch);
        }
        Ok(Self {
            action,
            target_origin,
            intent_digest,
            post_condition,
            observation,
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

    /// Return the validated monotonic observation window for this post-condition.
    #[must_use]
    pub const fn observation(&self) -> PostConditionObservation {
        self.observation
    }

    /// Return the monotonic timestamp recorded when action dispatch began.
    #[must_use]
    pub const fn dispatched_at_milliseconds(&self) -> u64 {
        self.observation.dispatched_at_milliseconds()
    }

    /// Return the monotonic timestamp recorded when the post-condition was observed.
    #[must_use]
    pub const fn observed_at_milliseconds(&self) -> u64 {
        self.observation.observed_at_milliseconds()
    }

    /// Return the caller-selected maximum accepted dispatch-to-observation delay.
    #[must_use]
    pub const fn maximum_observation_delay_milliseconds(&self) -> u64 {
        self.observation.maximum_observation_delay_milliseconds()
    }

    /// Return the exact provenance record that verified the post-condition.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceRecord {
        &self.provenance
    }
}
