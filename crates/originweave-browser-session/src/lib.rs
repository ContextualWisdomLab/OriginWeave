//! Browser Session lifecycle authority for OriginWeave.
//!
//! The aggregate implementation remains isolated from recovery custody. Public callers receive only
//! the narrow domain surface re-exported here; the concrete lifecycle adapter is never exposed.
//!
//! The read-only Browser Session projection must not become a capability-minting escape hatch. Only
//! the bound owner exposes the explicit presentation-authority surface.
//!
//! ```compile_fail
//! use originweave_browser_session::{BoundBrowserSession, DisposableContextPort};
//! use originweave_core::BrowsingContextId;
//!
//! fn read_view_cannot_mint<P: DisposableContextPort>(
//!     bound: &BoundBrowserSession<P>,
//!     context: BrowsingContextId,
//! ) {
//!     let _ = bound.browser_session().presentation_authority(context);
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_session;
mod recovery;

pub use browser_session::*;
pub use recovery::BoundBrowserSessionRecovery;
