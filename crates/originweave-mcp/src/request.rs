//! MCP 2026-07-28 request-envelope validation for typed tool calls.
//!
//! This adapter layer binds transport protocol metadata to the existing bounded
//! tool-routing validator. It deliberately retains no client identity or
//! capability contents and grants no OriginWeave browser, policy, secret, or
//! evidence authority.

use std::fmt;

use crate::{ActionKind, MCP_PROTOCOL_VERSION, routing};

/// A deterministic failure while validating one MCP `tools/call` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpToolBoundaryError {
    /// The transport request omitted the required MCP protocol-version header.
    MissingProtocolVersionHeader,
    /// The structured request metadata omitted the required MCP protocol version.
    MissingProtocolVersionMetadata,
    /// The transport protocol version disagrees with the structured request metadata.
    ProtocolVersionHeaderBodyMismatch,
    /// The request names an MCP protocol generation this adapter does not support.
    UnsupportedProtocolVersion,
    /// The structured request metadata omitted the required client-capabilities object.
    MissingClientCapabilities,
    /// MCP routing metadata disagrees with the method or tool name in the body.
    HeaderBodyMismatch,
    /// The request method violates the bounded ASCII MCP routing syntax.
    InvalidMethod,
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
            Self::MissingProtocolVersionHeader => {
                formatter.write_str("MCP protocol version header is required")
            }
            Self::MissingProtocolVersionMetadata => {
                formatter.write_str("MCP request metadata protocol version is required")
            }
            Self::ProtocolVersionHeaderBodyMismatch => {
                formatter.write_str("MCP protocol version header does not match request metadata")
            }
            Self::UnsupportedProtocolVersion => {
                formatter.write_str("unsupported MCP protocol version")
            }
            Self::MissingClientCapabilities => {
                formatter.write_str("MCP request metadata client capabilities are required")
            }
            Self::HeaderBodyMismatch => {
                formatter.write_str("MCP routing headers do not match the request body")
            }
            Self::InvalidMethod => {
                formatter.write_str("MCP method violates the bounded ASCII routing syntax")
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

impl From<routing::McpToolBoundaryError> for McpToolBoundaryError {
    fn from(error: routing::McpToolBoundaryError) -> Self {
        match error {
            routing::McpToolBoundaryError::UnsupportedProtocolVersion => {
                Self::UnsupportedProtocolVersion
            }
            routing::McpToolBoundaryError::HeaderBodyMismatch => Self::HeaderBodyMismatch,
            routing::McpToolBoundaryError::InvalidMethod => Self::InvalidMethod,
            routing::McpToolBoundaryError::UnsupportedMethod => Self::UnsupportedMethod,
            routing::McpToolBoundaryError::InvalidToolName => Self::InvalidToolName,
            routing::McpToolBoundaryError::UnknownTool => Self::UnknownTool,
        }
    }
}

/// An MCP tool call whose required request metadata and routing envelope were validated.
///
/// The value proves protocol-envelope integrity only. Client capability contents and optional
/// `clientInfo` are deliberately not retained because self-reported client metadata is not an
/// OriginWeave authorization signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedMcpToolCall {
    routed: routing::ValidatedMcpToolCall,
}

impl ValidatedMcpToolCall {
    /// Fail closed for the pre-2026-07-28 constructor shape.
    ///
    /// This compatibility surface preserves deterministic routing diagnostics for malformed
    /// legacy callers, but a syntactically valid route is rejected because this signature cannot
    /// prove the required per-request protocol metadata or client-capabilities presence. New
    /// adapters must use [`Self::new_with_request_metadata`].
    pub fn new(
        protocol_version: &str,
        routing_method: &str,
        routing_tool_name: &str,
        body_method: &str,
        body_tool_name: &str,
    ) -> Result<Self, McpToolBoundaryError> {
        let _ = routing::ValidatedMcpToolCall::new(
            protocol_version,
            routing_method,
            routing_tool_name,
            body_method,
            body_tool_name,
        )
        .map_err(McpToolBoundaryError::from)?;

        Err(McpToolBoundaryError::MissingProtocolVersionMetadata)
    }

    /// Validate one MCP 2026-07-28 `tools/call` request envelope.
    ///
    /// The transport protocol-version header and structured request `_meta` protocol version are
    /// both mandatory, are bounded before comparison, must agree exactly, and must equal
    /// [`MCP_PROTOCOL_VERSION`]. A trusted structured parser must also attest that the request's
    /// client-capabilities object was present. Capability contents and optional `clientInfo` grant
    /// no OriginWeave authority and are not retained. After metadata validation, the existing
    /// bounded method/tool validator performs the explicit tool-to-action mapping.
    pub fn new_with_request_metadata(
        protocol_version_header: Option<&str>,
        protocol_version_metadata: Option<&str>,
        client_capabilities_present: bool,
        routing_method: &str,
        routing_tool_name: &str,
        body_method: &str,
        body_tool_name: &str,
    ) -> Result<Self, McpToolBoundaryError> {
        let protocol_version_header =
            protocol_version_header.ok_or(McpToolBoundaryError::MissingProtocolVersionHeader)?;
        let protocol_version_metadata = protocol_version_metadata
            .ok_or(McpToolBoundaryError::MissingProtocolVersionMetadata)?;

        if protocol_version_header.len() > MCP_PROTOCOL_VERSION.len()
            || protocol_version_metadata.len() > MCP_PROTOCOL_VERSION.len()
        {
            return Err(McpToolBoundaryError::UnsupportedProtocolVersion);
        }
        if protocol_version_header != protocol_version_metadata {
            return Err(McpToolBoundaryError::ProtocolVersionHeaderBodyMismatch);
        }
        if protocol_version_metadata != MCP_PROTOCOL_VERSION {
            return Err(McpToolBoundaryError::UnsupportedProtocolVersion);
        }
        if !client_capabilities_present {
            return Err(McpToolBoundaryError::MissingClientCapabilities);
        }

        let routed = routing::ValidatedMcpToolCall::new(
            protocol_version_metadata,
            routing_method,
            routing_tool_name,
            body_method,
            body_tool_name,
        )
        .map_err(McpToolBoundaryError::from)?;

        Ok(Self { routed })
    }

    /// Validate one MCP 2026-07-28 stdio `tools/call` request envelope.
    ///
    /// Stdio has no HTTP routing headers, so callers provide only request-body protocol metadata,
    /// capability presence, method, and tool name. The body values are correlated with themselves
    /// inside the existing pure envelope validator only to reuse its bounds and catalog checks; no
    /// HTTP header value is accepted, retained, or surfaced as evidence by this constructor.
    pub fn new_for_stdio(
        protocol_version_metadata: Option<&str>,
        client_capabilities_present: bool,
        body_method: &str,
        body_tool_name: &str,
    ) -> Result<Self, McpToolBoundaryError> {
        let protocol_version_metadata = protocol_version_metadata
            .ok_or(McpToolBoundaryError::MissingProtocolVersionMetadata)?;

        Self::new_with_request_metadata(
            Some(protocol_version_metadata),
            Some(protocol_version_metadata),
            client_capabilities_present,
            body_method,
            body_tool_name,
            body_method,
            body_tool_name,
        )
    }

    /// Return the canonical static tool name selected by the explicit mapping.
    #[must_use]
    pub const fn tool_name(&self) -> &'static str {
        self.routed.tool_name()
    }

    /// Return the existing OriginWeave typed action selected by this tool.
    #[must_use]
    pub const fn action_kind(&self) -> ActionKind {
        self.routed.action_kind()
    }
}

impl routing::ValidatedMcpToolsListRequest {
    /// Validate one MCP 2026-07-28 stdio `tools/list` request envelope.
    ///
    /// Stdio carries the protocol metadata and method in the JSON-RPC request body and has no HTTP
    /// routing headers. The body method/version are correlated with themselves inside the existing
    /// pure list validator only to reuse its bounded syntax, cache, and cursor checks; callers cannot
    /// supply or obtain fabricated HTTP header evidence through this constructor.
    pub fn new_for_stdio(
        protocol_version_metadata: Option<&str>,
        client_capabilities_present: bool,
        body_method: &str,
        cursor: Option<&str>,
    ) -> Result<Self, routing::McpToolsListBoundaryError> {
        let protocol_version_metadata = protocol_version_metadata
            .ok_or(routing::McpToolsListBoundaryError::MissingProtocolVersionMetadata)?;

        Self::new(
            Some(protocol_version_metadata),
            Some(protocol_version_metadata),
            client_capabilities_present,
            body_method,
            body_method,
            cursor,
        )
    }
}
