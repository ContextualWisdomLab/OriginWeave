#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    NativeMessagingFrameDirection, NativeMessagingFrameError, decode_native_messaging_frame,
    encode_native_messaging_frame, native_messaging_payload_limit,
};

const HOST_TO_BROWSER_LIMIT: usize = 1_048_576;
const BROWSER_TO_HOST_LIMIT: usize = 67_108_864;

#[test]
fn native_messaging_payload_limits_are_direction_specific() {
    assert_eq!(
        native_messaging_payload_limit(NativeMessagingFrameDirection::HostToBrowser),
        HOST_TO_BROWSER_LIMIT
    );
    assert_eq!(
        native_messaging_payload_limit(NativeMessagingFrameDirection::BrowserToHost),
        BROWSER_TO_HOST_LIMIT
    );
}

#[test]
fn native_messaging_frame_round_trip_uses_native_u32_byte_length() {
    let payload = b"{}";

    for direction in [
        NativeMessagingFrameDirection::HostToBrowser,
        NativeMessagingFrameDirection::BrowserToHost,
    ] {
        let frame = encode_native_messaging_frame(direction, payload)
            .expect("small native messaging payload must be frameable");
        assert_eq!(&frame[..4], &2_u32.to_ne_bytes());
        assert_eq!(
            decode_native_messaging_frame(direction, &frame)
                .expect("freshly encoded native messaging frame must decode"),
            payload
        );
    }
}

#[test]
fn native_messaging_encoder_rejects_oversized_host_payload_before_framing() {
    let oversized = vec![b'x'; HOST_TO_BROWSER_LIMIT + 1];

    assert_eq!(
        encode_native_messaging_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &oversized,
        ),
        Err(NativeMessagingFrameError::PayloadTooLarge)
    );
}

#[test]
fn native_messaging_decoder_rejects_missing_oversized_and_mismatched_lengths() {
    assert_eq!(
        decode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &[0, 0, 0]),
        Err(NativeMessagingFrameError::MissingLengthPrefix)
    );

    let host_to_browser_oversized = 1_048_577_u32.to_ne_bytes();
    assert_eq!(
        decode_native_messaging_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &host_to_browser_oversized,
        ),
        Err(NativeMessagingFrameError::PayloadTooLarge)
    );

    let browser_to_host_oversized = 67_108_865_u32.to_ne_bytes();
    assert_eq!(
        decode_native_messaging_frame(
            NativeMessagingFrameDirection::BrowserToHost,
            &browser_to_host_oversized,
        ),
        Err(NativeMessagingFrameError::PayloadTooLarge)
    );

    let mut short_frame = Vec::from(4_u32.to_ne_bytes());
    short_frame.extend_from_slice(b"abc");
    assert_eq!(
        decode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &short_frame),
        Err(NativeMessagingFrameError::LengthMismatch)
    );

    let mut trailing_frame = Vec::from(2_u32.to_ne_bytes());
    trailing_frame.extend_from_slice(b"abc");
    assert_eq!(
        decode_native_messaging_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &trailing_frame,
        ),
        Err(NativeMessagingFrameError::LengthMismatch)
    );
}

#[test]
fn native_messaging_frame_errors_are_stable_and_source_free() {
    for (error, message) in [
        (
            NativeMessagingFrameError::MissingLengthPrefix,
            "native messaging frame is missing its 32-bit length prefix",
        ),
        (
            NativeMessagingFrameError::PayloadTooLarge,
            "native messaging payload exceeds the direction-specific limit",
        ),
        (
            NativeMessagingFrameError::LengthMismatch,
            "native messaging frame length does not match its prefix",
        ),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(Error::source(&error).is_none());
    }
}
