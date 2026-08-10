//! Direct-only policy-bound TCP connection authority for OriginWeave.
//!
//! The crate consumes a validated connection plan, opens one exact socket
//! address without hostname resolution or proxy inheritance, verifies the
//! operating-system peer, and emits credential-free evidence.
//!
//! Direct planning from an untimed resolution snapshot is intentionally not a
//! public capability. External callers must cross the fresh-resolution boundary
//! before they can obtain socket authority.
//!
//! ```compile_fail
//! use originweave_network::ConnectionPlan;
//!
//! fn stale_resolution_bypass(_: Option<ConnectionPlan>) {}
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;
mod fresh_connection;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS,
    NetworkError, SocketConnectionEvidence,
};
pub use fresh_connection::FreshConnectionPlan;
