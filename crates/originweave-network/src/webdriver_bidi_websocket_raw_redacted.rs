//! Nonce-safe adapter around the bounded WebDriver BiDi WebSocket frame transport.
//!
//! The frame-transport implementation predates the opening-handshake diagnostic contract carried by
//! its parent stack. Keep that implementation private here and expose a handshake-plan wrapper whose
//! `Debug` output cannot render the serialized opening request or its `Sec-WebSocket-Key` nonce.

use std::{fmt, time::Duration};

use originweave_core::VerifiedWebDriverBiDiSocketPeer;

use crate::WebDriverBiDiTcpConnection;

#[path = "webdriver_bidi_websocket_handshake.rs"]
mod legacy;

pub use legacy::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_FRAME_TIMEOUT,
    MAX_WEBSOCKET_OPENING_RESPONSE_SIZE, MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrame,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakeError,
    WebDriverBiDiWebSocketHandshakeResponseError, WebDriverBiDiWebSocketMaskKey,
    WebDriverBiDiWebSocketOpeningRequestSent, WebDriverBiDiWebSocketOpeningWriteError,
};

/// Raw frame-transport opening plan with nonce-safe deterministic diagnostics.
///
/// The wrapped implementation retains the serialized opening request because it must later write
/// those exact bytes to the verified stream. This adapter deliberately keeps that implementation
/// private and exposes only diagnostic metadata: verified peer evidence, an explicit nonce-redaction
/// marker, and the bounded request length.
pub struct WebDriverBiDiWebSocketHandshakePlan(legacy::WebDriverBiDiWebSocketHandshakePlan);

impl fmt::Debug for WebDriverBiDiWebSocketHandshakePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebDriverBiDiWebSocketHandshakePlan")
            .field("verified_peer", self.0.verified_peer())
            .field("client_key", &"<redacted WebSocket client nonce>")
            .field("request_byte_count", &self.0.request_bytes().len())
            .finish()
    }
}

impl WebDriverBiDiWebSocketHandshakePlan {
    /// Bind one canonical opening request to an already-verified plain BiDi TCP connection.
    pub fn new(
        connection: WebDriverBiDiTcpConnection,
        client_key: WebDriverBiDiWebSocketClientKey,
    ) -> Result<Self, WebDriverBiDiWebSocketHandshakeError> {
        legacy::WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key).map(Self)
    }

    /// Borrow the exact serialized RFC 6455 opening-request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        self.0.request_bytes()
    }

    /// Borrow the exact client key required for later server-accept correlation.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Borrow the exact peer/session evidence verified before request construction.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        self.0.verified_peer()
    }

    /// Write the complete bounded opening request on the exact verified stream.
    pub fn write_opening_request(
        self,
        write_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketOpeningRequestSent, WebDriverBiDiWebSocketOpeningWriteError>
    {
        self.0.write_opening_request(write_timeout)
    }
}
