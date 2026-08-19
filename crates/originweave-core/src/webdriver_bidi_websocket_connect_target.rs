//! Explicit no-DNS connection targets for correlated WebDriver BiDi endpoints.
//!
//! This boundary converts only literal loopback listener identities into exact socket metadata.
//! It deliberately refuses `localhost` so a later connector cannot silently inherit ambient DNS
//! authority from an admitted WebDriver endpoint. The resulting value does not open a socket,
//! authenticate a peer, negotiate TLS, perform a WebSocket handshake, or grant Agent authority.

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use crate::CorrelatedWebDriverBiDiWebSocketEndpoint;

/// An exact loopback socket destination derived from one correlated WebDriver BiDi endpoint.
///
/// The destination is inert connection metadata. It proves only that the already-admitted endpoint
/// named a literal loopback IP address, retained an explicit nonzero port, and was correlated to the
/// expected WebDriver session id. A runtime connector must independently enforce peer identity,
/// TLS, WebSocket, process, policy, and browser authority before using transport I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDriverBiDiWebSocketConnectTarget {
    socket_addr: SocketAddr,
    requires_tls: bool,
    session_id: String,
}

impl WebDriverBiDiWebSocketConnectTarget {
    /// Return the exact loopback socket destination without performing name resolution.
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    /// Return whether the admitted endpoint requires a TLS-protected WebSocket transport.
    #[must_use]
    pub const fn requires_tls(&self) -> bool {
        self.requires_tls
    }

    /// Return the exact WebDriver session id established by the preceding correlation boundary.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl CorrelatedWebDriverBiDiWebSocketEndpoint {
    /// Consume this correlated endpoint and derive one exact no-DNS loopback socket destination.
    ///
    /// Literal IPv4 and IPv6 loopback hosts become an exact [`SocketAddr`]. Any admitted host that
    /// is not an IP literal—including `localhost`—fails closed so the caller must perform an
    /// explicit, separately trusted name-resolution step rather than inheriting ambient resolver
    /// authority. This method performs no DNS lookup, socket I/O, peer authentication, TLS, or
    /// WebSocket handshake.
    pub fn into_explicit_connect_target(
        self,
    ) -> Result<WebDriverBiDiWebSocketConnectTarget, WebDriverBiDiWebSocketConnectTargetError> {
        let socket_addr = if let Ok(ipv4) = self.host().parse::<Ipv4Addr>() {
            SocketAddr::from((ipv4, self.port()))
        } else if let Ok(ipv6) = self.host().parse::<Ipv6Addr>() {
            SocketAddr::from((ipv6, self.port()))
        } else {
            return Err(WebDriverBiDiWebSocketConnectTargetError::NameResolutionRequired);
        };

        Ok(WebDriverBiDiWebSocketConnectTarget {
            socket_addr,
            requires_tls: self.is_secure(),
            session_id: self.session_id().to_owned(),
        })
    }
}

/// Fail-closed errors while deriving an explicit WebDriver BiDi socket destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiWebSocketConnectTargetError {
    /// The admitted endpoint used a host name and therefore requires explicit trusted resolution.
    NameResolutionRequired,
}

impl fmt::Display for WebDriverBiDiWebSocketConnectTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameResolutionRequired => formatter.write_str(
                "WebDriver BiDi WebSocket endpoint requires explicit trusted name resolution",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiWebSocketConnectTargetError {}
