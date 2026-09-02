use originweave_mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, ValidatedMcpToolCall,
};

/// MCP 2026-07-28 makes the protocol version and client capabilities
/// self-describing on every request. The final protocol keeps `clientInfo`
/// optional and non-authoritative, so absence of client identity must not be an
/// admission failure. The current `tools/call` constructor still has no input
/// for the required request `_meta` protocol version or per-request client
/// capabilities, so a call built only from the transport/routing fields must
/// not be admitted as a fully validated modern request.
#[test]
fn tools_call_without_required_per_request_metadata_fails_closed() {
    let result = ValidatedMcpToolCall::new(
        MCP_PROTOCOL_VERSION,
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
    );

    assert!(
        result.is_err(),
        "MCP 2026-07-28 tools/call must not validate without request protocol-version metadata and per-request client capabilities"
    );
}
