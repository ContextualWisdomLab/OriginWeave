//! Trusted, purpose-bound sensitive-value broker contracts for OriginWeave.
//!
//! The crate is intentionally separate from pure policy evaluation. It will own
//! caller-unforgeable handle state, bounded protected values, trusted time,
//! revocation, and atomic use reservation before any value reaches a trusted
//! consumer. Planner/model code must never receive the underlying protected bytes.
