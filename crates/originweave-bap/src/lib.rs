//! Stable internal Browser Agent Protocol lifecycle contracts.
//!
//! This crate intentionally owns no transport, browser, network, model, secret,
//! approval, or persistence authority. External protocol adapters may project
//! these states, but protocol metadata cannot mint or change OriginWeave task
//! authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Durable logical state of one governed BAP task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapTaskState {
    /// The task record exists but has not entered admission control.
    Created,
    /// Admission control accepted the task but execution has not started.
    Admitted,
    /// The task is actively executing governed work.
    Running,
    /// Execution is suspended until an approval decision is available.
    WaitingForApproval,
    /// Execution is suspended until required external input is available.
    WaitingForExternalInput,
    /// Execution is suspended at a compatible recoverable checkpoint.
    Checkpointed,
    /// Execution is suspended until an explicit reconciliation decision is recorded.
    ///
    /// The lifecycle state does not itself persist or authenticate reconciliation
    /// evidence. A durable owner must preserve the complete evidence that caused
    /// the task to enter this state before resolution is considered.
    ReconciliationRequired,
    /// The declared post-condition completed successfully.
    Succeeded,
    /// The task reached a terminal execution failure.
    Failed,
    /// Cancellation completed and the task cannot resume.
    Cancelled,
    /// The task exceeded its allowed lifetime and cannot resume.
    Expired,
    /// The task was terminally removed from automatic execution after governed handling.
    ///
    /// Durable dead-letter evidence remains the responsibility of the persistence
    /// boundary; this in-memory marker must not be treated as the evidence itself.
    DeadLettered,
}

impl BapTaskState {
    /// Return whether this state is final and must never transition again.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded
                | Self::Failed
                | Self::Cancelled
                | Self::Expired
                | Self::DeadLettered
        )
    }
}

/// One requested task-lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapTaskEvent {
    /// Admit a newly created task.
    Admit,
    /// Start an admitted task.
    Start,
    /// Suspend a running task until approval is available.
    WaitForApproval,
    /// Suspend a running task until external input is available.
    WaitForExternalInput,
    /// Suspend a running task at a recoverable checkpoint.
    Checkpoint,
    /// Resume a normal suspended task into governed execution.
    Resume,
    /// Suspend a running task because its external outcome requires reconciliation.
    RequireReconciliation,
    /// Explicitly resolve a reconciliation hold and return the task to governed execution.
    ResolveReconciliation,
    /// Terminally remove a running or reconciliation-held task from automatic execution.
    DeadLetter,
    /// Record successful completion after the declared post-condition is verified.
    Succeed,
    /// Record terminal task failure.
    Fail,
    /// Record terminal cancellation.
    Cancel,
    /// Record terminal expiry.
    Expire,
}

/// A fail-closed lifecycle transition failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapTaskTransitionError {
    /// The requested event is not valid from the current non-terminal state.
    InvalidTransition {
        /// Current state that rejected the event.
        from: BapTaskState,
        /// Event that was rejected.
        event: BapTaskEvent,
    },
    /// The lifecycle sequence reached its maximum representable value.
    SequenceExhausted,
    /// A terminal task cannot be reopened or mutated by lifecycle events.
    TerminalState {
        /// Final state that rejected all further events.
        state: BapTaskState,
    },
}

impl std::fmt::Display for BapTaskTransitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition { from, event } => {
                write!(
                    formatter,
                    "BAP task event {event:?} is invalid from state {from:?}"
                )
            }
            Self::SequenceExhausted => {
                write!(formatter, "BAP task transition sequence is exhausted")
            }
            Self::TerminalState { state } => {
                write!(formatter, "BAP task state {state:?} is terminal")
            }
        }
    }
}

impl std::error::Error for BapTaskTransitionError {}

/// A fail-closed lifecycle recovery failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapTaskRestoreError {
    /// The supplied state and transition sequence cannot arise from this state machine.
    InvalidSnapshot {
        /// Logical state supplied by the durable recovery boundary.
        state: BapTaskState,
        /// Last accepted transition sequence supplied by the durable recovery boundary.
        transition_sequence: u64,
    },
}

impl std::fmt::Display for BapTaskRestoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSnapshot {
                state,
                transition_sequence,
            } => write!(
                formatter,
                "BAP task snapshot state {state:?} with transition sequence {transition_sequence} is unreachable"
            ),
        }
    }
}

impl std::error::Error for BapTaskRestoreError {}

/// Immutable receipt for one accepted in-memory lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BapTaskTransition {
    previous_state: BapTaskState,
    current_state: BapTaskState,
    sequence: u64,
}

impl BapTaskTransition {
    /// Return the state before the accepted transition.
    #[must_use]
    pub const fn previous_state(self) -> BapTaskState {
        self.previous_state
    }

    /// Return the state after the accepted transition.
    #[must_use]
    pub const fn current_state(self) -> BapTaskState {
        self.current_state
    }

    /// Return the monotonic transition sequence for this lifecycle instance.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

/// Deterministic fail-closed BAP task-lifecycle kernel.
///
/// This value is intentionally an in-memory state-transition primitive. A
/// durable repository must persist accepted transitions and impose its own
/// bounded sequence/retention contract before commercial task recovery can be
/// claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BapTaskLifecycle {
    state: BapTaskState,
    transition_sequence: u64,
}

impl Default for BapTaskLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl BapTaskLifecycle {
    /// Create one lifecycle in the `created` state with no accepted transitions.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: BapTaskState::Created,
            transition_sequence: 0,
        }
    }

    /// Restore a lifecycle state and its last accepted transition sequence.
    ///
    /// Recovery accepts only state/sequence pairs that are reachable through
    /// this exact state machine. This prevents corrupt or stale durable metadata
    /// from manufacturing an impossible execution state.
    pub const fn restore(
        state: BapTaskState,
        transition_sequence: u64,
    ) -> Result<Self, BapTaskRestoreError> {
        if !reachable_snapshot(state, transition_sequence) {
            return Err(BapTaskRestoreError::InvalidSnapshot {
                state,
                transition_sequence,
            });
        }
        Ok(Self {
            state,
            transition_sequence,
        })
    }

    /// Return the current logical task state.
    #[must_use]
    pub const fn state(self) -> BapTaskState {
        self.state
    }

    /// Return the number of accepted lifecycle transitions.
    #[must_use]
    pub const fn transition_sequence(self) -> u64 {
        self.transition_sequence
    }

    /// Apply one reviewed lifecycle event without granting execution authority.
    ///
    /// Rejected events leave both state and sequence unchanged. Terminal states
    /// reject every later event before evaluating any normal transition rule.
    /// Reconciliation cannot use the generic `Resume` event: it requires the
    /// explicit `ResolveReconciliation` event so ambiguous external outcomes
    /// cannot silently re-enter execution.
    pub fn apply(
        &mut self,
        event: BapTaskEvent,
    ) -> Result<BapTaskTransition, BapTaskTransitionError> {
        if self.state.is_terminal() {
            return Err(BapTaskTransitionError::TerminalState { state: self.state });
        }

        let next_state = match (self.state, event) {
            (BapTaskState::Created, BapTaskEvent::Admit) => BapTaskState::Admitted,
            (BapTaskState::Admitted, BapTaskEvent::Start) => BapTaskState::Running,
            (BapTaskState::Running, BapTaskEvent::WaitForApproval) => {
                BapTaskState::WaitingForApproval
            }
            (BapTaskState::Running, BapTaskEvent::WaitForExternalInput) => {
                BapTaskState::WaitingForExternalInput
            }
            (BapTaskState::Running, BapTaskEvent::Checkpoint) => BapTaskState::Checkpointed,
            (
                BapTaskState::WaitingForApproval
                | BapTaskState::WaitingForExternalInput
                | BapTaskState::Checkpointed,
                BapTaskEvent::Resume,
            ) => BapTaskState::Running,
            (BapTaskState::Running, BapTaskEvent::RequireReconciliation) => {
                BapTaskState::ReconciliationRequired
            }
            (
                BapTaskState::ReconciliationRequired,
                BapTaskEvent::ResolveReconciliation,
            ) => BapTaskState::Running,
            (
                BapTaskState::Running | BapTaskState::ReconciliationRequired,
                BapTaskEvent::DeadLetter,
            ) => BapTaskState::DeadLettered,
            (BapTaskState::Running, BapTaskEvent::Succeed) => BapTaskState::Succeeded,
            (_, BapTaskEvent::Fail) => BapTaskState::Failed,
            (_, BapTaskEvent::Cancel) => BapTaskState::Cancelled,
            (_, BapTaskEvent::Expire) => BapTaskState::Expired,
            (from, event) => {
                return Err(BapTaskTransitionError::InvalidTransition { from, event });
            }
        };

        let Some(sequence) = self.transition_sequence.checked_add(1) else {
            return Err(BapTaskTransitionError::SequenceExhausted);
        };
        let previous_state = self.state;
        self.state = next_state;
        self.transition_sequence = sequence;
        Ok(BapTaskTransition {
            previous_state,
            current_state: next_state,
            sequence,
        })
    }
}

const fn reachable_snapshot(state: BapTaskState, transition_sequence: u64) -> bool {
    match state {
        BapTaskState::Created => transition_sequence == 0,
        BapTaskState::Admitted => transition_sequence == 1,
        BapTaskState::Running => transition_sequence >= 2 && transition_sequence.is_multiple_of(2),
        BapTaskState::WaitingForApproval
        | BapTaskState::WaitingForExternalInput
        | BapTaskState::Checkpointed
        | BapTaskState::ReconciliationRequired => {
            transition_sequence >= 3 && !transition_sequence.is_multiple_of(2)
        }
        BapTaskState::Succeeded => {
            transition_sequence >= 3 && !transition_sequence.is_multiple_of(2)
        }
        BapTaskState::Failed | BapTaskState::Cancelled | BapTaskState::Expired => {
            transition_sequence >= 1
        }
        BapTaskState::DeadLettered => transition_sequence >= 3,
    }
}
