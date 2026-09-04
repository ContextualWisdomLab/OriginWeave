use originweave_mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, MCP_TOOLS_LIST_METHOD, McpToolBoundaryError,
    McpToolsListBoundaryError, ValidatedMcpToolCall, ValidatedMcpToolsListRequest,
};

#[test]
fn modern_stdio_tools_call_admits_body_metadata_without_http_headers() {
    let call = ValidatedMcpToolCall::new_for_stdio(
        Some(MCP_PROTOCOL_VERSION),
        true,
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
    );

    assert_eq!(
        call.as_ref().map(ValidatedMcpToolCall::tool_name),
        Ok("originweave.observe")
    );
}

#[test]
fn modern_stdio_tools_call_still_requires_body_protocol_metadata_and_capabilities() {
    assert_eq!(
        ValidatedMcpToolCall::new_for_stdio(
            None,
            true,
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::MissingProtocolVersionMetadata)
    );
    assert_eq!(
        ValidatedMcpToolCall::new_for_stdio(
            Some(MCP_PROTOCOL_VERSION),
            false,
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::MissingClientCapabilities)
    );
}

#[test]
fn modern_stdio_tools_list_admits_body_metadata_without_http_headers() {
    let request = ValidatedMcpToolsListRequest::new_for_stdio(
        Some(MCP_PROTOCOL_VERSION),
        true,
        MCP_TOOLS_LIST_METHOD,
        None,
    );

    assert_eq!(
        request.as_ref().map(ValidatedMcpToolsListRequest::method),
        Ok(MCP_TOOLS_LIST_METHOD)
    );
}

#[test]
fn modern_stdio_tools_list_still_requires_body_protocol_metadata_and_capabilities() {
    assert_eq!(
        ValidatedMcpToolsListRequest::new_for_stdio(None, true, MCP_TOOLS_LIST_METHOD, None,),
        Err(McpToolsListBoundaryError::MissingProtocolVersionMetadata)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new_for_stdio(
            Some(MCP_PROTOCOL_VERSION),
            false,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::MissingClientCapabilities)
    );
}
