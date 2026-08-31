use std::{error::Error, fmt, time::Duration};

use originweave_core::{
    BrowserProtocolCapability, BrowserProtocolKind, ValidatedBrowserProtocolUse,
    WebDriverBiDiPointerClickCommand,
};

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

/// Fail-closed errors while transporting one already validated pointer-click command.
#[derive(Debug)]
pub enum WebDriverBiDiPointerClickSendError {
    /// The supplied protocol-use proof belongs to another browser protocol family.
    UnsupportedProtocolKind(BrowserProtocolKind),
    /// The supplied protocol-use proof did not validate typed-input capability.
    UnsupportedCapability(BrowserProtocolCapability),
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
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

/// Register and write one validated `input.performActions` pointer-click command.
///
/// The caller must transfer a non-cloneable [`ValidatedBrowserProtocolUse`] whose protocol family
/// is exactly [`BrowserProtocolKind::WebDriverBiDi`] and whose capability is exactly
/// [`BrowserProtocolCapability::TypedInput`]. The proof is consumed before command correlation or
/// frame I/O, so semantic-observation, navigation, CDP, or other protocol proofs cannot dispatch a
/// pointer click through this transport boundary.
///
/// Registration occurs before the first possible remote side effect. A correlation failure therefore
/// writes nothing. Once registration succeeds, a frame-write failure leaves the identifier
/// outstanding because a partial or complete remote side effect is ambiguous and the identifier
/// must not be silently reused.
///
/// This boundary accepts only [`WebDriverBiDiPointerClickCommand`], not arbitrary JSON or method
/// names. Typed-input protocol validation is still not policy authorization: a trusted caller must
/// separately establish current session/context/origin/document/node authority and deterministic
/// policy approval before transport, then retain correlated response and observed post-condition
/// evidence afterward. This function does not authenticate the browser, grant destination or secret
/// authority, retry, reconnect, or choose another destination.
pub fn send_webdriver_bidi_pointer_click(
    validated: ValidatedBrowserProtocolUse,
    command: &WebDriverBiDiPointerClickCommand,
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

    correlation
        .register_command(command.command_id())
        .map_err(|source| WebDriverBiDiPointerClickSendError::Correlation { source })?;
    established
        .write_text_frame(command.as_json(), masking_key, frame_timeout)
        .map_err(|source| WebDriverBiDiPointerClickSendError::FrameWrite { source })
}
