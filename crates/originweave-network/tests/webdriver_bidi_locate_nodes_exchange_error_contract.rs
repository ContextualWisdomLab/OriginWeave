use std::{error::Error as _, time::Duration};

use originweave_core::{
    WebDriverBiDiLocateNodesResponseDocumentError, WebDriverBiDiResponseDocumentAdmissionError,
};
use originweave_network::{
    MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiLocateNodesExchangeError,
    WebDriverBiDiWebSocketFrameError,
};

#[test]
fn downstream_callers_observe_exact_exchange_error_sources() {
    let frame = WebDriverBiDiLocateNodesExchangeError::Frame(
        WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
            frame_timeout: Duration::ZERO,
            maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
        },
    );
    assert!(frame.source().is_some());

    let document = WebDriverBiDiLocateNodesExchangeError::ResponseDocument(
        WebDriverBiDiResponseDocumentAdmissionError::InvalidUtf8,
    );
    assert!(document.source().is_some());

    let response = WebDriverBiDiLocateNodesExchangeError::LocateNodesResponse(
        WebDriverBiDiLocateNodesResponseDocumentError::MissingResultNodes,
    );
    assert!(response.source().is_some());

    let source_free_errors = [
        WebDriverBiDiLocateNodesExchangeError::ExchangeDeadlineExceeded {
            exchange_timeout: Duration::from_millis(500),
        },
        WebDriverBiDiLocateNodesExchangeError::PongMaskingKeyUnavailable,
        WebDriverBiDiLocateNodesExchangeError::UnexpectedResponseFrame {
            fin: false,
            opcode: 0x2,
        },
    ];
    for error in source_free_errors {
        assert!(error.source().is_none(), "{error:?}");
    }
}
