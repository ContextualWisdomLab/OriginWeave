//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while independently reusable
//! product responsibilities move into focused bounded contexts.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;

/// Temporary compatibility path for the protected-main release-acceptance API.
///
/// New code must depend on `originweave-release` directly. This re-export exists only to avoid an
/// unannounced break for the pre-alpha protected-main `originweave_core::release_acceptance` path
/// while callers migrate; no release domain behavior is implemented in core.
#[deprecated(note = "depend on originweave-release and use originweave_release::release_acceptance")]
pub use originweave_release::release_acceptance;

/// Temporary compatibility path for active-branch benchmark failure consumers.
#[deprecated(note = "depend on originweave-release and use originweave_release::benchmark_failure")]
pub use originweave_release::benchmark_failure;

/// Temporary compatibility path for active-branch zero-event safety-gate consumers.
#[deprecated(note = "depend on originweave-release and use originweave_release::zero_event_safety_gate")]
pub use originweave_release::zero_event_safety_gate;

/// Temporary compatibility path for active-branch zero-event threshold consumers.
#[deprecated(note = "depend on originweave-release and use originweave_release::zero_event_threshold")]
pub use originweave_release::zero_event_threshold;
