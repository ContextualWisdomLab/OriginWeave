use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    ValidatedWebDriverBiDiLocateNodesResponse, WebDriverBiDiAccessibilityQueryError,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiRemoteNodeReferenceError,
};

/// Fail-closed errors while admitting one correlated `locateNodes` result batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiLocateNodesResultAdmissionError {
    /// The returned node count exceeded the exact serialized command budget.
    Query(WebDriverBiDiAccessibilityQueryError),
    /// One returned item was not an admissible WebDriver BiDi node remote value.
    RemoteNode(WebDriverBiDiRemoteNodeReferenceError),
}

impl Display for WebDriverBiDiLocateNodesResultAdmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(error) => write!(
                formatter,
                "correlated locateNodes result violated the exact command budget: {error}"
            ),
            Self::RemoteNode(error) => write!(
                formatter,
                "correlated locateNodes result contained an inadmissible remote node: {error}"
            ),
        }
    }
}

impl Error for WebDriverBiDiLocateNodesResultAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::RemoteNode(error) => Some(error),
        }
    }
}

/// Non-cloneable evidence for one correlated, bounded, structurally admitted `locateNodes` result.
///
/// Construction consumes exact command-correlation evidence, validates the returned array length
/// against the exact `maxNodeCount` serialized by that command, and normalizes every returned item
/// through [`WebDriverBiDiRemoteNodeReference`]. The resulting batch therefore cannot be reused
/// with a different ambient query budget and cannot retain non-node remote values or unusable node
/// identifiers.
///
/// This is still transport evidence, not OriginWeave node authority. It does not parse raw JSON,
/// authenticate Chromium or its adapter, prove current session/context/origin/document authority,
/// mint [`crate::ObservedNodeHandle`] values, authorize policy or typed input, or establish an Agent
/// action. A later reviewed current-authority boundary must consume these normalized references.
#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedWebDriverBiDiLocateNodesResult {
    correlated: ValidatedWebDriverBiDiLocateNodesResponse,
    nodes: Vec<WebDriverBiDiRemoteNodeReference>,
}

impl ValidatedWebDriverBiDiLocateNodesResult {
    /// Return the exact command identifier proven to own this result batch.
    #[must_use]
    pub const fn command_id(&self) -> u64 {
        self.correlated.command_id()
    }

    /// Return the bounded browsing-context identifier serialized by the correlated command.
    #[must_use]
    pub fn browsing_context(&self) -> &str {
        self.correlated.browsing_context()
    }

    /// Return the exact `maxNodeCount` serialized by the correlated command.
    #[must_use]
    pub const fn max_node_count(&self) -> u16 {
        self.correlated.max_node_count()
    }

    /// Return the normalized untrusted node references admitted from the result array.
    #[must_use]
    pub fn nodes(&self) -> &[WebDriverBiDiRemoteNodeReference] {
        &self.nodes
    }
}

impl ValidatedWebDriverBiDiLocateNodesResponse {
    /// Consume exact command-correlation evidence and admit one structured `locateNodes` result.
    ///
    /// The result count is checked before any item is normalized so an over-budget response fails
    /// at the resource boundary even when its individual elements are malformed. Every in-budget
    /// item must then be the exact WebDriver BiDi `node` remote-value type and carry a usable
    /// `sharedId`. Success consumes the correlation evidence, preventing the same command response
    /// from being admitted repeatedly or against a different result payload.
    pub fn admit_result_nodes(
        self,
        items: &[(&str, Option<&str>)],
    ) -> Result<ValidatedWebDriverBiDiLocateNodesResult, WebDriverBiDiLocateNodesResultAdmissionError>
    {
        self.validate_result_count(items.len())
            .map_err(WebDriverBiDiLocateNodesResultAdmissionError::Query)?;

        let mut nodes = Vec::with_capacity(items.len());
        for (remote_type, shared_id) in items {
            nodes.push(
                WebDriverBiDiRemoteNodeReference::new(remote_type, *shared_id)
                    .map_err(WebDriverBiDiLocateNodesResultAdmissionError::RemoteNode)?,
            );
        }

        Ok(ValidatedWebDriverBiDiLocateNodesResult {
            correlated: self,
            nodes,
        })
    }
}
