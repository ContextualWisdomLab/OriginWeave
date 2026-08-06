//! Authenticate one canonical HTTPS origin over a verified direct TCP peer.
//!
//! OriginWeave TLS performs no DNS resolution, socket connection, proxy
//! discovery, HTTP parsing, Chromium control, or model inference. It consumes
//! an already verified direct TCP stream, applies WebPKI service-identity
//! validation with explicit roots and time, enforces bounded protocol policy,
//! and exposes the stream only with credential-free evidence.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod evidence;
mod handshake;
mod identity;
mod policy;
mod trust;

pub use error::TlsError;
pub use evidence::{
    AuthenticatedTlsConnection, NegotiatedAlpn, RevocationStatus, TlsConnectionEvidence,
    TlsProtocolVersion,
};
pub use handshake::TlsHandshakePlan;
pub use identity::TlsReferenceIdentity;
pub use policy::{
    AlpnRequirement, TlsClientPolicy, MAX_ALPN_PROTOCOL_COUNT, MAX_ALPN_PROTOCOL_LENGTH,
    MAX_ALPN_TOTAL_BYTES, MAX_SERVER_CERTIFICATE_BYTES, MAX_SERVER_CERTIFICATE_COUNT,
    MAX_TLS_HANDSHAKE_TIMEOUT,
};
pub use trust::{
    TrustBundleIdentifier, TrustRootBundle, MAX_TRUST_ROOT_BYTES, MAX_TRUST_ROOT_COUNT,
};
