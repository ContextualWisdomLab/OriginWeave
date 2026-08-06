//! Fail-closed resolved-destination and redirect policy for OriginWeave.
//!
//! This crate performs no DNS lookup and opens no socket. A browser-network
//! adapter supplies resolved addresses and receives deterministic approval,
//! pinning, rebinding, connection, and redirect decisions.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod address;
mod redirect;
mod resolution;

pub use address::{AddressClass, ClassifiedAddress, classify_address};
pub use redirect::{
    MAX_REDIRECT_HOPS, RedirectEvidence, RedirectError, RedirectGuard,
    RedirectTargetDigest, RedirectTargetDigestError,
};
pub use resolution::{
    ConnectionEvidence, DestinationError, DestinationPolicy, ResolutionSnapshot,
};
