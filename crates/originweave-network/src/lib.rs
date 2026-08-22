//! Direct-only policy-bound TCP connection authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection, binds an RFC 6455
//! opening request to that verified plain stream, and can write that exact request
//! under one bounded deadline, validate its bounded RFC 6455 opening response,
//! carry one bounded frame at a time, and bind one exact `locateNodes` command to
//! its bounded correlated response without granting browser, WebSocket, TLS,
//! policy, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod webdriver_bidi_connection;
mod webdriver_bidi_locate_nodes_exchange;
#[cfg(test)]
mod webdriver_bidi_locate_nodes_exchange_transport_failure_tests;
mod webdriver_bidi_websocket_control;
#[cfg(test)]
#[allow(clippy::expect_used)]
mod webdriver_bidi_websocket_coverage_tests;
#[cfg(test)]
#[allow(clippy::expect_used)]
mod webdriver_bidi_websocket_debug_tests;
#[path = "webdriver_bidi_websocket_validated.rs"]
mod webdriver_bidi_websocket_handshake;
#[path = "webdriver_bidi_websocket_handshake.rs"]
mod webdriver_bidi_websocket_handshake_raw;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use webdriver_bidi_connection::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiTcpConnectionPlan,
};
pub use webdriver_bidi_locate_nodes_exchange::{
    MAX_WEBDRIVER_BIDI_CONTROL_FRAMES_PER_EXCHANGE, WebDriverBiDiLocateNodesExchangeError,
};
pub use webdriver_bidi_websocket_handshake::{
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketOpeningRequestSent,
};
pub use webdriver_bidi_websocket_handshake_raw::{
    MAX_WEBSOCKET_FRAME_PAYLOAD_SIZE, MAX_WEBSOCKET_FRAME_TIMEOUT,
    MAX_WEBSOCKET_OPENING_RESPONSE_SIZE, MAX_WEBSOCKET_OPENING_RESPONSE_TIMEOUT,
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketFrame, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketHandshakeError, WebDriverBiDiWebSocketHandshakeResponseError,
    WebDriverBiDiWebSocketMaskKey, WebDriverBiDiWebSocketOpeningWriteError,
};
