use std::{error::Error, fmt, time::Duration};

use originweave_core::WebDriverBiDiPointerClickCommand;

use crate::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiCommandCorrelationError,
    WebDriverBiDiCommandKind, WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey,
};

/// Fail-closed errors while transporting one already validated pointer-click command.
#[derive(Debug)]
pub enum WebDriverBiDiPointerClickSendError {
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
            Self::Correlation { source } => Some(source),
            Self::FrameWrite { source } => Some(source),
        }
    }
}

/// Register and write one already validated `input.performActions` pointer-click command.
///
/// Registration occurs before the first possible remote side effect. A correlation failure therefore
/// writes nothing. Once registration succeeds, a frame-write failure leaves the identifier
/// outstanding because a partial or complete remote side effect is ambiguous and the identifier
/// must not be silently reused.
///
/// This boundary accepts only [`WebDriverBiDiPointerClickCommand`], not arbitrary JSON or method
/// names. It does not authenticate the browser, grant session/context/origin/document-epoch
/// authority, authorize policy or TypedInput capability, admit nodes, correlate a response, prove an
/// observed post-condition, retry, reconnect, or choose another destination. A trusted caller must
/// establish those independent authorities before transport and retain response/post-condition
/// evidence afterward.
pub fn send_webdriver_bidi_pointer_click(
    command: &WebDriverBiDiPointerClickCommand,
    established: WebDriverBiDiWebSocketEstablished,
    correlation: &mut WebDriverBiDiCommandCorrelation,
    masking_key: WebDriverBiDiWebSocketMaskKey,
    frame_timeout: Duration,
) -> Result<WebDriverBiDiWebSocketEstablished, WebDriverBiDiPointerClickSendError> {
    correlation
        .register_command_for(command.command_id(), WebDriverBiDiCommandKind::PointerClick)
        .map_err(|source| WebDriverBiDiPointerClickSendError::Correlation { source })?;
    established
        .write_text_frame(command.as_json(), masking_key, frame_timeout)
        .map_err(|source| WebDriverBiDiPointerClickSendError::FrameWrite { source })
}
