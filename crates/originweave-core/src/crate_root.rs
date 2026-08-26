//! OriginWeave core contracts plus narrowly scoped adapter authority boundaries.
//!
//! The existing deterministic core remains implemented in `lib.rs`; this crate
//! root re-exports that protected-main API and adds the independently reviewed
//! Chrome-permission separation boundary without weakening existing authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod base;
pub use base::*;

mod chrome_permission_authority;
pub use chrome_permission_authority::{
    ChromePermissionAuthorityError, chrome_permission_authorizes_agent_action,
};

/// Stateless MCP routing validation that maps only explicit tools to typed actions.
pub mod mcp;
