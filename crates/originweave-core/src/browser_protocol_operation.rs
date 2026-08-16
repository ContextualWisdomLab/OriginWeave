use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    BrowserAuthorityRegistry, BrowserContextOriginEpochDispatchTarget,
    BrowserContextProtocolDispatchError, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolRuntimeMetadata, DocumentEpoch,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

/// Exact WebDriver BiDi method used by the bounded accessibility-query contract.
pub const WEBDRIVER_BIDI_LOCATE_NODES_METHOD: &str = "browsingContext.locateNodes";
/// Maximum UTF-8 bytes accepted for one accessibility-role query value.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES: usize = 64;
/// Maximum UTF-8 bytes accepted for one accessibility-name query value.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES: usize = 512;
/// Maximum number of nodes one bounded accessibility query may request.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT: u16 = 128;

/// Fail-closed validation errors for one bounded WebDriver BiDi accessibility query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiAccessibilityQueryError {
    /// Neither an accessibility role nor an accessible name was supplied.
    MissingLocatorValue,
    /// An explicitly supplied accessibility role was empty.
    EmptyRole,
    /// The accessibility role exceeded the local UTF-8 byte budget.
    RoleTooLong,
    /// An explicitly supplied accessible name was empty.
    EmptyName,
    /// The accessible name exceeded the local UTF-8 byte budget.
    NameTooLong,
    /// The requested node count was zero or exceeded the local result budget.
    InvalidNodeCount,
}

impl Display for WebDriverBiDiAccessibilityQueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingLocatorValue => "accessibility query requires a role or accessible name",
            Self::EmptyRole => "accessibility query role must not be empty",
            Self::RoleTooLong => "accessibility query role exceeds the local byte budget",
            Self::EmptyName => "accessibility query name must not be empty",
            Self::NameTooLong => "accessibility query name exceeds the local byte budget",
            Self::InvalidNodeCount => "accessibility query node count is outside the local budget",
        };
        formatter.write_str(message)
    }
}

impl Error for WebDriverBiDiAccessibilityQueryError {}

/// Bounded transport parameters for WebDriver BiDi accessibility-node lookup.
///
/// This value captures only the reviewed `browsingContext.locateNodes` accessibility-locator
/// parameters needed by the first Chromium observation slice. It accepts an exact role, an exact
/// accessible name, or both, together with a finite result count. Text budgets are OriginWeave
/// resource limits rather than claims about upstream protocol maxima.
///
/// Construction grants no browser session, context, origin, semantic-node, policy, capability, or
/// network authority and performs no browser I/O. A trusted adapter must still bind the query to an
/// exact current browsing context through the separately reviewed authority and protocol boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBiDiAccessibilityQuery {
    role: Option<String>,
    name: Option<String>,
    max_node_count: u16,
}

impl WebDriverBiDiAccessibilityQuery {
    /// Validate one bounded accessibility lookup request.
    ///
    /// Explicit empty values fail closed rather than being treated as absent. Role and name limits
    /// are measured in UTF-8 bytes so later serialization cannot exceed the reviewed local budget
    /// through multi-byte text. At least one selector value and one result slot are required.
    pub fn new(
        role: Option<&str>,
        name: Option<&str>,
        max_node_count: u16,
    ) -> Result<Self, WebDriverBiDiAccessibilityQueryError> {
        if role.is_some_and(str::is_empty) {
            return Err(WebDriverBiDiAccessibilityQueryError::EmptyRole);
        }
        if role.is_some_and(|value| value.len() > MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES) {
            return Err(WebDriverBiDiAccessibilityQueryError::RoleTooLong);
        }
        if name.is_some_and(str::is_empty) {
            return Err(WebDriverBiDiAccessibilityQueryError::EmptyName);
        }
        if name.is_some_and(|value| value.len() > MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES) {
            return Err(WebDriverBiDiAccessibilityQueryError::NameTooLong);
        }
        if role.is_none() && name.is_none() {
            return Err(WebDriverBiDiAccessibilityQueryError::MissingLocatorValue);
        }
        if max_node_count == 0 || max_node_count > MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT {
            return Err(WebDriverBiDiAccessibilityQueryError::InvalidNodeCount);
        }

        Ok(Self {
            role: role.map(str::to_owned),
            name: name.map(str::to_owned),
            max_node_count,
        })
    }

    /// Return the exact upstream method associated with this query contract.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        WEBDRIVER_BIDI_LOCATE_NODES_METHOD
    }

    /// Return the exact WebDriver BiDi locator type represented by this value.
    #[must_use]
    pub const fn locator_type(&self) -> &'static str {
        "accessibility"
    }

    /// Return the exact optional accessibility role requested by the caller.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    /// Return the exact optional accessible name requested by the caller.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Return the finite maximum number of nodes requested from the adapter.
    #[must_use]
    pub const fn max_node_count(&self) -> u16 {
        self.max_node_count
    }
}

/// One typed buyer-visible browser operation whose transport prerequisite is derived internally.
///
/// This value carries operation semantics only. It grants no browser session, context, origin,
/// semantic-node, policy, approval, secret, network, or adapter authority and does not perform
/// browser I/O. The vocabulary intentionally mirrors the bounded first Chromium vertical slice so
/// callers cannot hide materially different actions behind a coarse transport capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserProtocolOperation {
    /// Navigate one controlled browser context.
    Navigate,
    /// Query bounded semantic nodes from the current document.
    QueryNodes,
    /// Dispatch one policy-authorized click to a separately validated semantic node.
    ClickNode,
    /// Dispatch policy-authorized text input to a separately validated semantic node.
    TypeText,
    /// Observe bounded semantic state until a separately specified condition is satisfied.
    WaitForState,
    /// Produce bounded browser-network evidence.
    ObserveNetwork,
}

impl BrowserProtocolOperation {
    /// Return the exact adapter capability required for this typed operation.
    ///
    /// This mapping is a transport prerequisite only. A matching capability does not authorize the
    /// operation itself: node freshness, policy, approval, destination, and post-condition checks
    /// remain independent authority boundaries.
    #[must_use]
    pub const fn required_capability(self) -> BrowserProtocolCapability {
        match self {
            Self::Navigate => BrowserProtocolCapability::Navigation,
            Self::QueryNodes | Self::WaitForState => BrowserProtocolCapability::SemanticObservation,
            Self::ClickNode | Self::TypeText => BrowserProtocolCapability::TypedInput,
            Self::ObserveNetwork => BrowserProtocolCapability::NetworkObservation,
        }
    }
}

impl BrowserProtocolAdapterDescriptor {
    /// Revalidate exact browser authority and derive adapter capability from one typed operation.
    ///
    /// The existing context/origin/document-epoch boundary runs first, followed by exact runtime
    /// protocol metadata and the capability derived from `operation`. The callback can run only
    /// after all prerequisites pass and receives the same typed operation together with the
    /// non-cloneable protocol-use proof and freshly revalidated document epoch.
    ///
    /// This method does not authenticate Chromium or the adapter, grant policy approval, validate
    /// semantic-node state, authorize destination/network activity, perform browser I/O, or prove
    /// an action post-condition. The operation value itself grants no authority.
    pub fn dispatch_operation_if_context_origin_epoch_current<R, F>(
        &self,
        authority_registry: &BrowserAuthorityRegistry,
        target: BrowserContextOriginEpochDispatchTarget<'_>,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        operation: BrowserProtocolOperation,
        dispatch: F,
    ) -> Result<R, BrowserContextProtocolDispatchError>
    where
        F: FnOnce(ValidatedBrowserProtocolUse, BrowserProtocolOperation, DocumentEpoch) -> R,
    {
        self.dispatch_if_context_origin_epoch_current(
            authority_registry,
            target,
            required_originweave_protocol_version,
            runtime_metadata,
            operation.required_capability(),
            |validated, epoch| dispatch(validated, operation, epoch),
        )
    }
}
