use std::{
    error::Error,
    fmt,
    time::{Duration, Instant},
};

use crate::{
    WebDriverBiDiWebSocketEstablished, WebDriverBiDiWebSocketFrameError,
    WebDriverBiDiWebSocketMaskKey, webdriver_bidi_connection::WebDriverBiDiConnectionGeneration,
};

struct ClosureDeadline<'a> {
    expires_at: Instant,
    now: &'a mut dyn FnMut() -> Instant,
}

impl ClosureDeadline<'_> {
    fn remaining(&mut self) -> Result<Duration, WebDriverBiDiWebSocketTransportClosureError> {
        let remaining = self.expires_at.saturating_duration_since((self.now)());
        if remaining.is_zero() {
            return Err(WebDriverBiDiWebSocketTransportClosureError::DeadlineExpired);
        }
        Ok(remaining)
    }
}

/// Bounded transport-closure condition observed on one consumed WebDriver BiDi WebSocket.
///
/// Every successful variant proves clean TCP EOF on the exact established connection. A preceding
/// RFC 6455 Close exchange is retained separately from EOF-only cessation so a consumer cannot
/// confuse entering CLOSING with the transport actually becoming closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebDriverBiDiWebSocketTransportClosureKind {
    /// The peer ended the TCP byte stream cleanly before any new WebSocket frame byte was read.
    PeerEof,
    /// A validated peer Close was answered with a masked client Close before clean TCP EOF.
    PeerCloseThenEof,
}

/// Credential-free observation that one established WebDriver BiDi transport actually closed.
///
/// Construction consumes the established WebSocket, so this value cannot be used to regain the
/// underlying connection. It retains the private process-local generation of that exact connection
/// so a later teardown consumer can reject closure observed on another socket even when session and
/// command identifiers are reused. Peer Close enters CLOSING only: OriginWeave answers it with a
/// caller-keyed masked Close and emits this final observation only after subsequent clean TCP EOF.
/// Pre-Close Ping is answered with an equally caller-keyed masked Pong carrying identical payload.
/// No masking entropy, browser authority, process state, profile cleanup, policy, secret, reconnect,
/// retry, or Agent authority is invented by this transport adapter.
#[derive(Debug, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketTransportClosureObservation {
    kind: WebDriverBiDiWebSocketTransportClosureKind,
    peer_close_status_code: Option<u16>,
    connection_generation: WebDriverBiDiConnectionGeneration,
}

impl WebDriverBiDiWebSocketTransportClosureObservation {
    /// Consume one established connection and prove bounded transport closure.
    ///
    /// The caller supplies independent fresh masking keys for the only pre-Close Ping response and
    /// the required Close response. At most one pre-Close Ping or unsolicited Pong is admitted;
    /// additional control traffic, application data, timeout, partial EOF, malformed frames, write
    /// failures, or any post-Close frame fail closed. Peer Close alone is never success: after the
    /// masked response this boundary requires zero-byte TCP EOF within one operation-wide
    /// deadline covering all reads and writes. Evidence arriving after that deadline is rejected.
    pub fn observe(
        established: WebDriverBiDiWebSocketEstablished,
        pong_masking_key: WebDriverBiDiWebSocketMaskKey,
        close_masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        Self::observe_with_clock(
            established,
            pong_masking_key,
            close_masking_key,
            frame_timeout,
            &mut Instant::now,
        )
    }

    fn observe_with_clock(
        established: WebDriverBiDiWebSocketEstablished,
        pong_masking_key: WebDriverBiDiWebSocketMaskKey,
        close_masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
        now: &mut dyn FnMut() -> Instant,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        crate::webdriver_bidi_websocket_frame::validate_frame_timeout(frame_timeout)
            .map_err(|source| WebDriverBiDiWebSocketTransportClosureError::Frame { source })?;
        let mut deadline = ClosureDeadline {
            expires_at: now() + frame_timeout,
            now,
        };
        let connection_generation = established.transport_evidence().connection_generation();
        Self::observe_pre_close(
            established,
            pong_masking_key,
            close_masking_key,
            &mut deadline,
            true,
            connection_generation,
        )
    }

    fn observe_pre_close(
        established: WebDriverBiDiWebSocketEstablished,
        pong_masking_key: WebDriverBiDiWebSocketMaskKey,
        close_masking_key: WebDriverBiDiWebSocketMaskKey,
        deadline: &mut ClosureDeadline<'_>,
        allow_pre_close_control: bool,
        connection_generation: WebDriverBiDiConnectionGeneration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        match established.read_frame(deadline.remaining()?) {
            Ok((established, frame)) if frame.opcode() == 0xa && allow_pre_close_control => {
                Self::observe_pre_close(
                    established,
                    pong_masking_key,
                    close_masking_key,
                    deadline,
                    false,
                    connection_generation,
                )
            }
            Ok((established, frame)) if frame.opcode() == 0x9 && allow_pre_close_control => {
                let established = established
                    .write_pong_frame(frame.payload(), pong_masking_key, deadline.remaining()?)
                    .map_err(
                        |source| WebDriverBiDiWebSocketTransportClosureError::Frame { source },
                    )?;
                Self::observe_pre_close(
                    established,
                    pong_masking_key,
                    close_masking_key,
                    deadline,
                    false,
                    connection_generation,
                )
            }
            Ok((established, frame)) if frame.opcode() == 0x8 => {
                let peer_close_status_code = frame
                    .payload()
                    .get(..2)
                    .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
                let established = established
                    .write_close_frame(
                        peer_close_status_code,
                        close_masking_key,
                        deadline.remaining()?,
                    )
                    .map_err(
                        |source| WebDriverBiDiWebSocketTransportClosureError::Frame { source },
                    )?;
                Self::observe_eof_after_close(
                    established,
                    deadline,
                    peer_close_status_code,
                    connection_generation,
                )
            }
            Ok((_established, frame)) => Err(
                WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame {
                    opcode: frame.opcode(),
                },
            ),
            Err(WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 }) => {
                deadline.remaining()?;
                Ok(Self {
                    kind: WebDriverBiDiWebSocketTransportClosureKind::PeerEof,
                    peer_close_status_code: None,
                    connection_generation,
                })
            }
            Err(source) => Err(WebDriverBiDiWebSocketTransportClosureError::Frame { source }),
        }
    }

    fn observe_eof_after_close(
        established: WebDriverBiDiWebSocketEstablished,
        deadline: &mut ClosureDeadline<'_>,
        peer_close_status_code: Option<u16>,
        connection_generation: WebDriverBiDiConnectionGeneration,
    ) -> Result<Self, WebDriverBiDiWebSocketTransportClosureError> {
        match established.read_frame(deadline.remaining()?) {
            Err(WebDriverBiDiWebSocketFrameError::FrameEnded { bytes_read: 0 }) => {
                deadline.remaining()?;
                Ok(Self {
                    kind: WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                    peer_close_status_code,
                    connection_generation,
                })
            }
            Ok((_established, frame)) => Err(
                WebDriverBiDiWebSocketTransportClosureError::UnexpectedFrame {
                    opcode: frame.opcode(),
                },
            ),
            Err(source) => Err(WebDriverBiDiWebSocketTransportClosureError::Frame { source }),
        }
    }

    /// Return the exact transport-closure condition that produced this observation.
    #[must_use]
    pub const fn kind(&self) -> WebDriverBiDiWebSocketTransportClosureKind {
        self.kind
    }

    /// Return the validated peer Close status when the completed closing exchange carried one.
    #[must_use]
    pub const fn peer_close_status_code(&self) -> Option<u16> {
        self.peer_close_status_code
    }

    pub(crate) const fn connection_generation(&self) -> WebDriverBiDiConnectionGeneration {
        self.connection_generation
    }
}

/// Fail-closed errors while converting one established BiDi transport into closure evidence.
#[derive(Debug)]
pub enum WebDriverBiDiWebSocketTransportClosureError {
    /// The operation-wide deadline expired before transport-closure evidence was admitted.
    DeadlineExpired,
    /// The peer sent a valid WebSocket frame outside the bounded closing state machine.
    UnexpectedFrame {
        /// Exact validated RFC 6455 opcode observed instead of admissible bounded closing traffic.
        opcode: u8,
    },
    /// The existing bounded WebSocket frame reader or writer failed before closure was proven.
    Frame {
        /// Original typed frame failure retained as the causal source.
        source: WebDriverBiDiWebSocketFrameError,
    },
}

impl fmt::Display for WebDriverBiDiWebSocketTransportClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeadlineExpired => {
                formatter.write_str("WebDriver BiDi transport closure deadline expired")
            }
            Self::UnexpectedFrame { .. } => formatter
                .write_str("WebDriver BiDi peer sent non-closure traffic instead of closing"),
            Self::Frame { .. } => {
                formatter.write_str("WebDriver BiDi transport closure could not be observed safely")
            }
        }
    }
}

impl Error for WebDriverBiDiWebSocketTransportClosureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedFrame { .. } | Self::DeadlineExpired => None,
            Self::Frame { source } => Some(source),
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::{
        WebDriverBiDiTcpConnectionPlan, WebDriverBiDiWebSocketClientKey,
        WebDriverBiDiWebSocketHandshakePlan,
    };
    use originweave_core::WebDriverBiDiWebSocketEndpoint;
    use std::{
        io::{Read, Write},
        net::{Shutdown, TcpListener, TcpStream},
        thread,
    };

    fn established_peer(frames: &[u8]) -> (WebDriverBiDiWebSocketEstablished, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind peer");
        let address = listener.local_addr().expect("peer address");
        let peer = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept client");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("bound peer");
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).expect("opening request");
                request.push(byte[0]);
            }
            stream.write_all(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n").expect("opening response");
            stream
        });
        let session = "01234567-89ab-cdef-0123-456789abcdef";
        let endpoint =
            WebDriverBiDiWebSocketEndpoint::new(&format!("ws://{address}/session/{session}"))
                .expect("endpoint");
        let target = endpoint
            .correlate_session_id(session)
            .expect("session")
            .into_explicit_connect_target()
            .expect("target");
        let connection = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1)
            .expect("plan")
            .connect()
            .expect("connect");
        let key = WebDriverBiDiWebSocketClientKey::new("dGhlIHNhbXBsZSBub25jZQ==").expect("key");
        let established = WebDriverBiDiWebSocketHandshakePlan::new(connection, key)
            .expect("handshake")
            .write_opening_request(Duration::from_secs(1))
            .expect("opening write")
            .read_opening_response(Duration::from_secs(1))
            .expect("opening read");
        let mut peer = peer.join().expect("peer handshake completed");
        peer.write_all(frames).expect("peer frames");
        peer.shutdown(Shutdown::Write).expect("peer write EOF");
        (established, peer)
    }

    #[test]
    fn fixed_clock_accepts_complete_closure_before_deadline() {
        for (frames, kind, status) in [
            (
                &[][..],
                WebDriverBiDiWebSocketTransportClosureKind::PeerEof,
                None,
            ),
            (
                &[0x88, 0][..],
                WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                None,
            ),
            (
                &[0x88, 2, 3, 0xe8][..],
                WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                Some(1000),
            ),
            (
                &[0x89, 0, 0x88, 0][..],
                WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                None,
            ),
            (
                &[0x8a, 0, 0x88, 0][..],
                WebDriverBiDiWebSocketTransportClosureKind::PeerCloseThenEof,
                None,
            ),
        ] {
            let (established, _peer) = established_peer(frames);
            let now = Instant::now();
            let observation =
                WebDriverBiDiWebSocketTransportClosureObservation::observe_with_clock(
                    established,
                    WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
                    WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
                    Duration::from_secs(1),
                    &mut || now,
                )
                .expect("closure before deadline");
            assert_eq!(observation.kind(), kind);
            assert_eq!(observation.peer_close_status_code(), status);
        }
    }

    #[test]
    fn invalid_deadline_is_rejected_before_clock_use() {
        for timeout in [
            Duration::ZERO,
            crate::MAX_WEBSOCKET_FRAME_TIMEOUT + Duration::from_nanos(1),
        ] {
            let (established, _peer) = established_peer(&[]);
            let mut clock_calls = 0;
            let result = WebDriverBiDiWebSocketTransportClosureObservation::observe_with_clock(
                established,
                WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
                WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
                timeout,
                &mut || {
                    clock_calls += 1;
                    Instant::now()
                },
            );
            assert!(matches!(
                result,
                Err(WebDriverBiDiWebSocketTransportClosureError::Frame {
                    source: WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout { .. }
                })
            ));
            assert_eq!(clock_calls, 0);
        }
    }

    #[test]
    fn deadline_expiry_is_checked_at_every_closure_transition() {
        for (frames, expire_tick) in [
            (&[][..], 2),
            (&[0x88, 0][..], 1),
            (&[0x88, 0][..], 2),
            (&[0x88, 0][..], 3),
            (&[0x88, 0][..], 4),
            (&[0x89, 0, 0x88, 0][..], 2),
            (&[0x89, 0, 0x88, 0][..], 3),
            (&[0x89, 0, 0x88, 0][..], 4),
            (&[0x89, 0, 0x88, 0][..], 5),
            (&[0x89, 0, 0x88, 0][..], 6),
            (&[0x8a, 0, 0x88, 0][..], 2),
        ] {
            let (established, _peer) = established_peer(frames);
            let start = Instant::now();
            let mut tick = 0;
            let mut now = || {
                let elapsed = Duration::from_secs(u64::from(tick >= expire_tick));
                tick += 1;
                start + elapsed
            };
            let error = WebDriverBiDiWebSocketTransportClosureObservation::observe_with_clock(
                established,
                WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]),
                WebDriverBiDiWebSocketMaskKey::new([5, 6, 7, 8]),
                Duration::from_secs(1),
                &mut now,
            )
            .expect_err("expired closure must not produce evidence");
            assert!(matches!(
                error,
                WebDriverBiDiWebSocketTransportClosureError::DeadlineExpired
            ));
            assert_eq!(
                error.to_string(),
                "WebDriver BiDi transport closure deadline expired"
            );
            assert!(error.source().is_none());
        }
    }
}
