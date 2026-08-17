//! Fail-closed MCP routing integrity for the external adapter boundary.
//!
//! This module validates only the stateless MCP protocol/method/tool routing
//! envelope and derives an existing [`ActionKind`]. It is deliberately not an
//! authorization decision: callers must independently enforce OriginWeave
//! capability, risk, approval, origin, secret-broker, and evidence policies.
//! No MCP arguments, outputs, credentials, or arbitrary model-visible values
//! are retained by this boundary.

use std::fmt;

use crate::{ActionKind, Capability, RiskClass};

/// MCP protocol generation accepted by this stateless adapter boundary.
pub const MCP_PROTOCOL_VERSION: &str = "2026-07-28";

/// The only MCP method that can enter the typed action-routing boundary.
pub const MCP_TOOLS_CALL_METHOD: &str = "tools/call";

/// The MCP discovery method accepted by the typed tools-list boundary.
pub const MCP_TOOLS_LIST_METHOD: &str = "tools/list";

/// Maximum accepted MCP tool-name length in bytes.
pub const MAX_MCP_TOOL_NAME_BYTES: usize = 128;

/// One deterministic MCP tool descriptor derived from OriginWeave's reviewed action registry.
///
/// The descriptor is discovery metadata only. It does not grant capabilities, origin access,
/// approval, secret access, or any other authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpToolCatalogEntry {
    tool_name: &'static str,
    action_kind: ActionKind,
}

impl McpToolCatalogEntry {
    /// Return the canonical MCP tool name exposed by this registry entry.
    #[must_use]
    pub const fn tool_name(&self) -> &'static str {
        self.tool_name
    }

    /// Return the typed OriginWeave action represented by this registry entry.
    #[must_use]
    pub const fn action_kind(&self) -> ActionKind {
        self.action_kind
    }

    /// Return the capability required by the represented action.
    #[must_use]
    pub const fn required_capability(&self) -> Capability {
        self.action_kind.required_capability()
    }

    /// Return the risk class assigned to the represented action.
    #[must_use]
    pub const fn risk_class(&self) -> RiskClass {
        self.action_kind.risk_class()
    }
}

/// The complete explicit MCP tool-to-action registry accepted by this boundary.
///
/// Order is deterministic so adapters can derive stable discovery output from this single
/// reviewed registry rather than maintaining a second mapping that could drift from routing.
const MCP_TOOL_CATALOG: &[McpToolCatalogEntry] = &[
    McpToolCatalogEntry {
        tool_name: "originweave.observe",
        action_kind: ActionKind::Observe,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.extract",
        action_kind: ActionKind::Extract,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.navigate",
        action_kind: ActionKind::Navigate,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.download",
        action_kind: ActionKind::Download,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.draft",
        action_kind: ActionKind::Draft,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.submit",
        action_kind: ActionKind::Submit,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.upload",
        action_kind: ActionKind::Upload,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.fill_secret",
        action_kind: ActionKind::FillSecret,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.purchase",
        action_kind: ActionKind::Purchase,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.delete",
        action_kind: ActionKind::Delete,
    },
    McpToolCatalogEntry {
        tool_name: "originweave.manage_permission",
        action_kind: ActionKind::ManagePermission,
    },
];

/// Return the deterministic reviewed MCP tool catalog.
///
/// Adapters may use this slice to derive discovery responses. Serialization, pagination, cache
/// policy, transport I/O, and authorization remain outside this stateless registry boundary.
#[must_use]
pub const fn supported_mcp_tools() -> &'static [McpToolCatalogEntry] {
    MCP_TOOL_CATALOG
}

/// Protocol disposition carried by a typed MCP result.
///
/// OriginWeave currently constructs only terminal results at this boundary. A transport adapter
/// must serialize [`Self::Complete`] as MCP's `"complete"` result type and must not omit or
/// reinterpret the required protocol field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpResultType {
    /// The request completed and this value contains the final result.
    Complete,
}

/// Cache-sharing scope for an MCP cacheable list result.
///
/// OriginWeave currently exposes only the conservative private scope. A transport adapter must
/// serialize this as MCP's `"private"` cache scope and must not widen it without a separately
/// reviewed policy that proves the returned catalog is safe to share across callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCacheScope {
    /// The result may be cached only for the current caller's private context.
    Private,
}

/// One typed MCP `tools/list` page derived from the reviewed tool catalog.
///
/// This value is discovery metadata only. It does not grant any tool capability or action
/// authority. The initial contract is deliberately one complete private page with zero freshness
/// so adapters cannot omit MCP's required result disposition or accidentally share or reuse
/// discovery metadata beyond the current request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpToolsListPage {
    result_type: McpResultType,
    tools: &'static [McpToolCatalogEntry],
    ttl_ms: u64,
    cache_scope: McpCacheScope,
    next_cursor: Option<&'static str>,
}

impl McpToolsListPage {
    /// Return the mandatory MCP result disposition for this list page.
    #[must_use]
    pub const fn result_type(&self) -> McpResultType {
        self.result_type
    }

    /// Return the deterministic reviewed tool entries in this page.
    #[must_use]
    pub const fn tools(&self) -> &'static [McpToolCatalogEntry] {
        self.tools
    }

    /// Return the MCP freshness lifetime in milliseconds.
    ///
    /// The current conservative contract is zero, so clients must treat the result as
    /// immediately stale rather than reusing it for a later request.
    #[must_use]
    pub const fn ttl_ms(&self) -> u64 {
        self.ttl_ms
    }

    /// Return the MCP cache-sharing scope for this page.
    #[must_use]
    pub const fn cache_scope(&self) -> McpCacheScope {
        self.cache_scope
    }

    /// Return the opaque continuation cursor when another page exists.
    ///
    /// The current fixed catalog is emitted as one complete page, so this is always `None`.
    #[must_use]
    pub const fn next_cursor(&self) -> Option<&'static str> {
        self.next_cursor
    }
}

/// Build the conservative typed MCP `tools/list` result for the reviewed catalog.
///
/// This function does not perform transport serialization, authorization, or pagination. It
/// binds the catalog to the mandatory complete result disposition plus explicit zero-TTL/private
/// cache hints so adapters cannot invent broader protocol or cache semantics independently from
/// this reviewed boundary.
#[must_use]
pub const fn mcp_tools_list_page() -> McpToolsListPage {
    McpToolsListPage {
        result_type: McpResultType::Complete,
        tools: MCP_TOOL_CATALOG,
        ttl_ms: 0,
        cache_scope: McpCacheScope::Private,
        next_cursor: None,
    }
}

/// A deterministic failure while validating one MCP `tools/list` request envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpToolsListBoundaryError {
    /// The transport request omitted the required MCP protocol-version header.
    MissingProtocolVersionHeader,
    /// The structured request metadata omitted the required MCP protocol version.
    MissingProtocolVersionMetadata,
    /// The transport protocol version disagrees with the structured request metadata.
    ProtocolVersionHeaderBodyMismatch,
    /// The request names an MCP protocol generation this boundary does not support.
    UnsupportedProtocolVersion,
    /// The structured request metadata omitted the required client-capabilities object.
    MissingClientCapabilities,
    /// MCP routing method metadata disagrees with the method in the request body.
    MethodHeaderBodyMismatch,
    /// The request method is not the supported `tools/list` operation.
    UnsupportedMethod,
    /// The request supplied a cursor that this fixed single-page catalog never issued.
    UnsupportedCursor,
}

impl fmt::Display for McpToolsListBoundaryError {
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
            Self::MethodHeaderBodyMismatch => {
                formatter.write_str("MCP method header does not match the request body")
            }
            Self::UnsupportedMethod => {
                formatter.write_str("only MCP tools/list requests can enter the discovery boundary")
            }
            Self::UnsupportedCursor => {
                formatter.write_str("MCP tools/list cursor was not issued by this fixed catalog")
            }
        }
    }
}

impl std::error::Error for McpToolsListBoundaryError {}

/// An MCP `tools/list` request whose protocol, required metadata, and routing envelope were
/// validated.
///
/// This boundary is deliberately narrower than a general transport or pagination implementation.
/// A trusted structured parser must prove whether the required per-request client-capabilities
/// object was present; this type never accepts its contents as authority. The current reviewed
/// catalog returns one complete page and emits no continuation cursor, so no non-null cursor can
/// be a value previously issued by OriginWeave. A transport adapter must not silently ignore or
/// reinterpret a supplied cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedMcpToolsListRequest {
    method: &'static str,
}

impl ValidatedMcpToolsListRequest {
    /// Validate the stateless request envelope for the current fixed `tools/list` catalog.
    ///
    /// Both the required transport protocol-version header and structured request `_meta`
    /// protocol version must be present, equal, and exactly [`MCP_PROTOCOL_VERSION`]. A trusted
    /// structured parser must also attest that the required `_meta` client-capabilities object was
    /// present; its contents grant no OriginWeave authority. The routing/body method must agree.
    /// Any supplied cursor fails closed because [`mcp_tools_list_page`] emits no continuation
    /// cursor; accepting one would silently invent pagination state that OriginWeave never issued.
    pub fn new(
        protocol_version_header: Option<&str>,
        protocol_version_metadata: Option<&str>,
        client_capabilities_present: bool,
        routing_method: &str,
        body_method: &str,
        cursor: Option<&str>,
    ) -> Result<Self, McpToolsListBoundaryError> {
        let protocol_version_header = protocol_version_header
            .ok_or(McpToolsListBoundaryError::MissingProtocolVersionHeader)?;
        let protocol_version_metadata = protocol_version_metadata
            .ok_or(McpToolsListBoundaryError::MissingProtocolVersionMetadata)?;

        if protocol_version_header != protocol_version_metadata {
            return Err(McpToolsListBoundaryError::ProtocolVersionHeaderBodyMismatch);
        }
        if protocol_version_metadata != MCP_PROTOCOL_VERSION {
            return Err(McpToolsListBoundaryError::UnsupportedProtocolVersion);
        }
        if !client_capabilities_present {
            return Err(McpToolsListBoundaryError::MissingClientCapabilities);
        }
        if routing_method != body_method {
            return Err(McpToolsListBoundaryError::MethodHeaderBodyMismatch);
        }
        if routing_method != MCP_TOOLS_LIST_METHOD {
            return Err(McpToolsListBoundaryError::UnsupportedMethod);
        }
        if cursor.is_some() {
            return Err(McpToolsListBoundaryError::UnsupportedCursor);
        }

        Ok(Self {
            method: MCP_TOOLS_LIST_METHOD,
        })
    }

    /// Return the canonical MCP method validated by this request.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        self.method
    }
}

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
    /// explicitly supported mapping. Each untrusted tool name is shape-validated
    /// before cross-field comparison so malformed or oversized names cannot
    /// bypass the bounded routing syntax through mismatch handling.
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
        if !valid_tool_name(routing_tool_name) || !valid_tool_name(body_tool_name) {
            return Err(McpToolBoundaryError::InvalidToolName);
        }
        if routing_method != body_method || routing_tool_name != body_tool_name {
            return Err(McpToolBoundaryError::HeaderBodyMismatch);
        }
        if routing_method != MCP_TOOLS_CALL_METHOD {
            return Err(McpToolBoundaryError::UnsupportedMethod);
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
    MCP_TOOL_CATALOG
        .iter()
        .find(|entry| entry.tool_name == tool_name)
        .map(|entry| (entry.tool_name, entry.action_kind))
        .ok_or(McpToolBoundaryError::UnknownTool)
}
