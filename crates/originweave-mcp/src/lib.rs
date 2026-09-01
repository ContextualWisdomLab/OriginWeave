//! Fail-closed MCP adapter contracts for OriginWeave.
//!
//! This crate owns MCP protocol-generation, discovery, and stateless tool-routing
//! contracts. It maps reviewed MCP protocol values into existing OriginWeave
//! action contracts but grants no policy, browser, network, secret, or evidence
//! authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use originweave_core::{ActionRequest, PolicyContext};
use originweave_policy::{Decision, DenialReason};

pub(crate) use originweave_core::{ActionKind, Capability, RiskClass};

mod routing;

pub use routing::*;

/// Evaluate one validated MCP route through the ordinary OriginWeave policy boundary.
///
/// Route validation proves only protocol integrity. It grants no capability, origin, approval,
/// secret, browser, network, or evidence authority. A route/action mismatch fails closed before
/// the request is delegated to the protocol-independent policy evaluator.
#[must_use]
pub fn evaluate_mcp(
    call: &ValidatedMcpToolCall,
    request: &ActionRequest,
    context: &PolicyContext,
) -> Decision {
    if call.action_kind() != request.action() {
        return Decision::Deny(DenialReason::McpActionMismatch);
    }

    originweave_policy::evaluate(request, context)
}
