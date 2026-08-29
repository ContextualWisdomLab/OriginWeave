//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Typed fail-closed benchmark failure classification.
pub mod benchmark_failure;
/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;
/// Deterministic fail-closed release benchmark acceptance aggregation.
pub mod release_acceptance;
/// Fixed-point statistical thresholds for zero-event benchmark safety evidence.
pub mod zero_event_threshold;
