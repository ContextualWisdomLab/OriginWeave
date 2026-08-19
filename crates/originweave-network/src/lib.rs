//! Direct-only policy-bound TCP connection authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection, binds an RFC 6455
//! opening request to that verified plain stream, and can write that exact request
//! under one bounded deadline without claiming a completed WebSocket handshake or
//! granting browser, WebSocket, TLS, policy, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod webdriver_bidi_connection;
mod webdriver_bidi_websocket_handshake;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use webdriver_bidi_connection::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiTcpConnectionPlan,
};
pub use webdriver_bidi_websocket_handshake::{
    MAX_WEBSOCKET_OPENING_WRITE_TIMEOUT, WebDriverBiDiWebSocketClientKey,
    WebDriverBiDiWebSocketHandshakeError, WebDriverBiDiWebSocketHandshakePlan,
    WebDriverBiDiWebSocketOpeningRequestSent, WebDriverBiDiWebSocketOpeningWriteError,
};
