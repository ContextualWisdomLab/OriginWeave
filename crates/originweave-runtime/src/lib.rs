//! Agent-task step admission for OriginWeave.
//!
//! The BAP lifecycle records task state but grants no authority, and the policy
//! engine decides one action without knowing the task state. This crate joins
//! them into one decision for each agent step and returns a [`DispatchPermit`]
//! only for an allowed step.
//!
//! This crate does not dispatch browser actions. A future dispatch adapter must
//! require a [`DispatchPermit`] and must revalidate its task sequence and task
//! identity before it acts. That adapter is planned work.
//!
//! The crate is pure: it performs no I/O, executes no browser action, and does
//! not persist lifecycle transitions. A durable owner must persist every
//! returned transition before acting on it.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use originweave_bap::{BapTaskEvent, BapTaskLifecycle, BapTaskState, BapTaskTransition};
use originweave_core::mcp::ValidatedMcpToolCall;
use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, Origin, PolicyContext, RiskClass,
};
use originweave_policy::{Decision, DenialReason, evaluate_mcp};

/// Record that one agent step was admitted for dispatch.
///
/// Only [`admit_step`] creates a permit. The permit is not `Clone` or `Copy`,
/// so code cannot duplicate one permit value. It binds the exact action, MCP
/// tool, target origin, intent digest, and task sequence that were admitted.
///
/// A permit does not prevent repeated admission: calling [`admit_step`] again
/// for the same running task and step returns another equal permit, and a
/// permit carries no task identity. A future dispatch adapter must consume the
/// permit and reject a stale task sequence or a different task before it acts.
#[derive(Debug, PartialEq, Eq)]
pub struct DispatchPermit {
    action: ActionKind,
    tool_name: &'static str,
    target_origin: Origin,
    intent_digest: ActionIntentDigest,
    task_sequence: u64,
}

impl DispatchPermit {
    /// Return the admitted action.
    #[must_use]
    pub const fn action(&self) -> ActionKind {
        self.action
    }

    /// Return the canonical MCP tool name that requested the action.
    #[must_use]
    pub const fn tool_name(&self) -> &'static str {
        self.tool_name
    }

    /// Return the origin that the admitted action may affect.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the digest of the complete admitted action intent.
    #[must_use]
    pub const fn intent_digest(&self) -> &ActionIntentDigest {
        &self.intent_digest
    }

    /// Return the task transition sequence at which the step was admitted.
    #[must_use]
    pub const fn task_sequence(&self) -> u64 {
        self.task_sequence
    }

    /// Consume the permit and return its action, MCP tool name, target origin,
    /// intent digest, and task sequence.
    ///
    /// The returned values are plain data, not a dispatch credential. A
    /// dispatch adapter must take the permit itself, not these parts.
    #[must_use]
    pub fn into_parts(self) -> (ActionKind, &'static str, Origin, ActionIntentDigest, u64) {
        (
            self.action,
            self.tool_name,
            self.target_origin,
            self.intent_digest,
            self.task_sequence,
        )
    }
}

/// The governed result of admitting one agent step.
#[derive(Debug, PartialEq, Eq)]
pub enum StepAdmission {
    /// Policy allowed the step. The task stays `Running`.
    Permitted(DispatchPermit),
    /// Policy requires approval. The task moved to `WaitingForApproval`.
    Suspended {
        /// Risk class that requires approval.
        reason: RiskClass,
        /// Accepted lifecycle transition that the durable owner must persist.
        transition: BapTaskTransition,
    },
    /// Policy denied the step. The lifecycle is unchanged.
    Denied(DenialReason),
}

/// A fail-closed step-admission failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepAdmissionError {
    /// Only a `Running` task can admit a step. The lifecycle is unchanged.
    TaskNotRunning {
        /// Current state that rejected the step.
        state: BapTaskState,
    },
}

impl std::fmt::Display for StepAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskNotRunning { state } => write!(
                formatter,
                "agent task step requires a running task, but the task is {state:?}"
            ),
        }
    }
}

impl std::error::Error for StepAdmissionError {}

/// Admit one agent step for a governed task.
///
/// The lifecycle gate runs first: a task that is not `Running` is rejected
/// before policy evaluation. The validated MCP route and the action request
/// then pass through [`evaluate_mcp`] unchanged.
///
/// - `Allow` returns a [`DispatchPermit`] and leaves the lifecycle unchanged.
/// - `RequireApproval` applies [`BapTaskEvent::WaitForApproval`] and returns
///   the accepted transition. No permit is created.
/// - `Deny` returns the denial reason and leaves the lifecycle unchanged.
///
/// # Errors
///
/// Returns [`StepAdmissionError::TaskNotRunning`] for a task that is not
/// `Running`. The lifecycle is unchanged.
pub fn admit_step(
    lifecycle: &mut BapTaskLifecycle,
    call: &ValidatedMcpToolCall,
    request: &ActionRequest,
    context: &PolicyContext,
) -> Result<StepAdmission, StepAdmissionError> {
    // Prepare the suspension on a copy. The BAP lifecycle accepts
    // `WaitForApproval` only from `Running`, and a reachable `Running`
    // sequence always has room for one more transition. So this trial both
    // proves that the task is running and prepares the suspension, without
    // duplicating lifecycle rules or changing the caller's lifecycle.
    let mut suspended = *lifecycle;
    let Ok(transition) = suspended.apply(BapTaskEvent::WaitForApproval) else {
        return Err(StepAdmissionError::TaskNotRunning {
            state: lifecycle.state(),
        });
    };

    match evaluate_mcp(call, request, context) {
        Decision::Allow => Ok(StepAdmission::Permitted(DispatchPermit {
            action: request.action(),
            tool_name: call.tool_name(),
            target_origin: request.target_origin().clone(),
            intent_digest: request.intent_digest().clone(),
            task_sequence: lifecycle.transition_sequence(),
        })),
        Decision::RequireApproval(reason) => {
            *lifecycle = suspended;
            Ok(StepAdmission::Suspended { reason, transition })
        }
        Decision::Deny(reason) => Ok(StepAdmission::Denied(reason)),
    }
}
