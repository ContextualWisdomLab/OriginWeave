//! Explicit no-DNS connection targets for correlated WebDriver BiDi endpoints.
//!
//! This boundary converts only literal loopback listener identities into exact socket metadata.
//! It deliberately refuses `localhost` so a later connector cannot silently inherit ambient DNS
//! authority from an admitted WebDriver endpoint. When explicit trusted name resolution is needed,
//! the typed error preserves the correlated endpoint instead of discarding its session evidence.
//! A separately observed connected peer must also match the approved socket destination exactly
//! before it becomes verified transport metadata. These values do not open a socket, authenticate
//! a process, negotiate TLS, perform a WebSocket handshake, or grant Agent authority.

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};

use crate::CorrelatedWebDriverBiDiWebSocketEndpoint;

/// An exact loopback socket destination derived from one correlated WebDriver BiDi endpoint.
///
/// The destination is inert connection metadata. It proves only that the already-admitted endpoint
/// named a literal loopback IP address, retained an explicit nonzero port, and was correlated to the
/// expected WebDriver session id. A runtime connector must independently establish a connection and
/// verify its observed peer before treating that transport as the approved destination. TLS,
/// WebSocket, process, policy, and browser authority remain separate boundaries.
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

    /// Consume this approved destination and verify one observed connected socket peer exactly.
    ///
    /// Matching requires the complete [`SocketAddr`]—IP address and port—to equal the approved
    /// no-DNS destination. A mismatch consumes the target and fails closed, preventing a connector
    /// from accidentally reusing the same authority after observing a different peer. Success
    /// produces inert verified-peer metadata only; it does not authenticate an OS process,
    /// negotiate TLS, perform a WebSocket handshake, or grant browser/Agent authority.
    pub fn verify_connected_peer(
        self,
        observed_peer: SocketAddr,
    ) -> Result<VerifiedWebDriverBiDiSocketPeer, WebDriverBiDiSocketPeerVerificationError> {
        let expected = self.socket_addr;
        if observed_peer != expected {
            return Err(WebDriverBiDiSocketPeerVerificationError::PeerMismatch {
                expected,
                actual: observed_peer,
            });
        }

        Ok(VerifiedWebDriverBiDiSocketPeer {
            connect_target: self,
        })
    }
}

/// Inert metadata proving that a connected peer exactly matched the approved BiDi destination.
///
/// This value carries only the destination, TLS requirement, and correlated WebDriver session id
/// already established by preceding boundaries. It does not prove process identity, TLS peer
/// identity, WebSocket protocol state, browser authenticity, policy authorization, or Agent action
/// authority.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedWebDriverBiDiSocketPeer {
    connect_target: WebDriverBiDiWebSocketConnectTarget,
}

impl VerifiedWebDriverBiDiSocketPeer {
    /// Return the exact approved and observed socket peer address.
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        self.connect_target.socket_addr()
    }

    /// Return whether the correlated endpoint still requires TLS before WebSocket use.
    #[must_use]
    pub const fn requires_tls(&self) -> bool {
        self.connect_target.requires_tls()
    }

    /// Return the exact correlated WebDriver session id.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.connect_target.session_id()
    }
}

/// Fail-closed errors while verifying an observed BiDi socket peer.
#[derive(Debug, PartialEq, Eq)]
pub enum WebDriverBiDiSocketPeerVerificationError {
    /// The connected peer differed from the exact destination approved before connection.
    PeerMismatch {
        /// Exact socket address that the connector was authorized to reach.
        expected: SocketAddr,
        /// Socket peer address observed after connection.
        actual: SocketAddr,
    },
}

impl fmt::Display for WebDriverBiDiSocketPeerVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PeerMismatch { .. } => formatter.write_str(
                "connected WebDriver BiDi socket peer does not match the approved destination",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiSocketPeerVerificationError {}

impl CorrelatedWebDriverBiDiWebSocketEndpoint {
    /// Consume this correlated endpoint and derive one exact no-DNS loopback socket destination.
    ///
    /// Literal IPv4 and IPv6 loopback hosts become an exact [`SocketAddr`]. Any admitted host that
    /// is not an IP literal—including `localhost`—fails closed so the caller must perform an
    /// explicit, separately trusted name-resolution step rather than inheriting ambient resolver
    /// authority. The name-resolution-required error retains this correlated endpoint so that
    /// trusted resolver handoff does not require reconstructing or recorrelation of session evidence.
    /// This method performs no DNS lookup, socket I/O, peer authentication, TLS, or WebSocket
    /// handshake.
    pub fn into_explicit_connect_target(
        self,
    ) -> Result<WebDriverBiDiWebSocketConnectTarget, WebDriverBiDiWebSocketConnectTargetError> {
        let socket_addr = if let Ok(ipv4) = self.host().parse::<Ipv4Addr>() {
            SocketAddr::from((ipv4, self.port()))
        } else if let Ok(ipv6) = self.host().parse::<Ipv6Addr>() {
            SocketAddr::from((ipv6, self.port()))
        } else {
            return Err(
                WebDriverBiDiWebSocketConnectTargetError::NameResolutionRequired {
                    correlated_endpoint: self,
                },
            );
        };

        Ok(WebDriverBiDiWebSocketConnectTarget {
            socket_addr,
            requires_tls: self.is_secure(),
            session_id: self.session_id().to_owned(),
        })
    }
}

/// Fail-closed errors while deriving an explicit WebDriver BiDi socket destination.
#[derive(Debug, PartialEq, Eq)]
pub enum WebDriverBiDiWebSocketConnectTargetError {
    /// The admitted endpoint used a host name and therefore requires explicit trusted resolution.
    NameResolutionRequired {
        /// The still-correlated endpoint that must be handed to a separately trusted resolver.
        correlated_endpoint: CorrelatedWebDriverBiDiWebSocketEndpoint,
    },
}

impl WebDriverBiDiWebSocketConnectTargetError {
    /// Borrow the correlated endpoint preserved for an explicit trusted resolver handoff.
    #[must_use]
    pub const fn correlated_endpoint(&self) -> &CorrelatedWebDriverBiDiWebSocketEndpoint {
        match self {
            Self::NameResolutionRequired {
                correlated_endpoint,
            } => correlated_endpoint,
        }
    }

    /// Recover the correlated endpoint for an explicit trusted resolver handoff.
    #[must_use]
    pub fn into_correlated_endpoint(self) -> CorrelatedWebDriverBiDiWebSocketEndpoint {
        match self {
            Self::NameResolutionRequired {
                correlated_endpoint,
            } => correlated_endpoint,
        }
    }
}

impl fmt::Display for WebDriverBiDiWebSocketConnectTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameResolutionRequired { .. } => formatter.write_str(
                "WebDriver BiDi WebSocket endpoint requires explicit trusted name resolution",
            ),
        }
    }
}

impl std::error::Error for WebDriverBiDiWebSocketConnectTargetError {}
