#![allow(clippy::expect_used)]

use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument, NativeMessagingManifestParseError,
};

fn parse_error(bytes: &[u8]) -> NativeMessagingManifestParseError {
    NativeMessagingManifestDocument::parse(bytes)
        .expect("malformed object fixture must pass only the bounded outer-object pre-parser")
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect_err("malformed manifest fixture must fail complete parsing")
}

#[test]
fn complete_parser_rejects_a_missing_member_separator_after_bounded_admission() {
    assert_eq!(
        parse_error(br#"{"name" "value"}"#),
        NativeMessagingManifestParseError::InvalidJson
    );
}

#[test]
fn complete_parser_rejects_non_string_required_fields_through_the_public_boundary() {
    assert_eq!(
        parse_error(br#"{"name":true}"#),
        NativeMessagingManifestParseError::InvalidFieldType
    );
}

#[test]
fn complete_parser_rejects_a_unicode_escape_truncated_by_the_outer_object_boundary() {
    assert_eq!(
        parse_error(br#"{"name":"\u1"}"#),
        NativeMessagingManifestParseError::InvalidJson
    );
}

#[test]
fn complete_parser_preserves_nested_string_failures_at_each_schema_boundary() {
    for raw in [
        br#"{"\q":"value"}"#.as_slice(),
        br#"{"path":"\q"}"#.as_slice(),
        br#"{"type":"\q"}"#.as_slice(),
        br#"{"allowed_origins":["\q"]}"#.as_slice(),
        br#"{"name":"\uD83D\u12"}"#.as_slice(),
    ] {
        assert_eq!(parse_error(raw), NativeMessagingManifestParseError::InvalidJson);
    }
}
