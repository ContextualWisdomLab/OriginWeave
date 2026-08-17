use std::net::SocketAddr;
use std::time::{Duration, Instant};

use originweave_core::Origin;
use originweave_destination::{DestinationError, FreshResolutionSnapshot};

use crate::connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS, NetworkError,
};

fn effective_origin_port(origin: &Origin) -> u16 {
    let default_port = match origin.scheme() {
        "https" => 443,
        _ => 80,
    };
    let authority = &origin.as_str()[origin.scheme().len() + 3..];
    let explicit_port = if authority.starts_with('[') {
        authority.rsplit_once("]:").map(|(_, port)| port)
    } else {
        authority.rsplit_once(':').map(|(_, port)| port)
    };

    match explicit_port {
        Some(port) => port
            .bytes()
            .fold(0_u16, |value, digit| value * 10 + u16::from(digit - b'0')),
        None => default_port,
    }
}

/// A single-use direct connection plan authorized by a fresh resolution window.
///
/// This adapter composes the destination crate's monotonic freshness authority
/// with the existing exact-socket connection planner. It performs no DNS lookup,
/// wall-clock read, proxy selection, TLS, HTTP, browser control, or persistence.
#[derive(Debug)]
pub struct FreshConnectionPlan {
    connection_plan: ConnectionPlan,
    resolution: FreshResolutionSnapshot,
    socket_address: SocketAddr,
    resolution_approved_at: Duration,
    resolution_valid_until: Duration,
    resolution_authorized_at: Duration,
    authorized_instant: Instant,
}

impl FreshConnectionPlan {
    /// Validate freshness and one exact direct-connection request without I/O.
    pub fn new(
        resolution: &FreshResolutionSnapshot,
        current_time: Duration,
        socket_address: SocketAddr,
        connect_timeout: Duration,
        maximum_attempts: u8,
    ) -> Result<Self, NetworkError> {
        let authorization_started_at = Instant::now();
        Self::new_with_authorization_instant(
            resolution,
            current_time,
            socket_address,
            connect_timeout,
            maximum_attempts,
            authorization_started_at,
        )
    }

    fn new_with_authorization_instant(
        resolution: &FreshResolutionSnapshot,
        current_time: Duration,
        socket_address: SocketAddr,
        connect_timeout: Duration,
        maximum_attempts: u8,
        authorization_started_at: Instant,
    ) -> Result<Self, NetworkError> {
        if socket_address.port() == 0 {
            return Err(NetworkError::InvalidPort);
        }
        if connect_timeout.is_zero() || connect_timeout > MAX_CONNECT_TIMEOUT {
            return Err(NetworkError::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout: MAX_CONNECT_TIMEOUT,
            });
        }
        if maximum_attempts == 0 || maximum_attempts > MAX_CONNECTION_ATTEMPTS {
            return Err(NetworkError::InvalidAttemptCount {
                attempt_count: maximum_attempts,
                maximum_attempts: MAX_CONNECTION_ATTEMPTS,
            });
        }
        let fresh_evidence = resolution
            .authorize_connection(socket_address.ip(), current_time)
            .map_err(|source| NetworkError::DestinationNotApproved {
                socket_address,
                source,
            })?;
        let expected_port = effective_origin_port(resolution.origin());
        if socket_address.port() != expected_port {
            return Err(NetworkError::OriginPortMismatch {
                requested_port: socket_address.port(),
                expected_port,
            });
        }
        let connection_plan = ConnectionPlan::new(
            resolution.resolution_snapshot(),
            socket_address,
            connect_timeout,
            maximum_attempts,
        )?;
        Ok(Self {
            connection_plan,
            resolution: resolution.clone(),
            socket_address,
            resolution_approved_at: fresh_evidence.resolution_approved_at(),
            resolution_valid_until: fresh_evidence.resolution_valid_until(),
            resolution_authorized_at: fresh_evidence.authorized_at(),
            authorized_instant: authorization_started_at,
        })
    }

    /// Return the trusted monotonic time at which resolution was approved.
    #[must_use]
    pub const fn resolution_approved_at(&self) -> Duration {
        self.resolution_approved_at
    }

    /// Return the exclusive end of the resolution authority window.
    #[must_use]
    pub const fn resolution_valid_until(&self) -> Duration {
        self.resolution_valid_until
    }

    /// Return the trusted monotonic time used to authorize this plan.
    #[must_use]
    pub const fn resolution_authorized_at(&self) -> Duration {
        self.resolution_authorized_at
    }

    /// Open the exact approved socket using the elapsed monotonic time since plan authorization.
    ///
    /// This compatibility path anchors a process-local [`Instant`] when the
    /// caller-supplied trusted resolution time is admitted. Actual elapsed time
    /// is added to that authorization value before socket I/O, so callers that
    /// do not supply a second timestamp cannot replay a plan indefinitely after
    /// its freshness window expires. New authority-bearing call sites should use
    /// [`FreshConnectionPlan::connect_at`] with their trusted monotonic clock.
    pub fn connect(self) -> Result<DirectTcpConnection, NetworkError> {
        let current_time = self
            .resolution_authorized_at
            .saturating_add(self.authorized_instant.elapsed());
        self.connect_at(current_time)
    }

    /// Open the exact approved socket only while resolution authority is still fresh.
    ///
    /// `current_time` must come from the same trusted monotonic clock domain used
    /// when this plan was created. Freshness is re-authorized immediately before
    /// socket I/O so a plan cannot be created inside the validity window and then
    /// replayed after that authority expires. A time earlier than the plan's own
    /// authorization checkpoint fails closed. The plan remains single-use because
    /// this method consumes `self`.
    pub fn connect_at(self, current_time: Duration) -> Result<DirectTcpConnection, NetworkError> {
        if current_time < self.resolution_authorized_at {
            return Err(NetworkError::DestinationNotApproved {
                socket_address: self.socket_address,
                source: DestinationError::ResolutionUseBeforeApproval {
                    approved_at: self.resolution_authorized_at,
                    current_time,
                },
            });
        }
        self.resolution
            .authorize_connection(self.socket_address.ip(), current_time)
            .map_err(|source| NetworkError::DestinationNotApproved {
                socket_address: self.socket_address,
                source,
            })?;
        self.connection_plan.connect()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, Instant};

    use originweave_core::Origin;
    use originweave_destination::{
        AddressClass, DestinationError, DestinationPolicy, FreshResolutionSnapshot,
    };

    use super::{FreshConnectionPlan, NetworkError, effective_origin_port};

    #[test]
    fn effective_origin_port_covers_default_and_explicit_authorities() {
        let fixtures = [
            ("http://localhost", 80),
            ("https://example.com", 443),
            ("http://localhost:8080", 8080),
            ("http://[::1]:8443", 8443),
        ];

        for (origin, expected_port) in fixtures {
            let actual_port = Origin::parse(origin).map(|parsed| effective_origin_port(&parsed));
            assert_eq!(actual_port.ok(), Some(expected_port));
        }
    }

    #[test]
    fn compatibility_anchor_includes_time_spent_before_plan_completion() -> Result<(), String> {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9);
        let origin = Origin::parse("http://localhost:9")
            .map_err(|error| format!("loopback origin fixture is invalid: {error:?}"))?;
        let snapshot = FreshResolutionSnapshot::approve(
            origin,
            [IpAddr::V4(Ipv4Addr::LOCALHOST)],
            &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
            Duration::from_secs(10),
            Duration::from_millis(1),
        )
        .map_err(|error| format!("short-lived snapshot is invalid: {error}"))?;
        let authorization_started_at = Instant::now();
        std::thread::sleep(Duration::from_millis(5));

        let plan = FreshConnectionPlan::new_with_authorization_instant(
            &snapshot,
            Duration::from_secs(10),
            socket,
            Duration::from_secs(1),
            1,
            authorization_started_at,
        )
        .map_err(|error| format!("authorize short-lived connection plan: {error}"))?;

        let result = plan.connect();
        assert!(matches!(
            result,
            Err(NetworkError::DestinationNotApproved {
                source: DestinationError::ResolutionApprovalExpired { .. },
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn origin_port_mismatch_error_is_deterministic_and_source_free() {
        let error = NetworkError::OriginPortMismatch {
            requested_port: 8080,
            expected_port: 80,
        };

        assert_eq!(
            error.to_string(),
            "socket port 8080 does not match canonical origin port 80"
        );
        assert!(error.source().is_none());
        assert_eq!(error.attempt_count(), None);
    }
}
