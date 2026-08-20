use std::error::Error;

use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument, NativeMessagingManifestParseError,
};

#[test]
fn empty_native_messaging_host_description_is_not_chrome_valid() {
    let document = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "description":"",
            "path":"/opt/originweave/native-host",
            "type":"stdio",
            "allowed_origins":["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"]
        }"#,
    )
    .expect("the bounded document is syntactically object-shaped");

    let error = document
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect_err("Chrome rejects an empty required native-host description");
    assert_eq!(error, NativeMessagingManifestParseError::InvalidFieldValue);
    assert_eq!(
        error.to_string(),
        "native messaging host manifest field has an invalid value"
    );
    assert!(error.source().is_none());
}
