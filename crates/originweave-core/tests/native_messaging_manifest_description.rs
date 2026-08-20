use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument,
    NativeMessagingManifestParseError,
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

    assert_eq!(
        document.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::InvalidFieldValue)
    );
}
