//! Direct-only policy-bound TCP and WebSocket transport authority for OriginWeave.
//!
//! The crate consumes validated connection plans, opens exact socket addresses
//! without hostname resolution or proxy inheritance, verifies operating-system
//! peers before exposing transport I/O, and emits credential-free evidence.
//! It also bridges a session-correlated WebDriver BiDi loopback target from
//! `originweave-core` into one bounded exact TCP connection, binds and validates
//! the RFC 6455 opening exchange, provides bounded masked client writes and
//! unmasked server-frame reads, assembles bounded WebDriver BiDi text messages,
//! classifies complete local-end JSON envelopes, tracks bounded command-response
//! correlation, sends narrowly typed `session.status` and `session.end` commands,
//! admits typed correlated status and end responses, and keeps protocol teardown
//! acknowledgment separate from explicit operational teardown observations without
//! exposing generic JSON bodies or granting browser, TLS, policy, secret, or Agent authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod webdriver_bidi_command_correlation;
mod webdriver_bidi_connection;
mod webdriver_bidi_json_envelope;
mod webdriver_bidi_session_end_command;
mod webdriver_bidi_session_end_response;
mod webdriver_bidi_session_status_command;
mod webdriver_bidi_session_status_response;
mod webdriver_bidi_session_teardown;
mod webdriver_bidi_websocket_frame;
mod webdriver_bidi_websocket_handshake;
mod webdriver_bidi_websocket_message;
mod webdriver_bidi_websocket_opening_recovery;

#[cfg(test)]
mod webdriver_bidi_json_envelope_public_boundary_tests;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use webdriver_bidi_command_correlation::{
    MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS, WebDriverBiDiCommandCorrelation,
    WebDriverBiDiCommandCorrelationError, WebDriverBiDiCorrelatedResponse,
    WebDriverBiDiCorrelatedResponseOutcome,
};
pub use webdriver_bidi_connection::{
    WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError,
    WebDriverBiDiTcpConnectionEvidence, WebDriverBiDiTcpConnectionPlan,
};
pub use webdriver_bidi_json_envelope::{
    MAX_WEBDRIVER_BIDI_JS_UINT, MAX_WEBDRIVER_BIDI_JSON_DEPTH, WebDriverBiDiJsonEnvelope,
    WebDriverBiDiJsonEnvelopeError, WebDriverBiDiJsonEnvelopeKind,
};
pub use webdriver_bidi_session_end_command::{
    WebDriverBiDiSessionEndCommand, WebDriverBiDiSessionEndCommandError,
};
pub use webdriver_bidi_session_end_response::{
    WebDriverBiDiSessionEndResponseError, WebDriverBiDiSessionEndResult,
};
pub use webdriver_bidi_session_status_command::{
    WebDriverBiDiSessionStatusCommand, WebDriverBiDiSessionStatusCommandError,
};
pub use webdriver_bidi_session_status_response::{
    MAX_WEBDRIVER_BIDI_SESSION_STATUS_MESSAGE_SIZE, WebDriverBiDiSessionStatusResponseError,
    WebDriverBiDiSessionStatusResult,
};
pub use webdriver_bidi_session_teardown::{
    WebDriverBiDiSessionTeardownAssessment, WebDriverBiDiSessionTeardownDisposition,
    WebDriverBiDiSessionTeardownObservations,
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
