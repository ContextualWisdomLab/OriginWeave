//! Fixed-point statistical thresholds for zero-observed-event benchmark safety evidence.
//!
//! Thresholds in this module decide only whether a zero-event observation is statistically
//! strong enough for a declared release policy. They do not convert insufficient evidence
//! into product success or product failure and do not grant release authority.

use std::fmt;

use crate::release_acceptance::ZeroEventSafetyEvidence;

/// One event per trial expressed as parts per million.
pub const MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION: u32 = 1_000_000;

/// Result of evaluating zero-event evidence against one explicit safety threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroEventSafetyThresholdOutcome {
    /// The evidence meets both the minimum confidence and maximum upper-rate requirements.
    Satisfied,
    /// The evidence confidence is below the threshold's required minimum.
    InsufficientConfidence,
    /// The one-sided upper event-rate bound is still above the permitted maximum.
    UpperBoundExceedsThreshold,
}

/// Validation error while constructing a zero-event safety threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroEventSafetyThresholdError {
    /// The maximum allowed upper event rate exceeded one event per trial.
    InvalidUpperRatePartsPerMillion,
    /// The minimum one-sided confidence was outside `1..=9999` basis points.
    InvalidConfidenceBasisPoints,
}

impl fmt::Display for ZeroEventSafetyThresholdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUpperRatePartsPerMillion => formatter.write_str(
                "zero-event safety upper-rate threshold must be at most 1000000 parts per million",
            ),
            Self::InvalidConfidenceBasisPoints => formatter.write_str(
                "zero-event safety threshold confidence must be between 1 and 9999 basis points",
            ),
        }
    }
}

impl std::error::Error for ZeroEventSafetyThresholdError {}

/// Fixed-point release policy for one zero-observed-event safety metric.
///
/// The rate threshold is retained in integer parts per million rather than caller-supplied
/// floating point. A finite zero-event run whose upper confidence bound is above the limit
/// remains insufficient evidence; it is not reclassified as a known product failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroEventSafetyThreshold {
    maximum_upper_event_rate_parts_per_million: u32,
    minimum_confidence_basis_points: u16,
}

impl ZeroEventSafetyThreshold {
    /// Construct a validated fixed-point zero-event safety threshold.
    pub fn new(
        maximum_upper_event_rate_parts_per_million: u32,
        minimum_confidence_basis_points: u16,
    ) -> Result<Self, ZeroEventSafetyThresholdError> {
        if maximum_upper_event_rate_parts_per_million > MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION {
            return Err(ZeroEventSafetyThresholdError::InvalidUpperRatePartsPerMillion);
        }
        if minimum_confidence_basis_points == 0 || minimum_confidence_basis_points >= 10_000 {
            return Err(ZeroEventSafetyThresholdError::InvalidConfidenceBasisPoints);
        }
        Ok(Self {
            maximum_upper_event_rate_parts_per_million,
            minimum_confidence_basis_points,
        })
    }

    /// Return the largest accepted one-sided upper event-rate bound, in parts per million.
    #[must_use]
    pub const fn maximum_upper_event_rate_parts_per_million(self) -> u32 {
        self.maximum_upper_event_rate_parts_per_million
    }

    /// Return the minimum accepted one-sided confidence level, in basis points.
    #[must_use]
    pub const fn minimum_confidence_basis_points(self) -> u16 {
        self.minimum_confidence_basis_points
    }

    /// Evaluate one zero-event observation without promoting insufficient evidence to success.
    #[must_use]
    pub fn evaluate(self, evidence: ZeroEventSafetyEvidence) -> ZeroEventSafetyThresholdOutcome {
        if evidence.confidence_basis_points() < self.minimum_confidence_basis_points {
            return ZeroEventSafetyThresholdOutcome::InsufficientConfidence;
        }

        let maximum_upper_event_rate = f64::from(self.maximum_upper_event_rate_parts_per_million)
            / f64::from(MAX_SAFETY_EVENT_RATE_PARTS_PER_MILLION);
        if evidence.upper_event_rate() <= maximum_upper_event_rate {
            ZeroEventSafetyThresholdOutcome::Satisfied
        } else {
            ZeroEventSafetyThresholdOutcome::UpperBoundExceedsThreshold
        }
    }
}
