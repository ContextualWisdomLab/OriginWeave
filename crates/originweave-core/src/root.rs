//! Shared security and governance contracts for OriginWeave.
//!
//! The historical core contracts remain source-compatible while adapter-specific
//! boundaries can live in focused modules without changing their authority model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;

mod attached_tab_assurance;
pub use attached_tab_assurance::*;

mod native_messaging;
pub use native_messaging::*;
