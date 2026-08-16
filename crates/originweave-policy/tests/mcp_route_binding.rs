#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, ValidatedMcpToolCall,
};
use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, Capability, ExecutionPurpose,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
};
use originweave_policy::{Decision, DenialReason, evaluate_mcp};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn origin() -> Origin {
    Origin::parse("https://mcp.example").expect("valid test origin")
}

fn intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(VALID_INTENT).expect("valid intent digest")
}

fn validated_call(tool_name: &str) -> ValidatedMcpToolCall {
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
    let site = origin();
    ActionRequest::new(
        action,
        site.clone(),
        site,
        InstructionSource::User,
        SecretDelivery::None,
        intent(),
    )
}

fn context(capabilities: BTreeSet<Capability>) -> PolicyContext {
    let site = origin();
    PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        capabilities,
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    )
}

#[test]
fn matching_mcp_route_enters_the_existing_policy_boundary() {
    let call = validated_call("originweave.observe");
    let decision = evaluate_mcp(
        &call,
        &request(ActionKind::Observe),
        &context(BTreeSet::from([Capability::Observe])),
    );

    assert_eq!(decision, Decision::Allow);
}

#[test]
fn mismatched_mcp_route_cannot_be_reinterpreted_as_another_action() {
    let call = validated_call("originweave.observe");
    let decision = evaluate_mcp(
        &call,
        &request(ActionKind::Navigate),
        &context(BTreeSet::from([Capability::Navigate])),
    );

    assert_eq!(decision, Decision::Deny(DenialReason::McpActionMismatch));
}

#[test]
fn matching_mcp_route_does_not_bypass_existing_policy_denials() {
    let call = validated_call("originweave.navigate");
    let decision = evaluate_mcp(
        &call,
        &request(ActionKind::Navigate),
        &context(BTreeSet::from([Capability::Observe])),
    );

    assert_eq!(
        decision,
        Decision::Deny(DenialReason::MissingCapability(Capability::Navigate))
    );
}
