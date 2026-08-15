//! Shared security and governance contracts for OriginWeave.
//!
//! This crate keeps the long-lived value contracts in `contracts` and the
//! protocol-identifier registry in a focused module so browser adapters can
//! evolve without turning raw CDP or WebDriver identifiers into authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod browser_registry;
mod contracts;

pub use browser_registry::{
    BrowserAuthorityRegistry, BrowserRegistryError, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES,
};
pub use contracts::*;
