//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Deterministic fail-closed release benchmark acceptance aggregation.
pub mod release_acceptance;
