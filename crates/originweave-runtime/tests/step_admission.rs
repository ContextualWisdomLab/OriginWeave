#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_bap::{BapTaskLifecycle, BapTaskState, BapTaskTransition};
use originweave_core::mcp::{MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, ValidatedMcpToolCall};
use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability,
    ExecutionPurpose, InstructionSource, Origin, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SessionMode,
};
use originweave_policy::DenialReason;
use originweave_runtime::{DispatchPermit, StepAdmission, StepAdmissionError, admit_step};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_INTENT: &str =
    "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

fn origin() -> Origin {
    Origin::parse("https://task.example").expect("valid test origin")
}

fn other_origin() -> Origin {
    Origin::parse("https://other.example").expect("valid test origin")
}

fn intent(value: &str) -> ActionIntentDigest {
    ActionIntentDigest::parse(value).expect("valid intent digest")
}

fn call(tool_name: &str) -> ValidatedMcpToolCall {
    ValidatedMcpToolCall::new(
        MCP_PROTOCOL_VERSION,
        MCP_TOOLS_CALL_METHOD,
        tool_name,
        MCP_TOOLS_CALL_METHOD,
        tool_name,
    )
    .expect("known test MCP tool")
}

fn request(action: ActionKind) -> ActionRequest {
    ActionRequest::new(
        action,
        origin(),
        origin(),
        InstructionSource::User,
        SecretDelivery::None,
        intent(VALID_INTENT),
    )
}

fn context(approval: ApprovalEvidence) -> PolicyContext {
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([Capability::Observe, Capability::Submit]),
        BTreeSet::from([origin(), other_origin()]),
        BTreeSet::from([origin(), other_origin()]),
        RobotsDecision::Allowed,
        approval,
    )
}

fn running() -> BapTaskLifecycle {
    BapTaskLifecycle::restore(BapTaskState::Running, 2).expect("reachable running snapshot")
}

fn permitted(admission: StepAdmission) -> DispatchPermit {
    match admission {
        StepAdmission::Permitted(permit) => Some(permit),
        StepAdmission::Suspended { .. } | StepAdmission::Denied(_) => None,
    }
    .expect("allowed step must yield a dispatch permit")
}

fn assert_permit(permit: &DispatchPermit, action: ActionKind, tool_name: &str, sequence: u64) {
    assert_eq!(permit.action(), action);
    assert_eq!(permit.tool_name(), tool_name);
    assert_eq!(permit.target_origin(), &origin());
    assert_eq!(permit.intent_digest(), &intent(VALID_INTENT));
    assert_eq!(permit.task_sequence(), sequence);
}

fn suspended(admission: StepAdmission) -> (RiskClass, BapTaskTransition) {
    match admission {
        StepAdmission::Suspended { reason, transition } => Some((reason, transition)),
        StepAdmission::Permitted(_) | StepAdmission::Denied(_) => None,
    }
    .expect("approval-gated step must suspend the task")
}

#[test]
fn require_approval_suspends_running_task_without_dispatch_permit() {
    let mut lifecycle = running();

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.submit"),
        &request(ActionKind::Submit),
        &context(ApprovalEvidence::None),
    )
    .expect("running task admits a governed decision");

    let (reason, transition) = suspended(admission);
    assert_eq!(reason, RiskClass::R3);
    assert_eq!(transition.previous_state(), BapTaskState::Running);
    assert_eq!(transition.current_state(), BapTaskState::WaitingForApproval);
    assert_eq!(transition.sequence(), 3);
    assert_eq!(lifecycle.state(), BapTaskState::WaitingForApproval);
    assert_eq!(lifecycle.transition_sequence(), 3);
}

#[test]
fn allow_mints_permit_bound_to_action_target_intent_and_sequence() {
    let mut lifecycle = running();
    let before = lifecycle;

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.observe"),
        &request(ActionKind::Observe),
        &context(ApprovalEvidence::None),
    )
    .expect("running task admits a governed decision");

    let permit = permitted(admission);
    assert_eq!(permit.action(), ActionKind::Observe);
    assert_eq!(permit.tool_name(), "originweave.observe");
    assert_eq!(permit.target_origin(), &origin());
    assert_eq!(permit.intent_digest(), &intent(VALID_INTENT));
    assert_eq!(permit.task_sequence(), 2);
    assert_eq!(lifecycle, before, "a permit must not change the lifecycle");

    let (action, tool_name, target, digest, sequence) = permit.into_parts();
    assert_eq!(action, ActionKind::Observe);
    assert_eq!(tool_name, "originweave.observe");
    assert_eq!(target, origin());
    assert_eq!(digest, intent(VALID_INTENT));
    assert_eq!(sequence, 2);
}

#[test]
fn exact_approval_permits_high_risk_step_without_suspension() {
    let mut lifecycle = running();
    let scope = ApprovalScope::new(ActionKind::Submit, origin(), intent(VALID_INTENT));

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.submit"),
        &request(ActionKind::Submit),
        &context(ApprovalEvidence::UserConfirmed(scope)),
    )
    .expect("running task admits a governed decision");

    let permit = permitted(admission);
    assert_permit(&permit, ActionKind::Submit, "originweave.submit", 2);
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), 2);
}

#[test]
fn enterprise_policy_approval_permits_high_risk_step() {
    let mut lifecycle = running();
    let scope = ApprovalScope::new(ActionKind::Submit, origin(), intent(VALID_INTENT));

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.submit"),
        &request(ActionKind::Submit),
        &context(ApprovalEvidence::EnterprisePolicy(scope)),
    )
    .expect("running task admits a governed decision");

    assert_permit(
        &permitted(admission),
        ActionKind::Submit,
        "originweave.submit",
        2,
    );
    assert_eq!(lifecycle, running());
}

#[test]
fn permit_records_a_later_running_sequence() {
    let mut lifecycle =
        BapTaskLifecycle::restore(BapTaskState::Running, 8).expect("reachable running snapshot");

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.observe"),
        &request(ActionKind::Observe),
        &context(ApprovalEvidence::None),
    )
    .expect("running task admits a governed decision");

    assert_permit(
        &permitted(admission),
        ActionKind::Observe,
        "originweave.observe",
        8,
    );
    assert_eq!(lifecycle.transition_sequence(), 8);
}

#[test]
fn policy_denial_leaves_lifecycle_unchanged() {
    let cases = [
        (
            call("originweave.observe"),
            request(ActionKind::Submit),
            ApprovalEvidence::None,
            DenialReason::McpActionMismatch,
        ),
        (
            call("originweave.submit"),
            ActionRequest::new(
                ActionKind::Submit,
                origin(),
                other_origin(),
                InstructionSource::User,
                SecretDelivery::None,
                intent(VALID_INTENT),
            ),
            ApprovalEvidence::None,
            DenialReason::CrossOriginMutation,
        ),
        (
            call("originweave.submit"),
            request(ActionKind::Submit),
            ApprovalEvidence::UserConfirmed(ApprovalScope::new(
                ActionKind::Submit,
                origin(),
                intent(OTHER_INTENT),
            )),
            DenialReason::ApprovalScopeMismatch,
        ),
    ];

    for (route, proposed, approval, expected) in cases {
        let mut lifecycle = running();
        let before = lifecycle;

        let admission = admit_step(&mut lifecycle, &route, &proposed, &context(approval))
            .expect("running task admits a governed decision");

        assert_eq!(admission, StepAdmission::Denied(expected));
        assert_eq!(lifecycle, before, "a denial must not change the lifecycle");
    }
}

#[test]
fn non_running_task_is_rejected_before_policy_evaluation() {
    let snapshots = [
        (BapTaskState::Created, 0),
        (BapTaskState::Admitted, 1),
        (BapTaskState::WaitingForApproval, 3),
        (BapTaskState::WaitingForExternalInput, 3),
        (BapTaskState::Checkpointed, 3),
        (BapTaskState::ReconciliationRequired, 3),
        (BapTaskState::Succeeded, 3),
        (BapTaskState::Failed, 1),
        (BapTaskState::Cancelled, 1),
        (BapTaskState::Expired, 1),
        (BapTaskState::DeadLettered, 3),
    ];

    for (state, sequence) in snapshots {
        let mut lifecycle =
            BapTaskLifecycle::restore(state, sequence).expect("reachable test snapshot");
        let before = lifecycle;

        // Observe is policy-allowed, so only the lifecycle gate can reject it.
        let result = admit_step(
            &mut lifecycle,
            &call("originweave.observe"),
            &request(ActionKind::Observe),
            &context(ApprovalEvidence::None),
        );

        assert_eq!(result, Err(StepAdmissionError::TaskNotRunning { state }));
        assert_eq!(
            lifecycle, before,
            "a rejected step must not change the lifecycle"
        );

        // A policy-denied step and an approval-required step must also be
        // rejected by the lifecycle gate, before policy evaluation.
        let denied = admit_step(
            &mut lifecycle,
            &call("originweave.observe"),
            &request(ActionKind::Submit),
            &context(ApprovalEvidence::None),
        );
        assert_eq!(denied, Err(StepAdmissionError::TaskNotRunning { state }));

        let approval_required = admit_step(
            &mut lifecycle,
            &call("originweave.submit"),
            &request(ActionKind::Submit),
            &context(ApprovalEvidence::None),
        );
        assert_eq!(
            approval_required,
            Err(StepAdmissionError::TaskNotRunning { state })
        );
        assert_eq!(
            lifecycle, before,
            "a rejected step must not change the lifecycle"
        );
    }
}

#[test]
fn suspension_at_largest_running_sequence_records_final_sequence() {
    // Running sequences are even, so u64::MAX - 1 is the largest reachable one.
    // The suspension transition must still be recorded at exactly u64::MAX.
    let mut lifecycle = BapTaskLifecycle::restore(BapTaskState::Running, u64::MAX - 1)
        .expect("reachable running snapshot");

    let admission = admit_step(
        &mut lifecycle,
        &call("originweave.submit"),
        &request(ActionKind::Submit),
        &context(ApprovalEvidence::None),
    )
    .expect("one more transition is representable");

    let (reason, transition) = suspended(admission);
    assert_eq!(reason, RiskClass::R3);
    assert_eq!(transition.current_state(), BapTaskState::WaitingForApproval);
    assert_eq!(transition.sequence(), u64::MAX);
    assert_eq!(lifecycle.state(), BapTaskState::WaitingForApproval);
    assert_eq!(lifecycle.transition_sequence(), u64::MAX);
}

#[test]
fn not_running_error_has_stable_display_and_no_source() {
    let not_running = StepAdmissionError::TaskNotRunning {
        state: BapTaskState::Checkpointed,
    };
    assert_eq!(
        not_running.to_string(),
        "agent task step requires a running task, but the task is Checkpointed"
    );
    assert!(std::error::Error::source(&not_running).is_none());
}
