//! Direct-only policy-bound TCP and WebSocket transport authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection, binds and validates
//! the RFC 6455 opening exchange, provides bounded masked client writes and
//! unmasked server-frame reads, and assembles bounded WebDriver BiDi text messages
//! without granting browser, TLS, policy, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod webdriver_bidi_connection;
mod webdriver_bidi_websocket_frame;
mod webdriver_bidi_websocket_handshake;
mod webdriver_bidi_websocket_message;
#[cfg(test)]
mod webdriver_bidi_websocket_message_public_contract;
mod webdriver_bidi_websocket_opening_recovery;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use webdriver_bidi_connection::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiTcpConnectionPlan,
};
pub use webdriver_bidi_websocket_frame::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_FRAME_TIMEOUT,
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrame,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketOpeningRequestSent,
};
pub use webdriver_bidi_websocket_handshake::{
    MAX_WEBSOCKET_OPENING_RESPONSE_SIZE, MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakeError, WebDriverBiDiWebSocketHandshakeResponseError,
    WebDriverBiDiWebSocketOpeningWriteError,
};
pub use webdriver_bidi_websocket_message::{
    MAX_WEBDRIVER_BIDI_MESSAGE_SIZE, WebDriverBiDiWebSocketControlKind,
    WebDriverBiDiWebSocketControlMessage, WebDriverBiDiWebSocketMessageAssembler,
    WebDriverBiDiWebSocketMessageAssembly, WebDriverBiDiWebSocketMessageError,
    WebDriverBiDiWebSocketTextMessage,
};
pub use webdriver_bidi_websocket_opening_recovery::WebDriverBiDiWebSocketOpeningWriteRecoveryDisposition;
