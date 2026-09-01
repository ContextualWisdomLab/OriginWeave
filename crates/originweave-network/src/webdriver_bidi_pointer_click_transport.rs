use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, BrowserProtocolCapability, BrowserProtocolKind,
    ValidatedBrowserProtocolUse, WebDriverBiDiPointerClickAuthorityError,
    WebDriverBiDiPointerClickCommand, WebDriverBiDiRemoteNodeReference,
};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

/// Fail-closed errors while transporting one current-authority pointer click.
#[derive(Debug)]
pub enum WebDriverBiDiPointerClickSendError {
    /// The supplied protocol-use proof belongs to another browser protocol family.
    UnsupportedProtocolKind(BrowserProtocolKind),
    /// The supplied protocol-use proof did not validate typed-input capability.
    UnsupportedCapability(BrowserProtocolCapability),
    /// The node, browser-context, document, or bounded command authority failed immediate revalidation.
    Authority {
        /// Exact typed immediate-use authority failure.
        source: WebDriverBiDiPointerClickAuthorityError,
    },
    /// The bounded correlation registry rejected the command before network I/O.
    Correlation {
        /// Exact typed correlation failure.
        source: WebDriverBiDiCommandCorrelationError,
    },
    /// Writing the already-registered command frame failed and the transport is not reusable.
    FrameWrite {
        /// Exact typed bounded WebSocket frame-write failure.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiPointerClickSendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedProtocolKind(_) => {
                "WebDriver BiDi pointer-click send requires a WebDriver BiDi proof"
            }
            Self::UnsupportedCapability(_) => {
                "WebDriver BiDi pointer-click send requires typed-input capability"
            }
            Self::Authority { .. } => "WebDriver BiDi pointer-click node authority was rejected",
            Self::Correlation { .. } => {
                "WebDriver BiDi pointer-click command correlation was rejected"
            }
            Self::FrameWrite { .. } => "WebDriver BiDi pointer-click command frame write failed",
        })
    }
}

impl Error for WebDriverBiDiPointerClickSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedProtocolKind(_) | Self::UnsupportedCapability(_) => None,
            Self::Authority { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

/// Revalidate, register, and write one `input.performActions` pointer click.
///
/// The caller must transfer a non-cloneable [`ValidatedBrowserProtocolUse`] whose protocol family
/// is exactly [`BrowserProtocolKind::WebDriverBiDi`] and whose capability is exactly
/// [`BrowserProtocolCapability::TypedInput`]. The proof is consumed before node authority,
/// command correlation, or frame I/O, so semantic-observation, navigation, CDP, or other protocol
/// proofs cannot dispatch a pointer click through this transport boundary.
///
/// After protocol validation and immediately before correlation, this boundary reconstructs the
/// bounded pointer command from the exact [`AdmittedNodeHandle`], external browsing-context
/// identifier, remote node reference, and live [`BrowserAuthorityRegistry`]. That immediate-use
/// check rejects stale document epochs, cross-registry handles, changed origins, mismatched external
/// contexts, and unadmitted wire node identifiers before any command identifier is registered or
/// any action frame is written. A previously constructed command therefore cannot outlive its node
/// authority and later bypass revalidation at transport time.
///
/// Registration occurs before the first possible remote side effect. A correlation failure therefore
/// writes nothing. Once registration succeeds, a frame-write failure leaves the identifier
/// outstanding because a partial or complete remote side effect is ambiguous and the identifier
/// must not be silently reused.
///
/// Typed-input and node authority validation are still not policy authorization. A trusted caller
/// must separately establish deterministic policy approval and destination authority, then retain
/// correlated response and observed post-condition evidence afterward. This function does not
/// authenticate the browser, grant destination or secret authority, retry, reconnect, or choose
/// another destination.
#[expect(
    clippy::too_many_arguments,
    reason = "this immediate-use security boundary keeps command identity, live node authority, transport, correlation, masking, and deadline inputs explicit rather than persisting a reusable prevalidated command"
)]
pub fn send_webdriver_bidi_pointer_click(
    validated: ValidatedBrowserProtocolUse,
    command_id: u64,
    browsing_context: &str,
    handle: &AdmittedNodeHandle,
    node: &WebDriverBiDiRemoteNodeReference,
    registry: &BrowserAuthorityRegistry,
    established: WebDriverBiDiWebSocketEstablished,
    correlation: &mut WebDriverBiDiCommandCorrelation,
    masking_key: WebDriverBiDiWebSocketMaskKey,
    frame_timeout: Duration,
) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiPointerClickSendError> {
    if validated.kind() != BrowserProtocolKind::WebDriverBiDi {
        return Err(WebDriverBiDiPointerClickSendError::UnsupportedProtocolKind(
            validated.kind(),
        ));
    }
    if validated.capability() != BrowserProtocolCapability::TypedInput {
        return Err(WebDriverBiDiPointerClickSendError::UnsupportedCapability(
            validated.capability(),
        ));
    }
    let _consumed_typed_input_proof = validated;

    let command = WebDriverBiDiPointerClickCommand::new_for_current_node(
        command_id,
        browsing_context,
        handle,
        node,
        registry,
    )
    .map_err(|source| WebDriverBiDiPointerClickSendError::Authority { source })?;

    correlation
        .register_command(command.command_id())
        .map_err(|source| WebDriverBiDiPointerClickSendError::Correlation { source })?;
    established
        .write_text_frame(command.as_json(), masking_key, frame_timeout)
        .map_err(|source| WebDriverBiDiPointerClickSendError::FrameWrite { source })
}
