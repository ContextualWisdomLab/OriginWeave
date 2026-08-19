use std::fmt;

use originweave_core::VerifiedWebDriverBiDiSocketPeer;

use crate::WebDriverBiDiTcpConnection;

const WEBSOCKET_CLIENT_KEY_LENGTH: usize = 24;

fn is_base64_data_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/')
}

fn is_canonical_16_byte_base64(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == WEBSOCKET_CLIENT_KEY_LENGTH
        && bytes[..22].iter().copied().all(is_base64_data_byte)
        && matches!(bytes[21], b'A' | b'Q' | b'g' | b'w')
        && bytes[22] == b'='
        && bytes[23] == b'='
}

/// Deterministic failures while preparing one WebDriver BiDi RFC 6455 opening request.
#[derive(Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketHandshakeError {
    /// The supplied client key was not the canonical base64 representation of exactly 16 bytes.
    InvalidClientKey,
    /// The verified WebDriver BiDi target requires TLS before a WebSocket opening request is sent.
    TlsRequired,
}

impl fmt::Display for WebDriverBiDiWebSocketHandshakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClientKey => formatter.write_str(
                "WebDriver BiDi WebSocket client key is not canonical base64 for exactly 16 bytes",
            ),
            Self::TlsRequired => formatter.write_str(
                "WebDriver BiDi WebSocket target requires authenticated TLS before the opening request",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiWebSocketHandshakeError {}

/// Canonical RFC 6455 client key for one WebDriver BiDi opening handshake.
///
/// RFC 6455 requires `Sec-WebSocket-Key` to be a nonce of 16 bytes encoded with base64. This type
/// validates only the canonical wire representation, including zero padding bits. It does not
/// generate entropy: callers remain responsible for supplying a fresh, unpredictable 16-byte nonce
/// for each connection attempt.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketClientKey(String);

impl WebDriverBiDiWebSocketClientKey {
    /// Admit one canonical base64 client key representing exactly 16 bytes.
    pub fn new(value: &str) -> Result<Self, WebDriverBiDiWebSocketHandshakeError> {
        if !is_canonical_16_byte_base64(value) {
            return Err(WebDriverBiDiWebSocketHandshakeError::InvalidClientKey);
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the exact canonical value for `Sec-WebSocket-Key` serialization.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Inert RFC 6455 opening request bound to one already-verified plain BiDi TCP connection.
///
/// The plan consumes the verified TCP connection so the opening request cannot be detached from the
/// socket peer/session evidence that authorized its exact loopback destination. It serializes only
/// the fixed WebSocket version-13 request required for the admitted `/session/<session-id>` resource
/// and retains the exact client key required to validate a later `Sec-WebSocket-Accept` response.
/// Secure `wss` targets fail closed here and require a separate authenticated TLS transport boundary
/// before any WebSocket bytes may be written.
///
/// Construction performs no socket write, TLS operation, response parsing, `Sec-WebSocket-Accept`
/// validation, WebSocket framing, Chromium/ChromeDriver process authentication, browser action, or
/// Agent-authority grant.
#[derive(Debug)]
pub struct WebDriverBiDiWebSocketHandshakePlan {
    connection: WebDriverBiDiTcpConnection,
    client_key: WebDriverBiDiWebSocketClientKey,
    request: Vec<u8>,
}

impl WebDriverBiDiWebSocketHandshakePlan {
    /// Bind one canonical opening request to an already-verified plain BiDi TCP connection.
    pub fn new(
        connection: WebDriverBiDiTcpConnection,
        client_key: WebDriverBiDiWebSocketClientKey,
    ) -> Result<Self, WebDriverBiDiWebSocketHandshakeError> {
        if connection.verified_peer().requires_tls() {
            return Err(WebDriverBiDiWebSocketHandshakeError::TlsRequired);
        }

        let peer = connection.verified_peer();
        let request = format!(
            "GET /session/{} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
            peer.session_id(),
            peer.socket_addr(),
            client_key.as_str(),
        )
        .into_bytes();

        Ok(Self {
            connection,
            client_key,
            request,
        })
    }

    /// Borrow the exact serialized RFC 6455 opening-request bytes.
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        &self.request
    }

    /// Borrow the exact client key that a later server-handshake validator must correlate.
    #[must_use]
    pub const fn client_key(&self) -> &WebDriverBiDiWebSocketClientKey {
        &self.client_key
    }

    /// Borrow the exact peer/session evidence already verified before request construction.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        self.connection.verified_peer()
    }
}
