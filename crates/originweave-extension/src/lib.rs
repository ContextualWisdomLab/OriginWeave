//! Extension Policy bounded context for Chromium-compatible extension integration.
//!
//! This crate owns Chrome-specific extension adapter vocabulary and authority checks.
//! Stable browser/session/action contracts remain in `originweave-core`; this context
//! depends inward on those contracts without exporting Chromium adapter types back into core.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use originweave_core::ExtensionId;

mod native_messaging;
pub use native_messaging::*;
