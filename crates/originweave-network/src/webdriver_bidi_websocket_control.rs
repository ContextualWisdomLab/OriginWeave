use std::{
    io::{self, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use crate::{
    MAX_WEBSOCKET_FRAME_TIMEOUT, WebDriverBiDiWebSocketEstablished,
    WebDriverBiDiWebSocketFrameError, WebDriverBiDiWebSocketMaskKey,
};

const MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES: usize = 125;

fn validate_pong_parameters(
    payload_bytes: usize,
    frame_timeout: Duration,
) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    if frame_timeout.is_zero() || frame_timeout > MAX_WEBSOCKET_FRAME_TIMEOUT {
        return Err(WebDriverBiDiWebSocketFrameError::InvalidFrameTimeout {
            frame_timeout,
            maximum_timeout: MAX_WEBSOCKET_FRAME_TIMEOUT,
        });
    }
    if payload_bytes > MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES {
        return Err(WebDriverBiDiWebSocketFrameError::FrameTooLarge {
            payload_bytes,
            maximum_bytes: MAX_WEBSOCKET_CONTROL_FRAME_PAYLOAD_BYTES,
        });
    }
    Ok(())
}

fn serialize_pong_frame(payload: &[u8], masking_key: WebDriverBiDiWebSocketMaskKey) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 6);
    frame.push(0x8a);
    frame.push(0x80 | payload.len() as u8);
    frame.extend_from_slice(masking_key.as_bytes());
    frame.extend(
        payload.iter().enumerate().map(|(index, byte)| {
            byte ^ masking_key.as_bytes()[index % masking_key.as_bytes().len()]
        }),
    );
    frame
}

trait PongFrameWriter {
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize>;
}

impl PongFrameWriter for TcpStream {
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        TcpStream::set_write_timeout(self, timeout)
    }

    fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write(bytes)
    }
}

fn write_pong_frame_with_clock(
    writer: &mut dyn PongFrameWriter,
    frame: &[u8],
    frame_timeout: Duration,
    now: &mut dyn FnMut() -> Instant,
) -> Result<(), WebDriverBiDiWebSocketFrameError> {
    let deadline = now() + frame_timeout;
    let mut bytes_written = 0;
    while bytes_written < frame.len() {
        let remaining = deadline.saturating_duration_since(now());
        if remaining.is_zero() {
            return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                bytes_written,
                source: io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Pong frame write deadline elapsed",
                ),
            });
        }
        writer
            .set_write_timeout(Some(remaining))
            .map_err(|source| {
                WebDriverBiDiWebSocketFrameError::FrameWriteModeConfigurationFailed {
                    bytes_written,
                    source,
                }
            })?;
        match writer.write_frame_bytes(&frame[bytes_written..]) {
            Ok(0) => {
                return Err(WebDriverBiDiWebSocketFrameError::FrameWriteZero { bytes_written });
            }
            Ok(written) => bytes_written += written,
            Err(source) => {
                if source.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if matches!(
                    source.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) {
                    if deadline.saturating_duration_since(now()).is_zero() {
                        return Err(WebDriverBiDiWebSocketFrameError::FrameWriteTimedOut {
                            bytes_written,
                            source,
                        });
                    }
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                return Err(WebDriverBiDiWebSocketFrameError::FrameWriteFailed {
                    bytes_written,
                    source,
                });
            }
        }
    }
    writer
        .set_write_timeout(None)
        .map_err(|source| WebDriverBiDiWebSocketFrameError::FrameWriteCleanupFailed { source })?;
    Ok(())
}

impl WebDriverBiDiWebSocketEstablished {
    /// Write one final masked RFC 6455 Pong control frame on this verified stream.
    ///
    /// The payload is limited to the RFC 6455 control-frame maximum of 125 bytes. A caller that is
    /// responding to Ping must pass the exact received Ping application data and a fresh,
    /// unpredictable masking key dedicated to this client frame. The operation consumes established
    /// state and returns it only after the complete frame is written within one monotonic bounded
    /// deadline and the operation-local socket timeout is cleared. Failure yields no reusable stream.
    /// This protocol response does not create browser, page, policy, origin, or Agent authority.
    pub fn write_pong_frame(
        mut self,
        payload: &[u8],
        masking_key: WebDriverBiDiWebSocketMaskKey,
        frame_timeout: Duration,
    ) -> Result<Self, WebDriverBiDiWebSocketFrameError> {
        validate_pong_parameters(payload.len(), frame_timeout).and_then(|()| {
            let frame = serialize_pong_frame(payload, masking_key);
            let mut now = Instant::now;
            write_pong_frame_with_clock(&mut self.stream, &frame, frame_timeout, &mut now)
                .map(|()| self)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug)]
    enum WriteAction {
        Count(usize),
        Error(io::ErrorKind),
    }

    #[derive(Debug)]
    struct FakeWriter {
        timeout_error: Option<io::ErrorKind>,
        cleanup_error: Option<io::ErrorKind>,
        actions: VecDeque<WriteAction>,
    }

    impl FakeWriter {
        fn new(actions: impl IntoIterator<Item = WriteAction>) -> Self {
            Self {
                timeout_error: None,
                cleanup_error: None,
                actions: actions.into_iter().collect(),
            }
        }
    }

    impl PongFrameWriter for FakeWriter {
        fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
            let error = if timeout.is_some() {
                self.timeout_error
            } else {
                self.cleanup_error
            };
            error.map_or(Ok(()), |kind| Err(io::Error::from(kind)))
        }

        fn write_frame_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self
                .actions
                .pop_front()
                .unwrap_or(WriteAction::Count(bytes.len()))
            {
                WriteAction::Count(count) => Ok(count.min(bytes.len())),
                WriteAction::Error(kind) => Err(io::Error::from(kind)),
            }
        }
    }

    fn write_with_fake(
        writer: &mut FakeWriter,
        now_values: impl IntoIterator<Item = Instant>,
    ) -> Result<(), WebDriverBiDiWebSocketFrameError> {
        let fallback = Instant::now();
        let mut now_values = now_values.into_iter();
        let mut now = || now_values.next().unwrap_or(fallback);
        write_pong_frame_with_clock(writer, b"abcdef", Duration::from_secs(1), &mut now)
    }

    #[test]
    fn pong_parameter_validation_is_fail_closed() {
        assert!(validate_pong_parameters(0, Duration::from_millis(1)).is_ok());

        let zero_timeout = validate_pong_parameters(0, Duration::ZERO)
            .expect_err("zero timeout must fail closed");
        assert!(format!("{zero_timeout:?}").starts_with("InvalidFrameTimeout"));

        let excessive_timeout = validate_pong_parameters(
            0,
            MAX_WEBSOCKET_FRAME_TIMEOUT + Duration::from_nanos(1),
        )
        .expect_err("timeout above the resource ceiling must fail closed");
        assert!(format!("{excessive_timeout:?}").starts_with("InvalidFrameTimeout"));

        let excessive_payload = validate_pong_parameters(126, Duration::from_millis(1))
            .expect_err("control payload above the RFC 6455 ceiling must fail closed");
        assert!(format!("{excessive_payload:?}").starts_with("FrameTooLarge"));
    }

    #[test]
    fn pong_serializer_emits_final_masked_control_frame() {
        let key = WebDriverBiDiWebSocketMaskKey::new([1, 2, 3, 4]);
        let frame = serialize_pong_frame(b"abc", key);
        assert_eq!(&frame[..6], &[0x8a, 0x83, 1, 2, 3, 4]);
        assert_eq!(&frame[6..], &[b'a' ^ 1, b'b' ^ 2, b'c' ^ 3]);
    }

    #[test]
    fn pong_writer_handles_partial_interrupted_and_would_block_progress() {
        let start = Instant::now();
        let mut partial = FakeWriter::new([WriteAction::Count(2), WriteAction::Count(4)]);
        assert!(write_with_fake(&mut partial, [start, start, start]).is_ok());

        let mut interrupted = FakeWriter::new([
            WriteAction::Error(io::ErrorKind::Interrupted),
            WriteAction::Count(6),
        ]);
        assert!(write_with_fake(&mut interrupted, [start, start, start]).is_ok());

        let mut would_block = FakeWriter::new([
            WriteAction::Error(io::ErrorKind::WouldBlock),
            WriteAction::Count(6),
        ]);
        assert!(write_with_fake(&mut would_block, [start, start, start, start]).is_ok());
    }

    #[test]
    fn pong_writer_preserves_typed_write_failures() {
        let start = Instant::now();
        let later = start + Duration::from_secs(1);

        let mut deadline = FakeWriter::new([]);
        let deadline_error = write_with_fake(&mut deadline, [start, later])
            .expect_err("elapsed deadline must fail closed");
        assert!(format!("{deadline_error:?}").starts_with("FrameWriteTimedOut"));

        let mut configure = FakeWriter::new([]);
        configure.timeout_error = Some(io::ErrorKind::PermissionDenied);
        let configure_error = write_with_fake(&mut configure, [start, start])
            .expect_err("write-timeout configuration failure must be preserved");
        assert!(
            format!("{configure_error:?}").starts_with("FrameWriteModeConfigurationFailed")
        );

        let mut zero = FakeWriter::new([WriteAction::Count(0)]);
        let zero_error = write_with_fake(&mut zero, [start, start])
            .expect_err("zero-byte progress must fail closed");
        assert!(format!("{zero_error:?}").starts_with("FrameWriteZero"));

        let mut timed_out = FakeWriter::new([WriteAction::Error(io::ErrorKind::TimedOut)]);
        let timed_out_error = write_with_fake(&mut timed_out, [start, start, later])
            .expect_err("timed-out write at the deadline must be preserved");
        assert!(format!("{timed_out_error:?}").starts_with("FrameWriteTimedOut"));

        let mut failed = FakeWriter::new([WriteAction::Error(io::ErrorKind::BrokenPipe)]);
        let failed_error = write_with_fake(&mut failed, [start, start])
            .expect_err("non-retryable write failure must be preserved");
        assert!(format!("{failed_error:?}").starts_with("FrameWriteFailed"));

        let mut cleanup = FakeWriter::new([WriteAction::Count(6)]);
        cleanup.cleanup_error = Some(io::ErrorKind::PermissionDenied);
        let cleanup_error = write_with_fake(&mut cleanup, [start, start])
            .expect_err("timeout cleanup failure must be preserved");
        assert!(format!("{cleanup_error:?}").starts_with("FrameWriteCleanupFailed"));
    }
}
