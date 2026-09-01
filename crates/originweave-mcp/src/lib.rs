//! Fail-closed MCP adapter contracts for OriginWeave.
//!
//! This crate owns MCP protocol-generation, discovery, and stateless tool-routing
//! contracts. It maps reviewed MCP protocol values into existing OriginWeave
//! action contracts but grants no policy, browser, network, secret, or evidence
//! authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub(crate) use originweave_core::{ActionKind, Capability, RiskClass};

mod routing;

pub use routing::*;
