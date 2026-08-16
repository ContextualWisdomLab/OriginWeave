use std::error::Error;

use originweave_core::mcp::{
    MAX_MCP_TOOL_NAME_BYTES, MCP_PROTOCOL_VERSION, MCP_TOOLS_CALL_METHOD, McpToolBoundaryError,
    ValidatedMcpToolCall, supported_mcp_tools,
};
use originweave_core::{ActionKind, Capability, RiskClass};

fn validate(tool_name: &str) -> Result<ValidatedMcpToolCall, McpToolBoundaryError> {
    ValidatedMcpToolCall::new(
        MCP_PROTOCOL_VERSION,
        MCP_TOOLS_CALL_METHOD,
        tool_name,
        MCP_TOOLS_CALL_METHOD,
        tool_name,
    )
}

#[test]
fn supported_mcp_tools_map_to_exact_originweave_actions() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            "originweave.observe",
            ActionKind::Observe,
            Capability::Observe,
            RiskClass::R0,
        ),
        (
            "originweave.extract",
            ActionKind::Extract,
            Capability::Extract,
            RiskClass::R0,
        ),
        (
            "originweave.navigate",
            ActionKind::Navigate,
            Capability::Navigate,
            RiskClass::R1,
        ),
        (
            "originweave.download",
            ActionKind::Download,
            Capability::Download,
            RiskClass::R1,
        ),
        (
            "originweave.draft",
            ActionKind::Draft,
            Capability::Draft,
            RiskClass::R2,
        ),
        (
            "originweave.submit",
            ActionKind::Submit,
            Capability::Submit,
            RiskClass::R3,
        ),
        (
            "originweave.upload",
            ActionKind::Upload,
            Capability::Upload,
            RiskClass::R3,
        ),
        (
            "originweave.fill_secret",
            ActionKind::FillSecret,
            Capability::FillSecret,
            RiskClass::R3,
        ),
        (
            "originweave.purchase",
            ActionKind::Purchase,
            Capability::Purchase,
            RiskClass::R4,
        ),
        (
            "originweave.delete",
            ActionKind::Delete,
            Capability::Delete,
            RiskClass::R4,
        ),
        (
            "originweave.manage_permission",
            ActionKind::ManagePermission,
            Capability::ManagePermission,
            RiskClass::R4,
        ),
    ];

    for (tool_name, expected_action, expected_capability, expected_risk) in cases {
        let call = validate(tool_name)?;
        assert_eq!(call.tool_name(), tool_name);
        assert_eq!(call.action_kind(), expected_action);
        assert_eq!(
            call.action_kind().required_capability(),
            expected_capability
        );
        assert_eq!(call.action_kind().risk_class(), expected_risk);
    }
    Ok(())
}

#[test]
fn mcp_tool_catalog_is_deterministic_complete_and_action_unambiguous() -> Result<(), Box<dyn Error>>
{
    let expected = [
        ("originweave.observe", ActionKind::Observe),
        ("originweave.extract", ActionKind::Extract),
        ("originweave.navigate", ActionKind::Navigate),
        ("originweave.download", ActionKind::Download),
        ("originweave.draft", ActionKind::Draft),
        ("originweave.submit", ActionKind::Submit),
        ("originweave.upload", ActionKind::Upload),
        ("originweave.fill_secret", ActionKind::FillSecret),
        ("originweave.purchase", ActionKind::Purchase),
        ("originweave.delete", ActionKind::Delete),
        (
            "originweave.manage_permission",
            ActionKind::ManagePermission,
        ),
    ];
    let catalog = supported_mcp_tools();

    assert_eq!(catalog.len(), expected.len());
    for (entry, (expected_name, expected_action)) in catalog.iter().zip(expected) {
        assert_eq!(entry.tool_name(), expected_name);
        assert_eq!(entry.action_kind(), expected_action);
        assert_eq!(
            entry.required_capability(),
            expected_action.required_capability()
        );
        assert_eq!(entry.risk_class(), expected_action.risk_class());

        let call = validate(entry.tool_name())?;
        assert_eq!(call.action_kind(), entry.action_kind());
    }

    for (index, entry) in catalog.iter().enumerate() {
        for other in &catalog[index + 1..] {
            assert_ne!(entry.tool_name(), other.tool_name());
            assert_ne!(entry.action_kind(), other.action_kind());
        }
    }
    assert!(
        catalog
            .iter()
            .all(|entry| entry.action_kind() != ActionKind::LegalConsent)
    );
    Ok(())
}

#[test]
fn mcp_route_rejects_protocol_header_body_and_method_drift() {
    assert_eq!(
        ValidatedMcpToolCall::new(
            "2025-11-25",
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        ValidatedMcpToolCall::new(
            MCP_PROTOCOL_VERSION,
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
            "tools/list",
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::HeaderBodyMismatch)
    );
    assert_eq!(
        ValidatedMcpToolCall::new(
            MCP_PROTOCOL_VERSION,
            MCP_TOOLS_CALL_METHOD,
            "originweave.observe",
            MCP_TOOLS_CALL_METHOD,
            "originweave.extract",
        ),
        Err(McpToolBoundaryError::HeaderBodyMismatch)
    );
    assert_eq!(
        ValidatedMcpToolCall::new(
            MCP_PROTOCOL_VERSION,
            "resources/read",
            "originweave.observe",
            "resources/read",
            "originweave.observe",
        ),
        Err(McpToolBoundaryError::UnsupportedMethod)
    );
}

#[test]
fn mcp_route_rejects_unbounded_malformed_and_unmapped_tool_names() {
    let at_limit = "x".repeat(MAX_MCP_TOOL_NAME_BYTES);
    let oversized = "x".repeat(MAX_MCP_TOOL_NAME_BYTES + 1);
    for tool_name in [
        "",
        "originweave legal",
        "originweave/observe",
        "originweave.관찰",
        &oversized,
    ] {
        assert_eq!(
            validate(tool_name),
            Err(McpToolBoundaryError::InvalidToolName)
        );
    }

    assert_eq!(validate(&at_limit), Err(McpToolBoundaryError::UnknownTool));
    assert_eq!(
        validate("originweave.legal_consent"),
        Err(McpToolBoundaryError::UnknownTool)
    );
    assert_eq!(
        validate("third_party.arbitrary_javascript"),
        Err(McpToolBoundaryError::UnknownTool)
    );
}

#[test]
fn mcp_boundary_errors_are_deterministic_and_do_not_echo_untrusted_values() {
    let cases = [
        (
            McpToolBoundaryError::UnsupportedProtocolVersion,
            "unsupported MCP protocol version",
        ),
        (
            McpToolBoundaryError::HeaderBodyMismatch,
            "MCP routing headers do not match the request body",
        ),
        (
            McpToolBoundaryError::UnsupportedMethod,
            "only MCP tools/call requests can enter the typed action boundary",
        ),
        (
            McpToolBoundaryError::InvalidToolName,
            "MCP tool name violates the bounded ASCII routing syntax",
        ),
        (
            McpToolBoundaryError::UnknownTool,
            "MCP tool is not mapped to an OriginWeave typed action",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(error.source().is_none());
    }
}
