use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, BrowserRegistryError, NodeHandleError,
    WebDriverBiDiPointerClickCommand, WebDriverBiDiPointerClickCommandError,
    WebDriverBiDiRemoteNodeReference,
};

/// Fail-closed authority errors while binding one pointer click to an admitted current node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiPointerClickAuthorityError {
    /// The final deterministic pointer command failed its bounded serialization contract.
    Command(WebDriverBiDiPointerClickCommandError),
    /// Current browser session, context, or origin authority could not be revalidated.
    BrowserAuthority(BrowserRegistryError),
    /// The observed node belongs to a stale or otherwise mismatched browser document lifetime.
    NodeHandle(NodeHandleError),
    /// The supplied wire node identifier is not the identifier admitted for this exact node handle.
    NodeExternalIdentifierMismatch,
}

impl Display for WebDriverBiDiPointerClickAuthorityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(error) => {
                write!(formatter, "pointer click command rejected input: {error}")
            }
            Self::BrowserAuthority(error) => {
                write!(
                    formatter,
                    "pointer click browser authority rejected input: {error}"
                )
            }
            Self::NodeHandle(error) => {
                write!(
                    formatter,
                    "pointer click node authority rejected input: {error}"
                )
            }
            Self::NodeExternalIdentifierMismatch => formatter.write_str(
                "pointer click wire node identifier does not match the admitted current node",
            ),
        }
    }
}

impl Error for WebDriverBiDiPointerClickAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Command(error) => Some(error),
            Self::BrowserAuthority(error) => Some(error),
            Self::NodeHandle(error) => Some(error),
            Self::NodeExternalIdentifierMismatch => None,
        }
    }
}

impl WebDriverBiDiPointerClickCommand {
    /// Bind one pointer click to the exact current semantic node admitted by this authority registry.
    ///
    /// The external browsing-context identifier must still name the handle's registered context,
    /// the handle must still belong to the registry's current document epoch and canonical origin,
    /// and the remote `sharedId` must be the exact wire identifier retained during semantic-node
    /// admission. The handle also carries opaque registry-instance provenance, so copying the same
    /// public session/context/origin/epoch/node tuple cannot recreate action authority. These checks
    /// are immediate-use validation only: they do not authenticate the browser process, grant policy
    /// or Agent authority, authorize a destination, or perform I/O.
    pub fn new_for_current_node(
        command_id: u64,
        browsing_context: &str,
        handle: &AdmittedNodeHandle,
        node: &WebDriverBiDiRemoteNodeReference,
        registry: &BrowserAuthorityRegistry,
    ) -> Result<Self, WebDriverBiDiPointerClickAuthorityError> {
        registry
            .require_context_external_identifier(
                handle.browser_session(),
                handle.browsing_context(),
                browsing_context,
            )
            .map_err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority)?;

        let current_epoch = registry
            .current_context_epoch(handle.browser_session(), handle.browsing_context())
            .map_err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority)?;
        handle
            .validate_current(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
                current_epoch,
            )
            .map_err(WebDriverBiDiPointerClickAuthorityError::NodeHandle)?;

        registry
            .require_context_origin(
                handle.browser_session(),
                handle.browsing_context(),
                handle.origin(),
            )
            .map_err(WebDriverBiDiPointerClickAuthorityError::BrowserAuthority)?;

        if !registry.node_external_identifier_matches(handle, node.shared_id()) {
            return Err(WebDriverBiDiPointerClickAuthorityError::NodeExternalIdentifierMismatch);
        }

        Self::new(command_id, browsing_context, node)
            .map_err(WebDriverBiDiPointerClickAuthorityError::Command)
    }
}
