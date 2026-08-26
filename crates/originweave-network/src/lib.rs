//! Direct-only policy-bound TCP connection authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection and can bind an inert
//! RFC 6455 opening request to an already-verified plain BiDi stream without
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
    WebDriverBiDiWebSocketClientKey, WebDriverBiDiWebSocketHandshakeError,
    WebDriverBiDiWebSocketHandshakePlan,
};
