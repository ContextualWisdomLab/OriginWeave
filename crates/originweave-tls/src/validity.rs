use std::fmt;
use std::time::Duration;

/// A product safety budget requiring a leaf certificate to remain valid for a
/// minimum duration beyond the trusted certificate-validation time.
///
/// This guard does not change RFC 5280 validity semantics. It is intended for
/// adapters that need point-in-time WebPKI success plus enough remaining leaf
/// validity for a bounded delegated task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafValidityHorizon {
    minimum_remaining: Duration,
}

impl LeafValidityHorizon {
    /// Create a validity-horizon guard.
    ///
    /// Fractional seconds are permitted and are conservatively rounded up when
    /// compared with X.509 validity timestamps, which have whole-second
    /// resolution in the evidence contract.
    #[must_use]
    pub const fn new(minimum_remaining: Duration) -> Self {
        Self { minimum_remaining }
    }

    /// Return the configured minimum remaining leaf-certificate validity.
    #[must_use]
    pub const fn minimum_remaining(self) -> Duration {
        self.minimum_remaining
    }

    /// Evaluate trusted-time and leaf-`notAfter` values from authenticated TLS
    /// evidence against this safety budget.
    ///
    /// A negative or already-expired `notAfter` value has zero remaining
    /// validity. Fractional minimum durations round up to the next whole second
    /// so the guard never grants less time than requested.
    pub fn evaluate(
        self,
        trusted_time_unix_seconds: u64,
        leaf_not_after_unix_seconds: i64,
    ) -> Result<(), LeafValidityHorizonError> {
        let minimum_seconds = minimum_seconds_rounded_up(self.minimum_remaining);
        let remaining_seconds = u64::try_from(leaf_not_after_unix_seconds)
            .ok()
            .and_then(|not_after| not_after.checked_sub(trusted_time_unix_seconds))
            .unwrap_or(0);

        if remaining_seconds >= minimum_seconds {
            Ok(())
        } else {
            Err(LeafValidityHorizonError::InsufficientRemainingValidity {
                remaining_seconds,
                minimum_seconds,
            })
        }
    }
}

const fn minimum_seconds_rounded_up(duration: Duration) -> u64 {
    if duration.subsec_nanos() == 0 {
        duration.as_secs()
    } else {
        duration.as_secs().saturating_add(1)
    }
}

/// A deterministic reason that a leaf certificate cannot satisfy a delegated
/// task's required validity horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafValidityHorizonError {
    /// The authenticated leaf certificate expires before the required horizon.
    InsufficientRemainingValidity {
        /// Whole seconds remaining after the trusted validation time.
        remaining_seconds: u64,
        /// Whole seconds required after conservative fractional-second rounding.
        minimum_seconds: u64,
    },
}

impl fmt::Display for LeafValidityHorizonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientRemainingValidity {
                remaining_seconds,
                minimum_seconds,
            } => write!(
                formatter,
                "TLS leaf certificate has {remaining_seconds} seconds remaining; delegated task requires at least {minimum_seconds} seconds",
            ),
        }
    }
}

impl std::error::Error for LeafValidityHorizonError {}
