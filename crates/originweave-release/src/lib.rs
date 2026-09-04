//! Deterministic, fail-closed commercial release-acceptance evidence contracts.
//!
//! This bounded context owns benchmark failure causality, bounded release evidence,
//! and zero-event statistical safety gates. It performs no benchmark I/O and grants
//! no merge, tag, publication, or operator release authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Typed fail-closed benchmark failure classification.
pub mod benchmark_failure;
/// Deterministic fail-closed release benchmark acceptance aggregation.
pub mod release_acceptance;
/// Fail-closed evaluation of declared zero-event benchmark safety requirements.
pub mod zero_event_safety_gate;
/// Fixed-point statistical thresholds for zero-event benchmark safety evidence.
pub mod zero_event_threshold;
