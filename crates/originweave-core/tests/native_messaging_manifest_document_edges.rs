#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    NativeMessagingHostPlatform, NativeMessagingManifestDocument, NativeMessagingManifestParseError,
};

const EXTENSION_ORIGIN: &str = "chrome-extension://abcdefghijklmnopabcdefghijklmnop/";

fn parse_error(raw: &str) -> NativeMessagingManifestParseError {
    NativeMessagingManifestDocument::parse(raw.as_bytes())
        .expect("edge fixture must pass only the bounded outer-object pre-parser")
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect_err("edge fixture must fail complete manifest parsing or authority validation")
}

#[test]
fn complete_parser_rejects_every_duplicate_reviewed_field() {
    let cases = [
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","name":"com.contextualwisdom.other","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","description":"other","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","path":"/tmp/other","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"],"allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"],"supports_native_initiated_connections":true,"supports_native_initiated_connections":false}}"#
        ),
    ];

    for raw in cases {
        assert_eq!(
            parse_error(&raw),
            NativeMessagingManifestParseError::DuplicateField
        );
    }
}

#[test]
fn complete_parser_covers_empty_and_malformed_array_and_boolean_shapes() {
    let empty_object = NativeMessagingManifestDocument::parse(b"{}")
        .expect("empty object passes only the bounded outer-object pre-parser");
    assert_eq!(
        empty_object.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::MissingRequiredField)
    );

    let empty_origins = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":[]}}"#
    );
    assert!(matches!(
        parse_error(&empty_origins),
        NativeMessagingManifestParseError::Manifest(_)
    ));

    for raw in [
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":[true]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":"{EXTENSION_ORIGIN}"}}"#
        ),
    ] {
        assert_eq!(
            parse_error(&raw),
            NativeMessagingManifestParseError::InvalidFieldType
        );
    }

    for raw in [
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}",]}}"#
        ),
        format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}" "{EXTENSION_ORIGIN}"]}}"#
        ),
        "{} {}".to_owned(),
    ] {
        assert_eq!(
            parse_error(&raw),
            NativeMessagingManifestParseError::InvalidJson
        );
    }

    let invalid_boolean = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"],"supports_native_initiated_connections":1}}"#
    );
    assert_eq!(
        parse_error(&invalid_boolean),
        NativeMessagingManifestParseError::InvalidFieldType
    );

    let false_boolean = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"],"supports_native_initiated_connections":false}}"#
    );
    let manifest = NativeMessagingManifestDocument::parse(false_boolean.as_bytes())
        .expect("valid false-boolean fixture passes pre-parser")
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect("valid false-boolean fixture passes complete parsing");
    assert!(!manifest.supports_native_initiated_connections());
}

#[test]
fn complete_parser_covers_json_escape_and_unicode_failure_edges() {
    let escaped_description = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"a\b\f\n\r\t-\u0041-\u00E9-\u263A-\uD83D\uDE00-\u00AF","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
    );
    let manifest = NativeMessagingManifestDocument::parse(escaped_description.as_bytes())
        .expect("valid escaped-string fixture passes pre-parser")
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect("valid escaped-string fixture passes complete parsing");
    assert_eq!(manifest.allowed_extension_count(), 1);

    for description in [
        r#"bad\q"#,
        r#"bad\uD83D"#,
        r#"bad\uD83D\x0000"#,
        r#"bad\uD83D\u0041"#,
        r#"bad\uDE00"#,
        r#"bad\u12"#,
        r#"bad\u00G0"#,
    ] {
        let raw = format!(
            r#"{{"name":"com.contextualwisdom.originweave","description":"{description}","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
        );
        assert_eq!(
            parse_error(&raw),
            NativeMessagingManifestParseError::InvalidJson
        );
    }

    let raw_control = format!(
        "{{\"name\":\"com.contextualwisdom.originweave\",\"description\":\"bad\u{0001}text\",\"path\":\"/opt/originweave/native-host\",\"type\":\"stdio\",\"allowed_origins\":[\"{EXTENSION_ORIGIN}\"]}}"
    );
    assert_eq!(
        parse_error(&raw_control),
        NativeMessagingManifestParseError::InvalidJson
    );

    let unterminated = r#"{"name":"unterminated}"#;
    assert_eq!(
        parse_error(unterminated),
        NativeMessagingManifestParseError::InvalidJson
    );
}

#[test]
fn parse_errors_expose_deterministic_display_and_only_causal_sources() {
    for error in [
        NativeMessagingManifestParseError::InvalidJson,
        NativeMessagingManifestParseError::DuplicateField,
        NativeMessagingManifestParseError::UnknownField,
        NativeMessagingManifestParseError::MissingRequiredField,
        NativeMessagingManifestParseError::InvalidFieldType,
    ] {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }

    let invalid_host = format!(
        r#"{{"name":"INVALID HOST","description":"host","path":"/opt/originweave/native-host","type":"stdio","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
    );
    let host_error = parse_error(&invalid_host);
    assert!(matches!(
        host_error,
        NativeMessagingManifestParseError::HostName(_)
    ));
    assert!(!host_error.to_string().is_empty());
    assert!(host_error.source().is_some());

    let invalid_manifest = format!(
        r#"{{"name":"com.contextualwisdom.originweave","description":"host","path":"/opt/originweave/native-host","type":"pipe","allowed_origins":["{EXTENSION_ORIGIN}"]}}"#
    );
    let manifest_error = parse_error(&invalid_manifest);
    assert!(matches!(
        manifest_error,
        NativeMessagingManifestParseError::Manifest(_)
    ));
    assert!(!manifest_error.to_string().is_empty());
    assert!(manifest_error.source().is_some());
}
