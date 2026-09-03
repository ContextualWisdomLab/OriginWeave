//! Fail-closed MCP adapter contracts for OriginWeave.
//!
//! This crate owns MCP protocol-generation, discovery, and stateless tool-routing
//! contracts. It maps reviewed MCP protocol values into existing OriginWeave
//! action contracts but grants no policy, browser, network, secret, or evidence
//! authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use originweave_core::{ActionRequest, PolicyContext};
use originweave_policy::Decision;

pub(crate) use originweave_core::{ActionKind, Capability, RiskClass};

mod request;
mod routing;

pub use request::{McpToolBoundaryError, ValidatedMcpToolCall};
pub use routing::{
    MAX_MCP_METHOD_NAME_BYTES, MAX_MCP_TOOL_NAME_BYTES, MCP_PROTOCOL_VERSION,
    MCP_TOOLS_CALL_METHOD, MCP_TOOLS_LIST_METHOD, McpCacheScope, McpResultType,
    McpToolCatalogEntry, McpToolsListBoundaryError, McpToolsListPage, ValidatedMcpToolsListRequest,
    mcp_tools_list_page, supported_mcp_tools,
};

/// A fail-closed rejection owned by the MCP routing boundary rather than policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpRouteRejection {
    /// The validated MCP route resolves to a different action than the typed request.
    ActionMismatch,
}

/// Evaluate one validated MCP route through the ordinary OriginWeave policy boundary.
///
/// Route validation proves only protocol integrity. It grants no capability, origin, approval,
/// secret, browser, network, or evidence authority. A route/action mismatch is returned as an
/// MCP-owned rejection before the request reaches policy. Callers may execute only
/// `Ok(Decision::Allow)`; every other result remains non-authorizing.
pub fn evaluate_mcp(
    call: &ValidatedMcpToolCall,
    request: &ActionRequest,
    context: &PolicyContext,
) -> Result<Decision, McpRouteRejection> {
    if call.action_kind() != request.action() {
        return Err(McpRouteRejection::ActionMismatch);
    }

    Ok(originweave_policy::evaluate(request, context))
}
