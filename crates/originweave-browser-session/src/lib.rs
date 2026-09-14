//! Browser Session lifecycle authority for OriginWeave.
//!
//! The aggregate implementation remains isolated from recovery custody. Public callers receive only
//! the narrow domain surface re-exported here; the concrete lifecycle adapter is never exposed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_session;
mod recovery;

pub use browser_session::*;
pub use recovery::BoundBrowserSessionRecovery;
