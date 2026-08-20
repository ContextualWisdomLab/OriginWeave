#![allow(clippy::expect_used)]

use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument, NativeMessagingManifestParseError,
};

const EXTENSION_ORIGIN: &str = "chrome-extension://abcdefghijklmnopabcdefghijklmnop/";

#[test]
fn complete_parser_enforces_origin_budget_before_decoding_excess_element() {
    let allowed_origins = vec![format!("\"{EXTENSION_ORIGIN}\""); 256].join(",");
    let raw = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":[{allowed_origins},1]}}"#
    );

    let error = NativeMessagingManifestDocument::parse(raw.as_bytes())
        .expect("bounded over-budget fixture must pass the document pre-parser")
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect_err("the 257th origin entry must fail at the origin-count budget before decoding");

    assert!(matches!(
        error,
        NativeMessagingManifestParseError::Manifest(_)
    ));
}
