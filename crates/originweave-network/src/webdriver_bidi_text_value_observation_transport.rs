use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    AdmittedNodeHandle, BrowserAuthorityRegistry, BrowserProtocolCapability, BrowserProtocolKind,
    ValidatedBrowserProtocolUse, WebDriverBiDiRemoteNodeReference,
    WebDriverBiDiTextValueObservationAuthorityError, WebDriverBiDiTextValueObservationCommand,
};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

/// Fail-closed errors while transporting one current-authority text-value observation command.
#[derive(Debug)]
pub enum WebDriverBiDiTextValueObservationSendError {
    /// The supplied protocol-use proof belongs to another browser protocol family.
    UnsupportedProtocolKind(BrowserProtocolKind),
    /// The supplied protocol-use proof did not validate semantic-observation capability.
    UnsupportedCapability(BrowserProtocolCapability),
    /// Current node, browser-context, document, or bounded command authority failed revalidation.
    Authority {
        /// Exact typed immediate-use authority failure.
        source: WebDriverBiDiTextValueObservationAuthorityError,
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

impl fmt::Display for WebDriverBiDiTextValueObservationSendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedProtocolKind(_) => {
                "WebDriver BiDi text-value observation send requires a WebDriver BiDi proof"
            }
            Self::UnsupportedCapability(_) => {
                "WebDriver BiDi text-value observation send requires semantic-observation capability"
            }
            Self::Authority { .. } => {
                "WebDriver BiDi text-value observation authority was rejected"
            }
            Self::Correlation { .. } => {
                "WebDriver BiDi text-value observation command correlation was rejected"
            }
            Self::FrameWrite { .. } => {
                "WebDriver BiDi text-value observation command frame write failed"
            }
        })
    }
}

impl Error for WebDriverBiDiTextValueObservationSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedProtocolKind(_) | Self::UnsupportedCapability(_) => None,
            Self::Authority { source } => Some(source),
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

/// Revalidate, register, and write one fixed text-value `script.callFunction` observation.
///
/// The caller must transfer a non-cloneable [`ValidatedBrowserProtocolUse`] whose protocol family
/// is exactly [`BrowserProtocolKind::WebDriverBiDi`] and whose capability is exactly
/// [`BrowserProtocolCapability::SemanticObservation`]. The proof is consumed before node
/// authority, command correlation, or frame I/O, so typed-input, navigation, CDP, or other
/// protocol proofs cannot dispatch this observation through this boundary.
///
/// Immediately before correlation, this boundary reconstructs the fixed product-owned command
/// from the [`AdmittedNodeHandle`], exact external browsing-context identifier, remote node
/// reference, and live [`BrowserAuthorityRegistry`]. The core constructor revalidates current
/// session, context, origin, document epoch, registry provenance, and exact admitted wire node
/// identifier. Callers cannot supply function source, sandbox, or generic script arguments.
///
/// Registration records [`WebDriverBiDiCommandKind::TextValueObservation`] before the first
/// possible remote side effect. A later typed response boundary must match both the exact id and
/// this command provenance; an unrelated outstanding command id therefore cannot certify a text
/// post-condition. A correlation failure writes nothing. Once registration succeeds, a frame-write
/// failure leaves the identifier outstanding because partial or complete remote execution is
/// ambiguous and the identifier must not be silently reused.
///
/// Dispatch is only protocol-level observation transport. This function does not authenticate the
/// browser, authorize the preceding text-input action, compare the eventual remote value with the
/// intended non-secret text, infer post-condition success, retry, reconnect, select another
/// destination, or grant policy, destination, secret, process, profile, or Agent authority.
#[expect(
    clippy::too_many_arguments,
    reason = "this immediate-use security boundary keeps command identity, live node authority, transport, correlation, masking, and deadline inputs explicit rather than persisting a reusable prevalidated command"
)]
pub fn send_webdriver_bidi_text_value_observation(
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
) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiTextValueObservationSendError> {
    if validated.kind() != BrowserProtocolKind::WebDriverBiDi {
        return Err(
            WebDriverBiDiTextValueObservationSendError::UnsupportedProtocolKind(validated.kind()),
        );
    }
    if validated.capability() != BrowserProtocolCapability::SemanticObservation {
        return Err(
            WebDriverBiDiTextValueObservationSendError::UnsupportedCapability(
                validated.capability(),
            ),
        );
    }
    let _consumed_semantic_observation_proof = validated;

    let command = WebDriverBiDiTextValueObservationCommand::new_for_current_node(
        command_id,
        browsing_context,
        handle,
        node,
        registry,
    )
    .map_err(|source| WebDriverBiDiTextValueObservationSendError::Authority { source })?;

    correlation
        .register_command_for(
            command.command_id(),
            WebDriverBiDiCommandKind::TextValueObservation,
        )
        .map_err(|source| WebDriverBiDiTextValueObservationSendError::Correlation { source })?;
    established
        .write_text_frame(command.as_json(), masking_key, frame_timeout)
        .map_err(|source| WebDriverBiDiTextValueObservationSendError::FrameWrite { source })
}
