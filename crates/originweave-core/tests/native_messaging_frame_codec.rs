#![allow(clippy::expect_used)]

use originweave_core::{
    NativeMessagingFrameDirection, NativeMessagingFrameError, decode_native_messaging_frame,
    encode_native_messaging_frame,
};

fn frame_with_declared_length(declared_length: u32, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&declared_length.to_ne_bytes());
    frame.extend_from_slice(payload);
    frame
}

#[test]
fn native_messaging_frames_use_native_endian_byte_length_and_round_trip_utf8() {
    let payload = r#"{"message":"안녕"}"#;
    let encoded = encode_native_messaging_frame(
        payload,
        NativeMessagingFrameDirection::ChromeToHost,
    )
    .expect("bounded payload should encode");

    let declared = u32::from_ne_bytes(encoded[..4].try_into().expect("four-byte header"));
    assert_eq!(declared as usize, payload.len());
    assert_eq!(&encoded[4..], payload.as_bytes());
    assert_eq!(
        decode_native_messaging_frame(
            &encoded,
            NativeMessagingFrameDirection::ChromeToHost,
        )
        .expect("encoded payload should decode"),
        payload
    );
}

#[test]
fn native_messaging_decode_rejects_truncated_header_and_length_mismatch() {
    assert_eq!(
        decode_native_messaging_frame(&[0, 0, 0], NativeMessagingFrameDirection::HostToChrome),
        Err(NativeMessagingFrameError::TruncatedHeader)
    );

    let truncated = frame_with_declared_length(5, b"{} ");
    assert_eq!(
        decode_native_messaging_frame(
            &truncated,
            NativeMessagingFrameDirection::HostToChrome,
        ),
        Err(NativeMessagingFrameError::LengthMismatch {
            declared_bytes: 5,
            actual_bytes: 3,
        })
    );

    let trailing = frame_with_declared_length(2, b"{}x");
    assert_eq!(
        decode_native_messaging_frame(&trailing, NativeMessagingFrameDirection::HostToChrome),
        Err(NativeMessagingFrameError::LengthMismatch {
            declared_bytes: 2,
            actual_bytes: 3,
        })
    );
}

#[test]
fn native_messaging_decode_enforces_direction_specific_chrome_limits_before_body_use() {
    let host_to_chrome_oversize = frame_with_declared_length(1_048_577, b"");
    assert_eq!(
        decode_native_messaging_frame(
            &host_to_chrome_oversize,
            NativeMessagingFrameDirection::HostToChrome,
        ),
        Err(NativeMessagingFrameError::PayloadTooLarge {
            declared_bytes: 1_048_577,
            maximum_bytes: 1_048_576,
        })
    );

    let chrome_to_host_oversize = frame_with_declared_length(67_108_865, b"");
    assert_eq!(
        decode_native_messaging_frame(
            &chrome_to_host_oversize,
            NativeMessagingFrameDirection::ChromeToHost,
        ),
        Err(NativeMessagingFrameError::PayloadTooLarge {
            declared_bytes: 67_108_865,
            maximum_bytes: 67_108_864,
        })
    );
}

#[test]
fn native_messaging_decode_rejects_non_utf8_payload_without_reflecting_bytes() {
    let frame = frame_with_declared_length(2, &[0xff, 0xfe]);
    let error = decode_native_messaging_frame(&frame, NativeMessagingFrameDirection::HostToChrome)
        .expect_err("non-UTF-8 native message must fail closed");
    assert_eq!(error, NativeMessagingFrameError::InvalidUtf8);
    assert_eq!(error.to_string(), "native-messaging payload is not valid UTF-8");
}
