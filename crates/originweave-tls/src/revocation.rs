use std::fmt;

/// A deterministic freshness window for independently verified revocation material.
///
/// This value does not fetch, parse, authenticate, or interpret OCSP responses or
/// certificate revocation lists. A trusted adapter must first obtain and
/// cryptographically validate the revocation material, then pass the signed
/// `thisUpdate` and `nextUpdate` timestamps into this authority together with a
/// caller-selected local maximum freshness window. Passing this check proves only
/// that the supplied material is within both its signed interval and the caller's
/// bounded freshness policy; it does not prove that any certificate is unrevoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationMaterialFreshness {
    this_update_unix_seconds: u64,
    next_update_unix_seconds: u64,
    maximum_window_seconds: u64,
}

impl RevocationMaterialFreshness {
    /// Create a non-empty, locally bounded freshness window from trusted signed timestamps.
    ///
    /// The signed window is half-open: `thisUpdate <= trusted_time < nextUpdate`.
    /// Equal or reversed timestamps fail closed because they provide no usable
    /// interval. `maximum_window_seconds` is a separate local policy ceiling and
    /// must be nonzero; signed material whose declared interval exceeds that
    /// ceiling is rejected even if its timestamps are otherwise well-formed.
    pub const fn new(
        this_update_unix_seconds: u64,
        next_update_unix_seconds: u64,
        maximum_window_seconds: u64,
    ) -> Result<Self, RevocationMaterialFreshnessError> {
        if next_update_unix_seconds <= this_update_unix_seconds {
            return Err(RevocationMaterialFreshnessError::InvalidWindow {
                this_update_unix_seconds,
                next_update_unix_seconds,
            });
        }
        if maximum_window_seconds == 0 {
            return Err(RevocationMaterialFreshnessError::ZeroMaximumWindow);
        }

        let window_seconds = next_update_unix_seconds - this_update_unix_seconds;
        if window_seconds > maximum_window_seconds {
            return Err(RevocationMaterialFreshnessError::WindowExceedsMaximum {
                window_seconds,
                maximum_window_seconds,
            });
        }

        Ok(Self {
            this_update_unix_seconds,
            next_update_unix_seconds,
            maximum_window_seconds,
        })
    }

    /// Return the signed time at which the revocation material becomes current.
    #[must_use]
    pub const fn this_update_unix_seconds(self) -> u64 {
        self.this_update_unix_seconds
    }

    /// Return the signed time at which this freshness window stops being usable.
    #[must_use]
    pub const fn next_update_unix_seconds(self) -> u64 {
        self.next_update_unix_seconds
    }

    /// Return the caller-selected maximum accepted signed-window duration.
    #[must_use]
    pub const fn maximum_window_seconds(self) -> u64 {
        self.maximum_window_seconds
    }

    /// Evaluate one trusted time against the half-open freshness window.
    ///
    /// A time before `thisUpdate` is not yet usable. A time equal to or later
    /// than `nextUpdate` is stale. Both cases fail closed without making any
    /// statement about the certificate's revocation state.
    pub const fn evaluate(
        self,
        trusted_time_unix_seconds: u64,
    ) -> Result<(), RevocationMaterialFreshnessError> {
        if trusted_time_unix_seconds < self.this_update_unix_seconds {
            Err(RevocationMaterialFreshnessError::NotYetValid {
                trusted_time_unix_seconds,
                this_update_unix_seconds: self.this_update_unix_seconds,
            })
        } else if trusted_time_unix_seconds >= self.next_update_unix_seconds {
            Err(RevocationMaterialFreshnessError::Expired {
                trusted_time_unix_seconds,
                next_update_unix_seconds: self.next_update_unix_seconds,
            })
        } else {
            Ok(())
        }
    }
}

/// A deterministic reason that verified revocation material is not fresh enough to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationMaterialFreshnessError {
    /// The supplied signed timestamps do not define a non-empty freshness window.
    InvalidWindow {
        /// Signed `thisUpdate` timestamp in Unix seconds.
        this_update_unix_seconds: u64,
        /// Signed `nextUpdate` timestamp in Unix seconds.
        next_update_unix_seconds: u64,
    },
    /// The caller supplied no positive local maximum freshness duration.
    ZeroMaximumWindow,
    /// The material's signed interval exceeds the caller's local freshness ceiling.
    WindowExceedsMaximum {
        /// Duration of the signed `thisUpdate` to `nextUpdate` interval in seconds.
        window_seconds: u64,
        /// Caller-selected maximum accepted interval in seconds.
        maximum_window_seconds: u64,
    },
    /// Trusted time falls before the material's signed `thisUpdate` timestamp.
    NotYetValid {
        /// Trusted evaluation time in Unix seconds.
        trusted_time_unix_seconds: u64,
        /// Signed `thisUpdate` timestamp in Unix seconds.
        this_update_unix_seconds: u64,
    },
    /// Trusted time is equal to or later than the material's signed `nextUpdate` timestamp.
    Expired {
        /// Trusted evaluation time in Unix seconds.
        trusted_time_unix_seconds: u64,
        /// Signed `nextUpdate` timestamp in Unix seconds.
        next_update_unix_seconds: u64,
    },
}

impl fmt::Display for RevocationMaterialFreshnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindow {
                this_update_unix_seconds,
                next_update_unix_seconds,
            } => write!(
                formatter,
                "revocation material window is invalid: thisUpdate {this_update_unix_seconds} must be before nextUpdate {next_update_unix_seconds}",
            ),
            Self::ZeroMaximumWindow => write!(
                formatter,
                "revocation material maximum freshness window must be greater than zero",
            ),
            Self::WindowExceedsMaximum {
                window_seconds,
                maximum_window_seconds,
            } => write!(
                formatter,
                "revocation material window is {window_seconds} seconds, exceeding the local maximum of {maximum_window_seconds} seconds",
            ),
            Self::NotYetValid {
                trusted_time_unix_seconds,
                this_update_unix_seconds,
            } => write!(
                formatter,
                "revocation material is not usable at trusted time {trusted_time_unix_seconds}; thisUpdate is {this_update_unix_seconds}",
            ),
            Self::Expired {
                trusted_time_unix_seconds,
                next_update_unix_seconds,
            } => write!(
                formatter,
                "revocation material is stale at trusted time {trusted_time_unix_seconds}; nextUpdate is {next_update_unix_seconds}",
            ),
        }
    }
}

impl std::error::Error for RevocationMaterialFreshnessError {}
