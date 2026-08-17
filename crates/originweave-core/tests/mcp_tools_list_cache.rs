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

fn valid_tools_list_request(
    cursor: Option<&str>,
) -> Result<ValidatedMcpToolsListRequest, McpToolsListBoundaryError> {
    ValidatedMcpToolsListRequest::new(
        Some(MCP_PROTOCOL_VERSION),
        Some(MCP_PROTOCOL_VERSION),
        true,
        MCP_TOOLS_LIST_METHOD,
        MCP_TOOLS_LIST_METHOD,
        cursor,
    )
}

#[test]
fn mcp_tools_list_request_requires_complete_request_metadata() {
    assert_eq!(
        valid_tools_list_request(None).map(|validated| validated.method()),
        Ok(MCP_TOOLS_LIST_METHOD)
    );

    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            None,
            Some(MCP_PROTOCOL_VERSION),
            true,
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::MissingProtocolVersionHeader)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some(MCP_PROTOCOL_VERSION),
            None,
            true,
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::MissingProtocolVersionMetadata)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some(MCP_PROTOCOL_VERSION),
            Some("2025-11-25"),
            true,
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::ProtocolVersionHeaderBodyMismatch)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some("2025-11-25"),
            Some("2025-11-25"),
            true,
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some(MCP_PROTOCOL_VERSION),
            Some(MCP_PROTOCOL_VERSION),
            false,
            MCP_TOOLS_LIST_METHOD,
            MCP_TOOLS_LIST_METHOD,
            None,
        ),
        Err(McpToolsListBoundaryError::MissingClientCapabilities)
    );
}

#[test]
fn mcp_tools_list_request_requires_exact_routing_and_no_unissued_cursor() {
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some(MCP_PROTOCOL_VERSION),
            Some(MCP_PROTOCOL_VERSION),
            true,
            MCP_TOOLS_LIST_METHOD,
            "tools/call",
            None,
        ),
        Err(McpToolsListBoundaryError::MethodHeaderBodyMismatch)
    );
    assert_eq!(
        ValidatedMcpToolsListRequest::new(
            Some(MCP_PROTOCOL_VERSION),
            Some(MCP_PROTOCOL_VERSION),
            true,
            "resources/list",
            "resources/list",
            None,
        ),
        Err(McpToolsListBoundaryError::UnsupportedMethod)
    );

    for cursor in ["cursor-1", ""] {
        assert_eq!(
            valid_tools_list_request(Some(cursor)),
            Err(McpToolsListBoundaryError::UnsupportedCursor)
        );
    }
}

#[test]
fn mcp_tools_list_request_errors_are_source_free_and_non_echoing() {
    let cases = [
        (
            McpToolsListBoundaryError::MissingProtocolVersionHeader,
            "MCP protocol version header is required",
        ),
        (
            McpToolsListBoundaryError::MissingProtocolVersionMetadata,
            "MCP request metadata protocol version is required",
        ),
        (
            McpToolsListBoundaryError::ProtocolVersionHeaderBodyMismatch,
            "MCP protocol version header does not match request metadata",
        ),
        (
            McpToolsListBoundaryError::UnsupportedProtocolVersion,
            "unsupported MCP protocol version",
        ),
        (
            McpToolsListBoundaryError::MissingClientCapabilities,
            "MCP request metadata client capabilities are required",
        ),
        (
            McpToolsListBoundaryError::MethodHeaderBodyMismatch,
            "MCP method header does not match the request body",
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
