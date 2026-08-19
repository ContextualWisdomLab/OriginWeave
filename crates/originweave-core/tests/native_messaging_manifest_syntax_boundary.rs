#![allow(clippy::expect_used)]

use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument, NativeMessagingManifestParseError,
};

#[test]
fn complete_parser_rejects_a_missing_member_separator_after_bounded_admission() {
    let document = NativeMessagingManifestDocument::parse(br#"{"name" "value"}"#)
        .expect("malformed object fixture must pass only the bounded outer-object pre-parser");

    assert!(matches!(
        document.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::InvalidJson)
    ));
}
