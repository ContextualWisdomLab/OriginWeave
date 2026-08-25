use std::error::Error;
use std::io::{Cursor, ErrorKind};

use originweave_core::{
    NativeMessagingFrameDirection, NativeMessagingFrameError, NativeMessagingFrameReadError,
    decode_native_messaging_frame, decode_native_messaging_text_frame,
    encode_native_messaging_frame, native_messaging_payload_limit, read_native_messaging_payload,
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
fn native_messaging_frame_round_trip_uses_native_u32_byte_length() -> Result<(), Box<dyn Error>> {
    let payload = b"{}";

    for direction in [
        NativeMessagingFrameDirection::HostToBrowser,
        NativeMessagingFrameDirection::BrowserToHost,
    ] {
        let frame = encode_native_messaging_frame(direction, payload)?;
        assert_eq!(&frame[..4], &2_u32.to_ne_bytes());
        assert_eq!(decode_native_messaging_frame(direction, &frame)?, payload);
    }
    Ok(())
}

#[test]
fn native_messaging_stream_reader_round_trips_one_bounded_payload() -> Result<(), Box<dyn Error>> {
    let payload = b"{\"message\":\"bounded\"}";
    let frame =
        encode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, payload)?;
    let mut reader = Cursor::new(frame);

    assert_eq!(
        read_native_messaging_payload(NativeMessagingFrameDirection::HostToBrowser, &mut reader,)?,
        payload
    );
    assert_eq!(reader.position(), payload.len() as u64 + 4);
    Ok(())
}

#[test]
fn native_messaging_stream_reader_rejects_oversized_prefix_before_reading_payload() {
    let oversized = (HOST_TO_BROWSER_LIMIT as u32 + 1).to_ne_bytes();
    let mut reader = Cursor::new(oversized);

    assert!(matches!(
        read_native_messaging_payload(NativeMessagingFrameDirection::HostToBrowser, &mut reader,),
        Err(NativeMessagingFrameReadError::Frame(
            NativeMessagingFrameError::PayloadTooLarge
        ))
    ));
    assert_eq!(reader.position(), 4);
}

#[test]
fn native_messaging_stream_reader_preserves_truncated_payload_io_cause() {
    let mut frame = Vec::from(4_u32.to_ne_bytes());
    frame.extend_from_slice(b"abc");
    let mut reader = Cursor::new(frame);

    match read_native_messaging_payload(NativeMessagingFrameDirection::HostToBrowser, &mut reader) {
        Err(NativeMessagingFrameReadError::Io(error)) => {
            assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        }
        other => panic!("expected typed I/O failure, got {other:?}"),
    }
}

#[test]
fn native_messaging_text_frame_accepts_utf8_and_rejects_invalid_text() -> Result<(), Box<dyn Error>>
{
    let utf8_payload = "{\"message\":\"안녕 👋\"}".as_bytes();
    let utf8_frame =
        encode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, utf8_payload)?;
    assert_eq!(
        decode_native_messaging_text_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &utf8_frame,
        )?,
        "{\"message\":\"안녕 👋\"}"
    );

    let invalid_utf8_frame =
        encode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &[0xff])?;
    assert_eq!(
        decode_native_messaging_text_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &invalid_utf8_frame,
        ),
        Err(NativeMessagingFrameError::InvalidUtf8Payload)
    );
    assert_eq!(
        decode_native_messaging_text_frame(
            NativeMessagingFrameDirection::HostToBrowser,
            &[0, 0, 0],
        ),
        Err(NativeMessagingFrameError::MissingLengthPrefix)
    );
    Ok(())
}

#[test]
fn native_messaging_encoder_rejects_oversized_host_payload_before_framing() {
    let oversized = vec![b'x'; HOST_TO_BROWSER_LIMIT + 1];

    assert_eq!(
        encode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &oversized,),
        Err(NativeMessagingFrameError::PayloadTooLarge)
    );
}

#[test]
fn native_messaging_decoder_rejects_missing_oversized_and_mismatched_lengths() {
    assert_eq!(
        decode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &[0, 0, 0],),
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
        decode_native_messaging_frame(NativeMessagingFrameDirection::HostToBrowser, &short_frame,),
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
        (
            NativeMessagingFrameError::InvalidUtf8Payload,
            "native messaging payload is not valid UTF-8",
        ),
    ] {
        assert_eq!(error.to_string(), message);
        assert!(Error::source(&error).is_none());
    }
}
