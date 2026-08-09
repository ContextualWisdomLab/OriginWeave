use std::collections::BTreeSet;
use std::time::Duration;

use rustls::pki_types::UnixTime;

use crate::TlsError;

/// The largest accepted total TLS handshake duration.
pub const MAX_TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest caller-configurable minimum remaining leaf-certificate validity.
///
/// Seven days is a product safety bound for delegated-task scheduling, not a
/// PKIX validity rule. Longer-lived work must obtain fresh transport authority
/// rather than treating one authenticated connection as durable authority.
pub const MAX_MINIMUM_LEAF_VALIDITY: Duration = Duration::from_secs(604_800);

/// The largest number of ALPN identifiers accepted in one policy.
pub const MAX_ALPN_PROTOCOL_COUNT: usize = 8;

/// The largest encoded byte length accepted for one ALPN identifier.
pub const MAX_ALPN_PROTOCOL_LENGTH: usize = 255;

/// The largest total ALPN identifier bytes accepted in one policy.
pub const MAX_ALPN_TOTAL_BYTES: usize = 1_024;

/// The largest number of server-presented certificates accepted for evidence.
pub const MAX_SERVER_CERTIFICATE_COUNT: usize = 16;

/// The largest total server-presented certificate DER bytes accepted for evidence.
pub const MAX_SERVER_CERTIFICATE_BYTES: usize = 1_048_576;

/// Whether successful TLS authentication requires a negotiated ALPN value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlpnRequirement {
    /// Record an explicit absence without assuming an application protocol.
    Optional,
    /// Reject a completed TLS handshake that negotiated no ALPN value.
    Required,
}

/// Fixed trusted time and bounded TLS protocol policy for one handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsClientPolicy {
    trusted_time: UnixTime,
    handshake_timeout: Duration,
    alpn_protocols: Vec<Vec<u8>>,
    alpn_requirement: AlpnRequirement,
    minimum_leaf_validity: Duration,
}

impl TlsClientPolicy {
    /// Validate a fixed-time, deadline-bound, explicit-ALPN TLS client policy.
    ///
    /// The minimum remaining leaf-certificate validity defaults to zero for
    /// compatibility. Delegated-task callers can opt into a nonzero product
    /// safety horizon with [`Self::with_minimum_leaf_validity`].
    pub fn new(
        trusted_time: UnixTime,
        handshake_timeout: Duration,
        alpn_protocols: Vec<Vec<u8>>,
        alpn_requirement: AlpnRequirement,
    ) -> Result<Self, TlsError> {
        let invalid_timeout =
            handshake_timeout.is_zero() | (handshake_timeout > MAX_TLS_HANDSHAKE_TIMEOUT);
        if invalid_timeout {
            return Err(TlsError::InvalidHandshakeTimeout {
                timeout: handshake_timeout,
                maximum_timeout: MAX_TLS_HANDSHAKE_TIMEOUT,
            });
        }
        let protocol_count = alpn_protocols.len();
        let invalid_protocol_count = (protocol_count > MAX_ALPN_PROTOCOL_COUNT)
            | (alpn_protocols.is_empty() & (alpn_requirement == AlpnRequirement::Required));
        if invalid_protocol_count {
            return Err(TlsError::InvalidAlpnCount {
                protocol_count,
                maximum_count: MAX_ALPN_PROTOCOL_COUNT,
            });
        }

        let mut seen = BTreeSet::new();
        let mut total_bytes = 0_usize;
        for (protocol_index, protocol) in alpn_protocols.iter().enumerate() {
            let invalid_identifier =
                protocol.is_empty() | (protocol.len() > MAX_ALPN_PROTOCOL_LENGTH);
            if invalid_identifier {
                return Err(TlsError::InvalidAlpnIdentifier {
                    protocol_index,
                    protocol_length: protocol.len(),
                    maximum_length: MAX_ALPN_PROTOCOL_LENGTH,
                });
            }
            total_bytes += protocol.len();
            if total_bytes > MAX_ALPN_TOTAL_BYTES {
                return Err(TlsError::InvalidAlpnBytes {
                    byte_count: total_bytes,
                    maximum_bytes: MAX_ALPN_TOTAL_BYTES,
                });
            }
            if !seen.insert(protocol.clone()) {
                return Err(TlsError::DuplicateAlpnIdentifier { protocol_index });
            }
        }

        Ok(Self {
            trusted_time,
            handshake_timeout,
            alpn_protocols,
            alpn_requirement,
            minimum_leaf_validity: Duration::ZERO,
        })
    }

    /// Configure the minimum remaining leaf-certificate validity required
    /// before an authenticated stream may be exposed to delegated work.
    ///
    /// Zero preserves point-in-time WebPKI behavior. Nonzero horizons are
    /// bounded by [`MAX_MINIMUM_LEAF_VALIDITY`] so a caller cannot turn one TLS
    /// authentication into indefinite task authority.
    pub fn with_minimum_leaf_validity(
        mut self,
        minimum_validity: Duration,
    ) -> Result<Self, TlsError> {
        if minimum_validity > MAX_MINIMUM_LEAF_VALIDITY {
            return Err(TlsError::InvalidMinimumLeafValidity {
                minimum_validity,
                maximum_validity: MAX_MINIMUM_LEAF_VALIDITY,
            });
        }
        self.minimum_leaf_validity = minimum_validity;
        Ok(self)
    }

    /// Return the fixed certificate-validation time.
    #[must_use]
    pub const fn trusted_time(&self) -> UnixTime {
        self.trusted_time
    }

    /// Return the total monotonic TLS handshake timeout.
    #[must_use]
    pub const fn handshake_timeout(&self) -> Duration {
        self.handshake_timeout
    }

    /// Return the ordered ALPN allow-list.
    #[must_use]
    pub fn alpn_protocols(&self) -> Vec<&[u8]> {
        self.alpn_protocols.iter().map(Vec::as_slice).collect()
    }

    /// Return whether ALPN negotiation is mandatory.
    #[must_use]
    pub const fn alpn_requirement(&self) -> AlpnRequirement {
        self.alpn_requirement
    }

    /// Return the configured delegated-task leaf validity horizon.
    #[must_use]
    pub const fn minimum_leaf_validity(&self) -> Duration {
        self.minimum_leaf_validity
    }

    pub(crate) fn into_parts(
        self,
    ) -> (UnixTime, Duration, Vec<Vec<u8>>, AlpnRequirement, Duration) {
        (
            self.trusted_time,
            self.handshake_timeout,
            self.alpn_protocols,
            self.alpn_requirement,
            self.minimum_leaf_validity,
        )
    }
}
