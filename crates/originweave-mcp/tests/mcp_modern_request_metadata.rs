use originweave_mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, ValidatedMcpToolCall,
};

/// MCP 2026-07-28 makes every request self-describing. The current tools/call
/// constructor has no input for the required per-request client identity or
/// capabilities, so a call built only from protocol/routing fields must not be
/// admitted as a fully validated modern request.
#[test]
fn tools_call_without_per_request_client_metadata_fails_closed() {
    let result = ValidatedMcpToolCall::new(
        MCP_PROTOCOL_VERSION,
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
    );

    assert!(
        result.is_err(),
        "MCP 2026-07-28 tools/call must not validate without per-request client identity and capabilities"
    );
}
