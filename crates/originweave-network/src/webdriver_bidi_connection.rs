use std::{
    io,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use originweave_core::{VerifiedWebDriverBiDiSocketPeer, WebDriverBiDiWebSocketConnectTarget};

use crate::connection::{MAX_CONNECT_TIMEOUT, MAX_CONNECTION_ATTEMPTS};

mod error;

pub use error::WebDriverBiDiTcpConnectionError;

#[cfg(test)]
mod tests;

fn is_retryable_connect_error(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::Interrupted
    )
}

/// Single-use authority to open one exact WebDriver BiDi loopback TCP destination.
///
/// The plan consumes a session-correlated, no-DNS [`WebDriverBiDiWebSocketConnectTarget`]
/// produced by `originweave-core`. It applies the same bounded per-attempt timeout and retry
/// ceilings as the general direct-network connector, opens only the exact [`SocketAddr`] carried by
/// that target, and does not expose the stream until the operating system's observed peer has been
/// verified by the consumed target.
///
/// This boundary performs no DNS lookup, proxy or PAC routing, Chromium/ChromeDriver process
/// authentication, TLS negotiation, WebSocket upgrade, BiDi framing, browser policy decision, or
/// Agent-authority grant.
#[derive(Debug)]
pub struct WebDriverBiDiTcpConnectionPlan {
    target: WebDriverBiDiWebSocketConnectTarget,
    connect_timeout: Duration,
    maximum_attempts: u8,
}

impl WebDriverBiDiTcpConnectionPlan {
    /// Validate one bounded exact-loopback connection plan without performing network I/O.
    pub fn new(
        target: WebDriverBiDiWebSocketConnectTarget,
        connect_timeout: Duration,
        maximum_attempts: u8,
    ) -> Result<Self, WebDriverBiDiTcpConnectionError> {
        if connect_timeout.is_zero() || connect_timeout > MAX_CONNECT_TIMEOUT {
            return Err(WebDriverBiDiTcpConnectionError::InvalidConnectTimeout {
                connect_timeout,
                maximum_timeout: MAX_CONNECT_TIMEOUT,
            });
        }
        if maximum_attempts == 0 || maximum_attempts > MAX_CONNECTION_ATTEMPTS {
            return Err(WebDriverBiDiTcpConnectionError::InvalidAttemptCount {
                attempt_count: maximum_attempts,
                maximum_attempts: MAX_CONNECTION_ATTEMPTS,
            });
        }

        Ok(Self {
            target,
            connect_timeout,
            maximum_attempts,
        })
    }

    /// Open the exact approved loopback socket and expose it only after peer verification.
    ///
    /// Retry is limited to transport errors that can occur transiently while a local browser driver
    /// listener is becoming ready. Peer-inspection and peer-mismatch failures are integrity failures
    /// and therefore fail closed without retry or fallback.
    pub fn connect(self) -> Result<WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError> {
        self.connect_with(&SystemWebDriverBiDiConnector)
    }

    fn connect_with(
        self,
        connector: &dyn WebDriverBiDiSocketConnector,
    ) -> Result<WebDriverBiDiTcpConnection, WebDriverBiDiTcpConnectionError> {
        let socket_address = self.target.socket_addr();
        let connect_timeout = self.connect_timeout;
        let maximum_attempts = self.maximum_attempts;
        let target = self.target;
        let mut attempt_number = 1;

        loop {
            match connector.connect_timeout(&socket_address, connect_timeout) {
                Ok(stream) => {
                    let observed_peer = connector.peer_addr(&stream).map_err(|source| {
                        WebDriverBiDiTcpConnectionError::PeerInspectionFailed {
                            socket_address,
                            attempt_number,
                            source,
                        }
                    })?;
                    let verified_peer =
                        target
                            .verify_connected_peer(observed_peer)
                            .map_err(|source| WebDriverBiDiTcpConnectionError::PeerMismatch {
                                attempt_number,
                                source,
                            })?;
                    return Ok(WebDriverBiDiTcpConnection {
                        stream,
                        verified_peer,
                        attempt_number,
                        connect_timeout,
                    });
                }
                Err(source)
                    if is_retryable_connect_error(source.kind())
                        && attempt_number < maximum_attempts =>
                {
                    attempt_number += 1;
                }
                Err(source) => {
                    if source.kind() == io::ErrorKind::TimedOut {
                        return Err(WebDriverBiDiTcpConnectionError::ConnectionTimedOut {
                            socket_address,
                            attempt_count: attempt_number,
                            connect_timeout,
                            source,
                        });
                    }
                    return Err(WebDriverBiDiTcpConnectionError::ConnectionFailed {
                        socket_address,
                        attempt_count: attempt_number,
                        source,
                    });
                }
            }
        }
    }
}

trait WebDriverBiDiSocketConnector {
    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<TcpStream>;

    fn peer_addr(&self, stream: &TcpStream) -> io::Result<SocketAddr>;
}

struct SystemWebDriverBiDiConnector;

impl WebDriverBiDiSocketConnector for SystemWebDriverBiDiConnector {
    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<TcpStream> {
        TcpStream::connect_timeout(socket_address, timeout)
    }

    fn peer_addr(&self, stream: &TcpStream) -> io::Result<SocketAddr> {
        stream.peer_addr()
    }
}

/// Established WebDriver BiDi TCP stream whose observed peer matched the approved target exactly.
///
/// This wrapper proves only exact transport-destination equality for one bounded connection. The
/// caller must still establish any required TLS channel, complete a WebSocket handshake, bind the
/// transport to the expected browser process/session, and pass separate action-policy checks.
#[derive(Debug)]
pub struct WebDriverBiDiTcpConnection {
    stream: TcpStream,
    verified_peer: VerifiedWebDriverBiDiSocketPeer,
    attempt_number: u8,
    connect_timeout: Duration,
}

impl WebDriverBiDiTcpConnection {
    /// Borrow the verified TCP stream.
    #[must_use]
    pub const fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Borrow the session-correlated exact peer evidence consumed by this connection.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        &self.verified_peer
    }

    /// Return the one-based bounded attempt on which the connection succeeded.
    #[must_use]
    pub const fn attempt_number(&self) -> u8 {
        self.attempt_number
    }

    /// Return the per-attempt timeout applied while establishing this connection.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// Consume the wrapper into the original verified stream and credential-free transport evidence.
    ///
    /// This handoff does not clone the socket or create reusable connection authority. The returned
    /// evidence records only the already-verified peer plus bounded connection-attempt metadata; it
    /// does not authenticate a browser process, establish TLS, complete WebSocket framing, or grant
    /// browser or Agent authority.
    #[must_use]
    pub fn into_parts(self) -> (TcpStream, WebDriverBiDiTcpConnectionEvidence) {
        let evidence = WebDriverBiDiTcpConnectionEvidence {
            verified_peer: self.verified_peer,
            attempt_number: self.attempt_number,
            connect_timeout: self.connect_timeout,
        };
        (self.stream, evidence)
    }
}

/// Credential-free evidence retained when a verified WebDriver BiDi TCP stream is consumed.
///
/// This value records exact peer/session/TLS-requirement metadata inherited from the consumed
/// no-DNS target together with the successful bounded attempt and per-attempt timeout. It is
/// transport evidence only and grants no process, TLS, WebSocket, browser-action, or Agent authority.
#[derive(Debug)]
pub struct WebDriverBiDiTcpConnectionEvidence {
    verified_peer: VerifiedWebDriverBiDiSocketPeer,
    attempt_number: u8,
    connect_timeout: Duration,
}

impl WebDriverBiDiTcpConnectionEvidence {
    /// Borrow the exact session-correlated peer verified before stream exposure.
    #[must_use]
    pub const fn verified_peer(&self) -> &VerifiedWebDriverBiDiSocketPeer {
        &self.verified_peer
    }

    /// Return the one-based bounded attempt on which the connection succeeded.
    #[must_use]
    pub const fn attempt_number(&self) -> u8 {
        self.attempt_number
    }

    /// Return the per-attempt timeout applied while establishing the connection.
    #[must_use]
    pub const fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
}
