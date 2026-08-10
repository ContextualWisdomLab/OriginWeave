use std::net::SocketAddr;
use std::time::Duration;

use originweave_destination::FreshResolutionSnapshot;

use crate::connection::{ConnectionPlan, DirectTcpConnection, NetworkError};

/// A single-use direct connection plan authorized by a fresh resolution window.
///
/// This adapter composes the destination crate's monotonic freshness authority
/// with the existing exact-socket connection planner. It performs no DNS lookup,
/// wall-clock read, proxy selection, TLS, HTTP, browser control, or persistence.
#[derive(Debug)]
pub struct FreshConnectionPlan {
    connection_plan: ConnectionPlan,
    resolution_approved_at: Duration,
    resolution_valid_until: Duration,
    resolution_authorized_at: Duration,
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
        let connection_plan = ConnectionPlan::new(
            resolution.resolution_snapshot(),
            socket_address,
            connect_timeout,
            maximum_attempts,
        )?;
        Ok(Self {
            connection_plan,
            resolution_approved_at: fresh_evidence.resolution_approved_at(),
            resolution_valid_until: fresh_evidence.resolution_valid_until(),
            resolution_authorized_at: fresh_evidence.authorized_at(),
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

    /// Open the exact approved socket and expose it only after peer verification.
    pub fn connect(self) -> Result<DirectTcpConnection, NetworkError> {
        self.connection_plan.connect()
    }
}
