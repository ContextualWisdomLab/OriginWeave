use std::error::Error;

use originweave_core::mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_LIST_METHOD, McpCacheScope, McpResultType,
    McpToolsListBoundaryError, ValidatedMcpToolsListRequest, mcp_tools_list_page,
    supported_mcp_tools,
};

#[test]
fn mcp_tools_list_page_is_complete_private_and_immediately_stale() {
    let page = mcp_tools_list_page();

    assert_eq!(page.result_type(), McpResultType::Complete);
    assert_eq!(page.tools(), supported_mcp_tools());
    assert_eq!(page.ttl_ms(), 0);
    assert_eq!(page.cache_scope(), McpCacheScope::Private);
    assert_eq!(page.next_cursor(), None);
}

#[test]
fn mcp_tools_list_request_requires_exact_routing_and_no_unissued_cursor() {
    let request = ValidatedMcpToolsListRequest::new(
        MCP_PROTOCOL_VERSION,
        MCP_TOOLS_LIST_METHOD,
        MCP_TOOLS_LIST_METHOD,
        None,
    );
    assert_eq!(
        request.map(|validated| validated.method()),
        Ok(MCP_TOOLS_LIST_METHOD)
    );

    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            "2025-11-25",
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            MCP_PROTOCOL_VERSION,
            MCP_TOOLS_LIST_METHOD,
            "tools/call",
            None,
        ),
        Err(McpToolsListBoundaryError::HeaderBodyMismatch)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            MCP_PROTOCOL_VERSION,
            "resources/list",
            "resources/list",
            None,
        ),
        Err(McpToolsListBoundaryError::UnsupportedMethod)
    );

    for cursor in ["cursor-1", ""] {
        assert_eq!(
            ValidatedMcpToolsListRequest::new(
                MCP_PROTOCOL_VERSION,
                MCP_TOOLS_LIST_METHOD,
                MCP_TOOLS_LIST_METHOD,
                Some(cursor),
            ),
            Err(McpToolsListBoundaryError::UnsupportedCursor)
        );
    }
}

#[test]
fn mcp_tools_list_request_errors_are_source_free_and_non_echoing() {
    let cases = [
        (
            McpToolsListBoundaryError::UnsupportedProtocolVersion,
            "unsupported MCP protocol version",
        ),
        (
            McpToolsListBoundaryError::HeaderBodyMismatch,
            "MCP routing headers do not match the request body",
        ),
        (
            McpToolsListBoundaryError::UnsupportedMethod,
            "only MCP tools/list requests can enter the discovery boundary",
        ),
        (
            McpToolsListBoundaryError::UnsupportedCursor,
            "MCP tools/list cursor was not issued by this fixed catalog",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
