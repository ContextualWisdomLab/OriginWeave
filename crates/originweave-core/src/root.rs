//! Shared security and governance contracts for OriginWeave.
//!
//! The shared kernel owns stable cross-context security vocabulary. Independently reusable
//! product responsibilities belong to their focused bounded contexts and must not depend back
//! through this crate merely to preserve historical module paths.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod contracts;

pub use contracts::*;

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;
