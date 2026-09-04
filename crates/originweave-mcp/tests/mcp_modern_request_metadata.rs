use originweave_mcp::{
    MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, McpToolBoundaryError, ValidatedMcpToolCall,
};

fn modern_call(
    protocol_version_header: Option<&str>,
    protocol_version_metadata: Option<&str>,
    client_capabilities_present: bool,
) -> Result<ValidatedMcpToolCall, McpToolBoundaryError> {
    ValidatedMcpToolCall::new_with_request_metadata(
        protocol_version_header,
        protocol_version_metadata,
        client_capabilities_present,
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
        MCP_TOOLS_CALL_METHOD,
        "originweave.observe",
    )
}

#[test]
fn legacy_tools_call_shape_fails_closed_without_request_metadata() {
    assert_eq!(
        ValidatedMcpToolCall::new(
            MCP_PROTOCOL_VERSION,
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::MissingProtocolVersionMetadata)
    );
}

#[test]
fn modern_tools_call_requires_both_protocol_version_surfaces() {
    assert_eq!(
        modern_call(None, Some(MCP_PROTOCOL_VERSION), true),
        Err(McpToolBoundaryError::MissingProtocolVersionHeader)
    );
    assert_eq!(
        modern_call(Some(MCP_PROTOCOL_VERSION), None, true),
        Err(McpToolBoundaryError::MissingProtocolVersionMetadata)
    );
}

#[test]
fn modern_tools_call_bounds_and_cross_checks_protocol_versions() {
    let oversized = format!("{MCP_PROTOCOL_VERSION}x");

    assert_eq!(
        modern_call(Some(&oversized), Some(MCP_PROTOCOL_VERSION), true),
        Err(McpToolBoundaryError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        modern_call(Some(MCP_PROTOCOL_VERSION), Some(&oversized), true),
        Err(McpToolBoundaryError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        modern_call(Some(MCP_PROTOCOL_VERSION), Some("2025-11-25"), true),
        Err(McpToolBoundaryError::ProtocolVersionHeaderBodyMismatch)
    );
    assert_eq!(
        modern_call(Some("2025-11-25"), Some("2025-11-25"), true),
        Err(McpToolBoundaryError::UnsupportedProtocolVersion)
    );
}

#[test]
fn modern_tools_call_requires_per_request_client_capabilities() {
    assert_eq!(
        modern_call(
            Some(MCP_PROTOCOL_VERSION),
            Some(MCP_PROTOCOL_VERSION),
            false,
        ),
        Err(McpToolBoundaryError::MissingClientCapabilities)
    );

    let call = modern_call(Some(MCP_PROTOCOL_VERSION), Some(MCP_PROTOCOL_VERSION), true);
    assert_eq!(
        call.as_ref().map(ValidatedMcpToolCall::tool_name),
        Ok("originweave.observe")
    );
}
