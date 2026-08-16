use crate::{
    BrowserAuthorityRegistry, BrowserContextOriginEpochDispatchTarget,
    BrowserContextProtocolDispatchError, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolRuntimeMetadata, DocumentEpoch,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

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
