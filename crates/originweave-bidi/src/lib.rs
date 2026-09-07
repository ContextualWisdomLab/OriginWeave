//! Narrow WebDriver BiDi adapter contracts for OriginWeave browser sessions.
//!
//! This crate depends inward on presentation-identity values. It records only
//! capabilities that the pinned WebDriver BiDi specification can express; it
//! does not expose generic JavaScript or DevTools pass-through authority and it
//! does not claim that a command acknowledgement proves page-visible state.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod presentation_capabilities;

pub use presentation_capabilities::{
    WEBDRIVER_BIDI_PRESENTATION_REVISION, require_complete_presentation_profile,
    webdriver_bidi_presentation_surfaces,
};
