use std::net::SocketAddr;
use std::time::{Duration, Instant};

use originweave_core::Origin;
use originweave_destination::{DestinationError, FreshResolutionSnapshot};

use crate::connection::{ConnectionPlan, DirectTcpConnection, NetworkError};

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
        let fresh_evidence = resolution
            .authorize_connection(socket_address.ip(), current_time)
            .map_err(|source| NetworkError::DestinationNotApproved {
                socket_address,
                source,
            })?;
        if socket_address.port() != effective_origin_port(resolution.origin()) {
            return Err(NetworkError::InvalidPort);
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
            authorized_instant: Instant::now(),
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
    use originweave_core::Origin;

    use super::effective_origin_port;

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
            assert!(matches!(actual_port, Ok(port) if port == expected_port));
        }
    }
}
