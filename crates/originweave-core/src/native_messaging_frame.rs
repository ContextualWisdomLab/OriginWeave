//! Bounded Chrome native-messaging frame encoding and decoding.
//!
//! Chrome native messaging prefixes each UTF-8 JSON message with a 32-bit
//! native-endian byte length. This module validates only that framing boundary;
//! JSON parsing, host registration, process launch, authority, and secret handling
//! remain separate reviewed layers.

use std::fmt;

const NATIVE_MESSAGING_HOST_TO_CHROME_MAX_BYTES: usize = 1_048_576;
const NATIVE_MESSAGING_CHROME_TO_HOST_MAX_BYTES: usize = 67_108_864;
const NATIVE_MESSAGING_LENGTH_BYTES: usize = 4;

/// Direction of one Chrome native-messaging payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingFrameDirection {
    /// A message written by Chrome and read by the native host.
    ChromeToHost,
    /// A message written by the native host and read by Chrome.
    HostToChrome,
}

impl NativeMessagingFrameDirection {
    const fn maximum_payload_bytes(self) -> usize {
        match self {
            Self::ChromeToHost => NATIVE_MESSAGING_CHROME_TO_HOST_MAX_BYTES,
            Self::HostToChrome => NATIVE_MESSAGING_HOST_TO_CHROME_MAX_BYTES,
        }
    }
}

/// A fail-closed native-messaging frame validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeMessagingFrameError {
    /// Fewer than four bytes were available for the native-endian length header.
    TruncatedHeader,
    /// The declared or outbound payload exceeds Chrome's limit for this direction.
    PayloadTooLarge {
        /// Number of payload bytes declared or supplied.
        declared_bytes: usize,
        /// Maximum payload bytes Chrome permits for this direction.
        maximum_bytes: usize,
    },
    /// The frame body length does not exactly match the declared byte length.
    LengthMismatch {
        /// Number of payload bytes declared by the frame header.
        declared_bytes: usize,
        /// Number of payload bytes actually present after the header.
        actual_bytes: usize,
    },
    /// The frame body is not valid UTF-8 and therefore cannot be a JSON text message.
    InvalidUtf8,
}

impl fmt::Display for NativeMessagingFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader => {
                formatter.write_str("native-messaging frame is missing its four-byte header")
            }
            Self::PayloadTooLarge { .. } => {
                formatter.write_str("native-messaging payload exceeds the direction limit")
            }
            Self::LengthMismatch { .. } => {
                formatter.write_str("native-messaging frame length does not match its header")
            }
            Self::InvalidUtf8 => formatter.write_str("native-messaging payload is not valid UTF-8"),
        }
    }
}

impl std::error::Error for NativeMessagingFrameError {}

/// Encode one already-serialized UTF-8 JSON message as a bounded Chrome native frame.
///
/// This function does not validate JSON syntax. Callers must serialize a reviewed
/// message schema before this framing step and must keep authority decisions outside
/// the payload codec.
pub fn encode_native_messaging_frame(
    payload: &str,
    direction: NativeMessagingFrameDirection,
) -> Result<Vec<u8>, NativeMessagingFrameError> {
    let payload_bytes = payload.as_bytes();
    let maximum_bytes = direction.maximum_payload_bytes();
    if payload_bytes.len() > maximum_bytes {
        return Err(NativeMessagingFrameError::PayloadTooLarge {
            declared_bytes: payload_bytes.len(),
            maximum_bytes,
        });
    }

    // Both reviewed Chrome direction limits are far below u32::MAX, so the
    // preceding bound proves this conversion cannot truncate.
    let declared_length = payload_bytes.len() as u32;
    let mut frame = Vec::with_capacity(NATIVE_MESSAGING_LENGTH_BYTES + payload_bytes.len());
    frame.extend_from_slice(&declared_length.to_ne_bytes());
    frame.extend_from_slice(payload_bytes);
    Ok(frame)
}

/// Decode one complete bounded Chrome native-messaging frame as UTF-8 text.
///
/// Declared size is checked before the body is interpreted, and the complete input
/// must contain exactly one frame. The returned text borrows the caller-owned frame.
pub fn decode_native_messaging_frame(
    frame: &[u8],
    direction: NativeMessagingFrameDirection,
) -> Result<&str, NativeMessagingFrameError> {
    if frame.len() < NATIVE_MESSAGING_LENGTH_BYTES {
        return Err(NativeMessagingFrameError::TruncatedHeader);
    }

    let header = [frame[0], frame[1], frame[2], frame[3]];
    let declared_bytes = u32::from_ne_bytes(header) as usize;
    let maximum_bytes = direction.maximum_payload_bytes();
    if declared_bytes > maximum_bytes {
        return Err(NativeMessagingFrameError::PayloadTooLarge {
            declared_bytes,
            maximum_bytes,
        });
    }

    let payload = &frame[NATIVE_MESSAGING_LENGTH_BYTES..];
    if payload.len() != declared_bytes {
        return Err(NativeMessagingFrameError::LengthMismatch {
            declared_bytes,
            actual_bytes: payload.len(),
        });
    }

    std::str::from_utf8(payload).map_err(|_| NativeMessagingFrameError::InvalidUtf8)
}
