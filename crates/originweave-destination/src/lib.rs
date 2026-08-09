//! Fail-closed destination, redirect, and proxy-route policy for OriginWeave.
//!
//! This crate performs no DNS lookup and opens no socket. A browser-network
//! adapter supplies resolved addresses and selected routes and receives
//! deterministic approval, pinning, rebinding, connection, redirect, and route
//! decisions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod address;
mod proxy;
mod redirect;
mod resolution;

pub use address::{AddressClass, ClassifiedAddress, classify_address};
pub use proxy::{
    MAX_PAC_ORIGIN_COUNT, MAX_PROXY_SERVER_COUNT, ProxyRoute, ProxyRouteError, ProxyRouteEvidence,
    ProxyRouteKind, ProxyRoutePolicy, ProxyServer, ProxyServerError, ProxyServerScheme,
};
pub use redirect::{
    MAX_REDIRECT_HOPS, RedirectError, RedirectEvidence, RedirectGuard, RedirectTargetDigest,
    RedirectTargetDigestError,
};
pub use resolution::{
    ConnectionEvidence, DestinationError, DestinationPolicy, MAX_RESOLUTION_ADDRESS_COUNT,
    ResolutionSnapshot,
};