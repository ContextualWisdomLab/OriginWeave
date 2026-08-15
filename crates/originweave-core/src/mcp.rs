//! Fail-closed MCP routing integrity for the external adapter boundary.
//!
//! This module validates only the stateless MCP protocol/method/tool routing
//! envelope and derives an existing [`ActionKind`]. It is deliberately not an
//! authorization decision: callers must independently enforce OriginWeave
//! capability, risk, approval, origin, secret-broker, and evidence policies.
//! No MCP arguments, outputs, credentials, or arbitrary model-visible values
//! are retained by this boundary.

use std::fmt;

use crate::ActionKind;

/// MCP protocol generation accepted by this stateless adapter boundary.
pub const MCP_PROTOCOL_VERSION: &str = "2026-07-28";

/// The only MCP method that can enter the typed action-routing boundary.
pub const MCP_TOOLS_CALL_METHOD: &str = "tools/call";

/// Maximum accepted MCP tool-name length in bytes.
pub const MAX_MCP_TOOL_NAME_BYTES: usize = 128;

/// The complete explicit MCP tool-to-action mapping accepted by this boundary.
const MCP_TOOL_ACTION_MAP: &[(&str, ActionKind)] = &[
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

/// A deterministic failure while validating untrusted MCP routing metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpToolBoundaryError {
    /// The request names an MCP protocol generation this boundary does not support.
    UnsupportedProtocolVersion,
    /// MCP routing metadata disagrees with the method or tool name in the body.
    HeaderBodyMismatch,
    /// The request method is not the supported `tools/call` operation.
    UnsupportedMethod,
    /// The tool name violates the bounded ASCII MCP routing syntax.
    InvalidToolName,
    /// The tool name has no explicit mapping to an OriginWeave typed action.
    UnknownTool,
}

impl fmt::Display for McpToolBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion => {
                formatter.write_str("unsupported MCP protocol version")
            }
            Self::HeaderBodyMismatch => {
                formatter.write_str("MCP routing headers do not match the request body")
            }
            Self::UnsupportedMethod => formatter
                .write_str("only MCP tools/call requests can enter the typed action boundary"),
            Self::InvalidToolName => {
                formatter.write_str("MCP tool name violates the bounded ASCII routing syntax")
            }
            Self::UnknownTool => {
                formatter.write_str("MCP tool is not mapped to an OriginWeave typed action")
            }
        }
    }
}

impl std::error::Error for McpToolBoundaryError {}

/// An MCP tool call whose routing envelope has been validated and mapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedMcpToolCall {
    tool_name: &'static str,
    action_kind: ActionKind,
}

impl ValidatedMcpToolCall {
    /// Validate one stateless MCP tool-call routing envelope.
    ///
    /// Routing integrity is intentionally narrower than authorization. A
    /// successful value proves only that the untrusted protocol version,
    /// routing metadata, body method, and body tool name agree with one
    /// explicitly supported mapping.
    pub fn new(
        protocol_version: &str,
        routing_method: &str,
        routing_tool_name: &str,
        body_method: &str,
        body_tool_name: &str,
    ) -> Result<Self, McpToolBoundaryError> {
        if protocol_version != MCP_PROTOCOL_VERSION {
            return Err(McpToolBoundaryError::UnsupportedProtocolVersion);
        }
        if routing_method != body_method || routing_tool_name != body_tool_name {
            return Err(McpToolBoundaryError::HeaderBodyMismatch);
        }
        if routing_method != MCP_TOOLS_CALL_METHOD {
            return Err(McpToolBoundaryError::UnsupportedMethod);
        }
        if !valid_tool_name(routing_tool_name) {
            return Err(McpToolBoundaryError::InvalidToolName);
        }

        let (tool_name, action_kind) = map_tool(routing_tool_name)?;
        Ok(Self {
            tool_name,
            action_kind,
        })
    }

    /// Return the canonical static tool name selected by the explicit mapping.
    #[must_use]
    pub const fn tool_name(&self) -> &'static str {
        self.tool_name
    }

    /// Return the existing OriginWeave typed action selected by this tool.
    #[must_use]
    pub const fn action_kind(&self) -> ActionKind {
        self.action_kind
    }
}

fn valid_tool_name(tool_name: &str) -> bool {
    if tool_name.is_empty() || tool_name.len() > MAX_MCP_TOOL_NAME_BYTES {
        return false;
    }
    tool_name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn map_tool(tool_name: &str) -> Result<(&'static str, ActionKind), McpToolBoundaryError> {
    MCP_TOOL_ACTION_MAP
        .iter()
        .copied()
        .find(|(mapped_name, _)| *mapped_name == tool_name)
        .ok_or(McpToolBoundaryError::UnknownTool)
}
