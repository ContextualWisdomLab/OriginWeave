//! Stable internal Browser Agent Protocol lifecycle contracts.
//!
//! This crate intentionally owns no transport, browser, network, model, secret,
//! approval, or persistence authority. External protocol adapters may project
//! these states, but protocol metadata cannot mint or change OriginWeave task
//! authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Maximum UTF-8 byte length of a mutating command's idempotency key.
pub const MAX_BAP_IDEMPOTENCY_KEY_BYTES: usize = 128;
/// Maximum UTF-8 byte length of a BAP task identifier.
pub const MAX_BAP_TASK_ID_BYTES: usize = 128;

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
    /// The declared post-condition completed successfully.
    Succeeded,
    /// The task reached a terminal execution failure.
    Failed,
    /// Cancellation completed and the task cannot resume.
    Cancelled,
    /// The task exceeded its allowed lifetime and cannot resume.
    Expired,
}

impl BapTaskState {
    /// Return whether this state is final and must never transition again.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Expired
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
    /// Resume a suspended task into governed execution.
    Resume,
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
    event: BapTaskEvent,
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

    /// Return the accepted lifecycle event represented by this receipt.
    #[must_use]
    pub const fn event(self) -> BapTaskEvent {
        self.event
    }
}

/// A validation or lifecycle failure while issuing a BAP command receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapCommandReceiptError {
    /// The idempotency key was empty or contained unsupported input.
    InvalidIdempotencyKey,
    /// The idempotency key exceeded its byte bound.
    IdempotencyKeyLimitExceeded,
    /// The task identifier was empty or contained unsupported input.
    InvalidTaskId,
    /// The task identifier exceeded its byte bound.
    TaskIdLimitExceeded,
    /// The lifecycle event could not be accepted for the current task state.
    TransitionRejected {
        /// The lifecycle failure preserved by the receipt boundary.
        error: BapTaskTransitionError,
    },
}

impl std::fmt::Display for BapCommandReceiptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdempotencyKey => write!(formatter, "BAP idempotency key is invalid"),
            Self::IdempotencyKeyLimitExceeded => {
                write!(formatter, "BAP idempotency key exceeds its byte limit")
            }
            Self::InvalidTaskId => write!(formatter, "BAP task ID is invalid"),
            Self::TaskIdLimitExceeded => write!(formatter, "BAP task ID exceeds its byte limit"),
            Self::TransitionRejected { error } => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BapCommandReceiptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TransitionRejected { error } => Some(error),
            Self::InvalidIdempotencyKey
            | Self::IdempotencyKeyLimitExceeded
            | Self::InvalidTaskId
            | Self::TaskIdLimitExceeded => None,
        }
    }
}

/// An immutable receipt binding one accepted lifecycle command to its retry key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BapCommandReceipt {
    idempotency_key: String,
    task_id: String,
    transition: BapTaskTransition,
}

impl BapCommandReceipt {
    fn from_validated(idempotency_key: &str, task_id: &str, transition: BapTaskTransition) -> Self {
        Self {
            idempotency_key: idempotency_key.to_owned(),
            task_id: task_id.to_owned(),
            transition,
        }
    }

    /// Return the opaque retry key supplied by the caller.
    #[must_use]
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Return the task identity bound to this receipt.
    #[must_use]
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Return the accepted lifecycle event.
    #[must_use]
    pub const fn event(&self) -> BapTaskEvent {
        self.transition.event()
    }

    /// Return the immutable transition evidence carried by this receipt.
    #[must_use]
    pub const fn transition(&self) -> BapTaskTransition {
        self.transition
    }

    /// Return whether a retry has the exact same task, key, and lifecycle event.
    #[must_use]
    pub fn matches(&self, idempotency_key: &str, task_id: &str, event: BapTaskEvent) -> bool {
        self.idempotency_key == idempotency_key && self.task_id == task_id && self.event() == event
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
            event,
        })
    }

    /// Apply one lifecycle event and bind the accepted transition to a retry receipt.
    ///
    /// This remains an in-memory contract: it identifies an exact retry but does
    /// not provide durable deduplication or side-effect suppression. Receipts can
    /// only be minted at this accepted-command boundary; callers cannot rebind an
    /// already accepted transition to different retry or task metadata afterward.
    pub fn apply_with_receipt(
        &mut self,
        idempotency_key: &str,
        task_id: &str,
        event: BapTaskEvent,
    ) -> Result<BapCommandReceipt, BapCommandReceiptError> {
        validate_idempotency_key(idempotency_key)?;
        validate_task_id(task_id)?;
        let transition = self
            .apply(event)
            .map_err(|error| BapCommandReceiptError::TransitionRejected { error })?;
        Ok(BapCommandReceipt::from_validated(
            idempotency_key,
            task_id,
            transition,
        ))
    }
}

fn validate_idempotency_key(value: &str) -> Result<(), BapCommandReceiptError> {
    if value.len() > MAX_BAP_IDEMPOTENCY_KEY_BYTES {
        return Err(BapCommandReceiptError::IdempotencyKeyLimitExceeded);
    }
    if value.is_empty() || !value.bytes().all(valid_identifier_byte) {
        return Err(BapCommandReceiptError::InvalidIdempotencyKey);
    }
    Ok(())
}

fn validate_task_id(value: &str) -> Result<(), BapCommandReceiptError> {
    if value.len() > MAX_BAP_TASK_ID_BYTES {
        return Err(BapCommandReceiptError::TaskIdLimitExceeded);
    }
    if value.is_empty() || !value.bytes().all(valid_identifier_byte) {
        return Err(BapCommandReceiptError::InvalidTaskId);
    }
    Ok(())
}

const fn valid_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

const fn reachable_snapshot(state: BapTaskState, transition_sequence: u64) -> bool {
    match state {
        BapTaskState::Created => transition_sequence == 0,
        BapTaskState::Admitted => transition_sequence == 1,
        BapTaskState::Running => transition_sequence >= 2 && transition_sequence.is_multiple_of(2),
        BapTaskState::WaitingForApproval
        | BapTaskState::WaitingForExternalInput
        | BapTaskState::Checkpointed => {
            transition_sequence >= 3 && !transition_sequence.is_multiple_of(2)
        }
        BapTaskState::Succeeded => {
            transition_sequence >= 3 && !transition_sequence.is_multiple_of(2)
        }
        BapTaskState::Failed | BapTaskState::Cancelled | BapTaskState::Expired => {
            transition_sequence >= 1
        }
    }
}
