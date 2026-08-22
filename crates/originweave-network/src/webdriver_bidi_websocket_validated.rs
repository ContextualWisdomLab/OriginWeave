//! Validated public WebDriver BiDi WebSocket state wrappers.
//!
//! The underlying transport remains responsible for exact-stream I/O. These wrappers preserve the
//! public state machine while adding protocol validation that must run before a received frame is
//! released to callers.

use std::{fmt, time::Duration};

use originweave_core::VerifiedWebDriverBiDiSocketPeer;

use crate::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionEvidence,
    webdriver_bidi_websocket_handshake_raw as raw,
};

/// Inert RFC 6455 opening request bound to one already-verified plain BiDi TCP connection.
pub struct WebDriverBiDiWebSocketHandshakePlan(raw::WebDriverBiDiWebSocketHandshakePlan);

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
        client_key: raw::WebDriverBiDiWebSocketClientKey,
    ) -> Result<Self, raw::WebDriverBiDiWebSocketHandshakeError> {
        raw::WebDriverBiDiWebSocketHandshakePlan::new(connection, client_key).map(Self)
    }

    /// Borrow the exact serialized RFC 6455 opening-request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        self.0.request_bytes()
    }

    /// Borrow the exact client key that a later server-handshake validator must correlate.
    #[must_use]
    pub const fn client_key(&self) -> &raw::WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Borrow the exact peer/session evidence already verified before request construction.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        self.0.verified_peer()
    }

    /// Write the complete bounded opening request on the exact verified stream within one deadline.
    pub fn write_opening_request(
        self,
        write_timeout: Duration,
    ) -> Result<
        WebDriverBiDiWebSocketOpeningRequestSent,
        raw::WebDriverBiDiWebSocketOpeningWriteError,
    > {
        self.0
            .write_opening_request(write_timeout)
            .map(WebDriverBiDiWebSocketOpeningRequestSent)
    }
}

/// A live verified stream after the complete client WebSocket opening request has been written.
pub struct WebDriverBiDiWebSocketOpeningRequestSent(raw::WebDriverBiDiWebSocketOpeningRequestSent);

impl fmt::Debug for WebDriverBiDiWebSocketOpeningRequestSent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl WebDriverBiDiWebSocketOpeningRequestSent {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        self.0.transport_evidence()
    }

    /// Borrow the exact client key required to validate the later server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &raw::WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Return the exact number of opening-request bytes written before success was emitted.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.0.request_byte_count()
    }

    /// Return the total write deadline configured for this opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.0.write_timeout()
    }

    /// Read and validate the bounded RFC 6455 server opening response on this exact stream.
    pub fn read_opening_response(
        self,
        response_timeout: Duration,
    ) -> Result<WebDriverBiDiWebSocketEstablished, raw::WebDriverBiDiWebSocketHandshakeResponseError>
    {
        self.0
            .read_opening_response(response_timeout)
            .map(WebDriverBiDiWebSocketEstablished)
    }
}

/// A live verified stream after both RFC 6455 opening messages were validated.
pub struct WebDriverBiDiWebSocketEstablished(raw::WebDriverBiDiWebSocketEstablished);

impl fmt::Debug for WebDriverBiDiWebSocketEstablished {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl WebDriverBiDiWebSocketEstablished {
    /// Borrow the exact verified transport evidence retained with this live stream.
    #[must_use]
    pub const fn transport_evidence(&self) -> &WebDriverBiDiTcpConnectionEvidence {
        self.0.transport_evidence()
    }

    /// Borrow the exact client key correlated with the validated server accept value.
    #[must_use]
    pub const fn client_key(&self) -> &raw::WebDriverBiDiWebSocketClientKey {
        self.0.client_key()
    }

    /// Return the validated HTTP status code, currently always `101` on success.
    #[must_use]
    pub const fn response_status(&self) -> u16 {
        self.0.response_status()
    }

    /// Return the number of HTTP opening-response bytes consumed through its header terminator.
    #[must_use]
    pub const fn response_byte_count(&self) -> usize {
        self.0.response_byte_count()
    }

    /// Return the total response deadline configured for this opening response.
    #[must_use]
    pub const fn response_timeout(&self) -> Duration {
        self.0.response_timeout()
    }

    /// Return the number of request bytes written before the response was read.
    #[must_use]
    pub const fn request_byte_count(&self) -> usize {
        self.0.request_byte_count()
    }

    /// Return the total write deadline configured for the preceding opening request.
    #[must_use]
    pub const fn write_timeout(&self) -> Duration {
        self.0.write_timeout()
    }

    /// Write one unfragmented, masked UTF-8 text frame on this verified stream.
    pub fn write_text_frame(
        self,
        text: &str,
        masking_key: raw::WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, raw::WebDriverBiDiWebSocketFrameError> {
        self.0
            .write_text_frame(text, masking_key, frame_timeout)
            .map(Self)
    }

    /// Write one final masked RFC 6455 Pong control frame on this verified stream.
    pub fn write_pong_frame(
        self,
        payload: &[u8],
        masking_key: raw::WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, raw::WebDriverBiDiWebSocketFrameError> {
        self.0
            .write_pong_frame(payload, masking_key, frame_timeout)
            .map(Self)
    }

    /// Read one bounded RFC 6455 frame and reject close status codes forbidden on the wire.
    pub fn read_frame(
        self,
        frame_timeout: Duration,
    ) -> Result<(Self, raw::WebDriverBiDiWebSocketFrame), raw::WebDriverBiDiWebSocketFrameError>
    {
        let (established, frame) = self.0.read_frame(frame_timeout)?;
        validate_close_status_code(&frame)?;
        Ok((Self(established), frame))
    }
}

fn validate_close_status_code(
    frame: &raw::WebDriverBiDiWebSocketFrame,
) -> Result<(), raw::WebDriverBiDiWebSocketFrameError> {
    if frame.opcode() != 0x8 || frame.payload().len() < 2 {
        return Ok(());
    }

    let status_code = u16::from_be_bytes([frame.payload()[0], frame.payload()[1]]);
    if !(1000..=4999).contains(&status_code) || matches!(status_code, 1004 | 1005 | 1006 | 1015) {
        return Err(raw::WebDriverBiDiWebSocketFrameError::MalformedFrame {
            reason: "Close frame status code is not valid on the wire",
        });
    }
    Ok(())
}
