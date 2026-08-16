use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    BrowserAuthorityRegistry, BrowserContextOriginEpochDispatchTarget,
    BrowserContextProtocolDispatchError, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolRuntimeMetadata, BrowserRegistryError, DocumentEpoch,
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, ObservedNodeHandle, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse,
};

/// Exact WebDriver BiDi method used by the bounded accessibility-query contract.
pub const WEBDRIVER_BIDI_LOCATE_NODES_METHOD: &str = "browsingContext.locateNodes";
/// Maximum UTF-8 bytes accepted for one accessibility-role query value.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES: usize = 64;
/// Maximum UTF-8 bytes accepted for one accessibility-name query value.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES: usize = 512;
/// Maximum number of nodes one bounded accessibility query may request.
pub const MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT: u16 = 128;
/// Fixed DOM serialization depth for the first bounded BiDi node-query slice.
pub const WEBDRIVER_BIDI_QUERY_MAX_DOM_DEPTH: u16 = 0;
/// Fixed object serialization depth for the first bounded BiDi node-query slice.
pub const WEBDRIVER_BIDI_QUERY_MAX_OBJECT_DEPTH: u16 = 0;
/// Fixed shadow-tree serialization mode for the first bounded BiDi node-query slice.
pub const WEBDRIVER_BIDI_QUERY_INCLUDE_SHADOW_TREE: &str = "none";

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
    /// The accessibility role contained whitespace, a control, or a Unicode format character.
    InvalidRole,
    /// The accessible name contained a control, Unicode format character, or only whitespace.
    InvalidName,
    /// The accessible name exceeded the local UTF-8 byte budget.
    NameTooLong,
    /// The requested node count was zero or exceeded the local result budget.
    InvalidNodeCount,
    /// The untrusted adapter returned more nodes than the reviewed request budget allowed.
    ResultNodeCountExceeded,
}

impl Display for WebDriverBiDiAccessibilityQueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingLocatorValue => "accessibility query requires a role or accessible name",
            Self::EmptyRole => "accessibility query role must not be empty",
            Self::RoleTooLong => "accessibility query role exceeds the local byte budget",
            Self::EmptyName => "accessibility query name must not be empty",
            Self::InvalidRole => {
                "accessibility query role must not contain whitespace, control, or Unicode format characters"
            }
            Self::InvalidName => {
                "accessibility query name must not contain control or Unicode format characters or only whitespace"
            }
            Self::NameTooLong => "accessibility query name exceeds the local byte budget",
            Self::InvalidNodeCount => "accessibility query node count is outside the local budget",
            Self::ResultNodeCountExceeded => {
                "accessibility query result exceeds the requested node budget"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for WebDriverBiDiAccessibilityQueryError {}

/// Bounded transport parameters for WebDriver BiDi accessibility-node lookup.
///
/// This value captures only the reviewed `browsingContext.locateNodes` accessibility-locator
/// parameters needed by the first Chromium observation slice. It accepts an exact role, an exact
/// accessible name, or both, together with a finite result count. Roles are exact tokens and
/// therefore reject whitespace, controls, and Unicode format characters. Accessible names may
/// contain ordinary spaces but reject controls, format characters, and whitespace-only values.
/// Text budgets are OriginWeave resource limits rather
/// than claims about upstream protocol maxima.
///
/// The first slice also fixes WebDriver BiDi serialization to zero DOM depth, zero object depth,
/// and no shadow-tree expansion. Those settings intentionally minimize the remote-value surface a
/// future transport adapter may request. The adapter must additionally revalidate the returned
/// node count against this exact query before it retains or normalizes any returned node data.
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
    /// through multi-byte text. Roles are exact WAI-ARIA tokens, so whitespace, control, and
    /// Unicode format characters fail closed instead of becoming fallback-role lists. Accessible
    /// names may contain ordinary spaces but not controls, Unicode format characters, or
    /// whitespace-only values. At least one selector value and one result slot are required.
    pub fn new(
        role: Option<&str>,
        name: Option<&str>,
        max_node_count: u16,
    ) -> Result<Self, WebDriverBiDiAccessibilityQueryError> {
        if role.is_some_and(str::is_empty) {
            return Err(WebDriverBiDiAccessibilityQueryError::EmptyRole);
        }
        if role.is_some_and(|value| crate::contains_disallowed_protocol_text(value, false)) {
            return Err(WebDriverBiDiAccessibilityQueryError::InvalidRole);
        }
        if role.is_some_and(|value| value.len() > MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES) {
            return Err(WebDriverBiDiAccessibilityQueryError::RoleTooLong);
        }
        if name.is_some_and(str::is_empty) {
            return Err(WebDriverBiDiAccessibilityQueryError::EmptyName);
        }
        if name.is_some_and(|value| {
            crate::contains_disallowed_protocol_text(value, true)
                || value.chars().all(char::is_whitespace)
        }) {
            return Err(WebDriverBiDiAccessibilityQueryError::InvalidName);
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

    /// Return the fixed maximum DOM serialization depth for returned remote nodes.
    #[must_use]
    pub const fn serialization_max_dom_depth(&self) -> u16 {
        WEBDRIVER_BIDI_QUERY_MAX_DOM_DEPTH
    }

    /// Return the fixed maximum object serialization depth for returned remote nodes.
    #[must_use]
    pub const fn serialization_max_object_depth(&self) -> u16 {
        WEBDRIVER_BIDI_QUERY_MAX_OBJECT_DEPTH
    }

    /// Return the fixed shadow-tree serialization mode for returned remote nodes.
    #[must_use]
    pub const fn serialization_include_shadow_tree(&self) -> &'static str {
        WEBDRIVER_BIDI_QUERY_INCLUDE_SHADOW_TREE
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

    /// Revalidate an untrusted `locateNodes` result count against this exact request budget.
    ///
    /// A conforming browser is expected to honor `maxNodeCount`, but an adapter boundary must not
    /// treat that expectation as resource authority. Zero through the requested maximum are valid;
    /// any larger returned array fails closed before later node normalization or retention.
    pub fn validate_result_count(
        &self,
        returned_node_count: usize,
    ) -> Result<(), WebDriverBiDiAccessibilityQueryError> {
        if returned_node_count > usize::from(self.max_node_count) {
            return Err(WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded);
        }
        Ok(())
    }

    /// Admit one untrusted `locateNodes` result against the exact current document authority.
    ///
    /// The caller must transfer a non-cloneable [`ValidatedBrowserProtocolUse`] whose capability is
    /// exactly [`BrowserProtocolCapability::SemanticObservation`]. Navigation and TypedInput proofs
    /// fail closed before the registry is consulted, so those adapters cannot mint observation
    /// handles. The proof is consumed by ownership and cannot be reused for a later bind.
    ///
    /// The registry then proves that `target` still names the current session, browsing context,
    /// canonical origin, and document epoch. Only then is the returned item count checked against
    /// this query's budget. Each item must be an exact `node` remote value with a usable shared
    /// identifier; those identifiers are translated through the registry into
    /// [`ObservedNodeHandle`] values bound to that same current authority.
    ///
    /// This method performs no browser I/O, does not authenticate Chromium, and does not grant
    /// policy, destination, or typed-input authority. A later action must still revalidate the
    /// returned handles immediately before use.
    pub fn bind_current_nodes(
        &self,
        validated: ValidatedBrowserProtocolUse,
        authority_registry: &mut BrowserAuthorityRegistry,
        target: BrowserContextOriginEpochDispatchTarget<'_>,
        items: &[(&str, Option<&str>)],
    ) -> Result<Vec<ObservedNodeHandle>, WebDriverBiDiLocateNodesAdmissionError> {
        if validated.capability() != BrowserProtocolCapability::SemanticObservation {
            return Err(
                WebDriverBiDiLocateNodesAdmissionError::UnsupportedCapability(
                    validated.capability(),
                ),
            );
        }
        let _consumed_query_nodes_proof = validated;
        let context_origin = target.context_origin();
        let context = context_origin.context();
        let current_epoch = authority_registry
            .require_context_origin(
                context.browser_session(),
                context.browsing_context(),
                context_origin.expected_origin(),
            )
            .map_err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority)?;
        if current_epoch != target.expected_epoch() {
            return Err(
                WebDriverBiDiLocateNodesAdmissionError::DocumentEpochMismatch {
                    expected: target.expected_epoch(),
                    current: current_epoch,
                },
            );
        }
        self.validate_result_count(items.len())
            .map_err(WebDriverBiDiLocateNodesAdmissionError::Query)?;

        let mut references = Vec::new();
        for (remote_type, shared_id) in items {
            references.push(
                WebDriverBiDiRemoteNodeReference::new(remote_type, *shared_id)
                    .map_err(WebDriverBiDiLocateNodesAdmissionError::RemoteNode)?,
            );
        }

        let mut handles = Vec::new();
        for reference in references {
            handles.push(
                authority_registry
                    .bind_node(
                        context.browser_session(),
                        context.browsing_context(),
                        context_origin.expected_origin(),
                        reference.shared_id(),
                    )
                    .map_err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority)?,
            );
        }
        Ok(handles)
    }
}

/// Fail-closed errors for QueryNodes admission that requires SemanticObservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiQueryNodesAdmissionError {
    /// Protocol metadata or the QueryNodes capability failed before node admission.
    ProtocolDispatch(BrowserContextProtocolDispatchError),
    /// The untrusted `locateNodes` result failed current-authority admission.
    LocateNodes(WebDriverBiDiLocateNodesAdmissionError),
}

impl Display for WebDriverBiDiQueryNodesAdmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolDispatch(error) => {
                write!(
                    formatter,
                    "QueryNodes protocol dispatch rejected locateNodes admission: {error}"
                )
            }
            Self::LocateNodes(error) => {
                write!(
                    formatter,
                    "QueryNodes current-authority admission rejected locateNodes result: {error}"
                )
            }
        }
    }
}

impl Error for WebDriverBiDiQueryNodesAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProtocolDispatch(error) => Some(error),
            Self::LocateNodes(error) => Some(error),
        }
    }
}

/// Fail-closed errors for admitting an untrusted `locateNodes` result into current node authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesAdmissionError {
    /// The untrusted result violated this query's reviewed locator or result-count contract.
    Query(WebDriverBiDiAccessibilityQueryError),
    /// One result item was not an admissible node remote value.
    RemoteNode(WebDriverBiDiRemoteNodeReferenceError),
    /// The observed document epoch no longer matches the registry's current document.
    DocumentEpochMismatch {
        /// The document epoch that produced the observation being bound.
        expected: DocumentEpoch,
        /// The document epoch currently active in the registry.
        current: DocumentEpoch,
    },
    /// The supplied browser session, context, or origin is not current in the registry.
    BrowserAuthority(BrowserRegistryError),
    /// The consumed protocol-use proof was not SemanticObservation.
    UnsupportedCapability(BrowserProtocolCapability),
}

impl Display for WebDriverBiDiLocateNodesAdmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(error) => {
                write!(
                    formatter,
                    "accessibility query rejected locateNodes admission: {error}"
                )
            }
            Self::RemoteNode(error) => {
                write!(
                    formatter,
                    "remote node reference rejected locateNodes admission: {error}"
                )
            }
            Self::DocumentEpochMismatch { expected, current } => write!(
                formatter,
                "browser document epoch {} no longer matches observed epoch {}",
                current.value(),
                expected.value()
            ),
            Self::BrowserAuthority(error) => {
                write!(
                    formatter,
                    "browser authority denied locateNodes admission: {error}"
                )
            }
            Self::UnsupportedCapability(capability) => {
                let name = match capability {
                    BrowserProtocolCapability::Navigation => "Navigation",
                    BrowserProtocolCapability::SemanticObservation => "SemanticObservation",
                    BrowserProtocolCapability::TypedInput => "TypedInput",
                    BrowserProtocolCapability::NetworkObservation => "NetworkObservation",
                };
                write!(
                    formatter,
                    "locateNodes admission requires a SemanticObservation protocol-use proof, not {name}"
                )
            }
        }
    }
}

impl Error for WebDriverBiDiLocateNodesAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::RemoteNode(error) => Some(error),
            Self::DocumentEpochMismatch { .. } => None,
            Self::BrowserAuthority(error) => Some(error),
            Self::UnsupportedCapability(_) => None,
        }
    }
}

/// Exact WebDriver BiDi remote-value type admitted as a later node handle.
pub const WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE: &str = "node";

/// Fail-closed validation errors for one untrusted WebDriver BiDi node remote value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiRemoteNodeReferenceError {
    /// The remote value type was not the exact `node` type.
    UnexpectedRemoteType,
    /// The remote value omitted `sharedId`.
    MissingSharedId,
    /// The shared identifier was empty, contained control, whitespace, or Unicode format text, or exceeded the local budget.
    InvalidSharedId,
}

impl Display for WebDriverBiDiRemoteNodeReferenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnexpectedRemoteType => {
                "remote node reference type must be the exact node remote value"
            }
            Self::MissingSharedId => "remote node reference requires a shared id",
            Self::InvalidSharedId => {
                "remote node reference shared id is empty, contains control, whitespace, or Unicode format characters, or exceeds the local byte budget"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for WebDriverBiDiRemoteNodeReferenceError {}

/// Bounded admission of one untrusted WebDriver BiDi `script.NodeRemoteValue`.
///
/// The 1 June 2026 WebDriver BiDi Working Draft returns `script.NodeRemoteValue` items from
/// `browsingContext.locateNodes`. Those values have a required `type` of `node` and an optional
/// `sharedId`. OriginWeave admits a result item only when the type is exactly `node` and a
/// non-empty `sharedId` fits the same UTF-8 identifier budget used by browser session and context
/// identifiers and contains no control, whitespace, or Unicode format characters.
///
/// Requiring `sharedId` is a local fail-closed policy: the Working Draft permits omitting it, but a
/// later typed-input adapter cannot refer to the same node across realms without that shared
/// identity. Construction grants no session, context, origin, document-epoch, semantic-node,
/// policy, or network authority and performs no browser I/O. The admitted value remains an
/// untrusted transport handle until a separately reviewed authority boundary binds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBiDiRemoteNodeReference {
    shared_id: String,
}

impl WebDriverBiDiRemoteNodeReference {
    /// Admit one untrusted locateNodes remote value as a later node handle.
    ///
    /// The remote type is checked first so a non-node value cannot be retained even when it carries
    /// a well-formed shared identifier. A missing shared identifier is distinct from an empty or
    /// over-budget identifier so callers can distinguish protocol omission from local
    /// resource-budget rejection.
    pub fn new(
        remote_type: &str,
        shared_id: Option<&str>,
    ) -> Result<Self, WebDriverBiDiRemoteNodeReferenceError> {
        if remote_type != WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE {
            return Err(WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType);
        }
        let Some(shared_id) = shared_id else {
            return Err(WebDriverBiDiRemoteNodeReferenceError::MissingSharedId);
        };
        if shared_id.is_empty()
            || shared_id.len() > MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES
            || crate::contains_disallowed_protocol_text(shared_id, false)
        {
            return Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId);
        }
        Ok(Self {
            shared_id: shared_id.to_owned(),
        })
    }

    /// Return the exact admitted WebDriver BiDi remote-value type.
    #[must_use]
    pub const fn remote_type(&self) -> &'static str {
        WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE
    }

    /// Return the exact shared node identifier admitted from the remote value.
    #[must_use]
    pub fn shared_id(&self) -> &str {
        &self.shared_id
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

    /// Admit one untrusted `locateNodes` result only after QueryNodes protocol proof.
    ///
    /// The same-call boundary first obtains a non-cloneable protocol-use proof for
    /// [`BrowserProtocolOperation::QueryNodes`], which derives
    /// [`BrowserProtocolCapability::SemanticObservation`]. That proof is transferred by
    /// ownership into [`WebDriverBiDiAccessibilityQuery::bind_current_nodes`], which refuses
    /// any other capability before translating admitted `sharedId` values into
    /// [`ObservedNodeHandle`] values on the exact current session, browsing context, canonical
    /// origin, and document epoch.
    ///
    /// This method performs no browser I/O, does not authenticate Chromium, and does not grant
    /// policy, destination, or typed-input authority. A later action must still revalidate the
    /// returned handles immediately before use.
    pub fn admit_query_nodes(
        &self,
        authority_registry: &mut BrowserAuthorityRegistry,
        target: BrowserContextOriginEpochDispatchTarget<'_>,
        required_originweave_protocol_version: OriginWeaveProtocolVersion,
        runtime_metadata: BrowserProtocolRuntimeMetadata<'_>,
        query: &WebDriverBiDiAccessibilityQuery,
        items: &[(&str, Option<&str>)],
    ) -> Result<Vec<ObservedNodeHandle>, WebDriverBiDiQueryNodesAdmissionError> {
        let validated = self
            .dispatch_operation_if_context_origin_epoch_current(
                authority_registry,
                target,
                required_originweave_protocol_version,
                runtime_metadata,
                BrowserProtocolOperation::QueryNodes,
                |validated, _operation, _epoch| validated,
            )
            .map_err(WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch)?;
        query
            .bind_current_nodes(validated, authority_registry, target, items)
            .map_err(WebDriverBiDiQueryNodesAdmissionError::LocateNodes)
    }
}
