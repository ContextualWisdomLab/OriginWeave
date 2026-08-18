#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES, NativeMessagingHostManifestError,
    NativeMessagingHostPlatform, NativeMessagingManifestDocument,
    NativeMessagingManifestDocumentError, NativeMessagingManifestParseError,
};

const ALLOWED_EXTENSION: &str = "abcdefghijklmnopabcdefghijklmnop";
const LINUX_HOST_PATH: &str = "/opt/originweave/native-host";

#[test]
fn native_messaging_manifest_document_is_bounded_before_text_storage() {
    let mut exact_limit = vec![b' '; MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES];
    exact_limit[0] = b'{';
    exact_limit[MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES - 1] = b'}';
    let document = NativeMessagingManifestDocument::parse(&exact_limit)
        .expect("the exact OriginWeave manifest-document safety bound remains accepted");
    assert_eq!(document.as_str().len(), exact_limit.len());

    let one_over = vec![b' '; MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES + 1];
    assert_eq!(
        NativeMessagingManifestDocument::parse(&one_over),
        Err(NativeMessagingManifestDocumentError::DocumentTooLarge)
    );
}

#[test]
fn native_messaging_manifest_document_rejects_empty_and_invalid_utf8() {
    assert_eq!(
        NativeMessagingManifestDocument::parse(&[]),
        Err(NativeMessagingManifestDocumentError::EmptyDocument)
    );
    assert_eq!(
        NativeMessagingManifestDocument::parse(&[0xff]),
        Err(NativeMessagingManifestDocumentError::InvalidUtf8)
    );
}

#[test]
fn native_messaging_manifest_document_requires_outer_object_boundary() {
    assert_eq!(
        NativeMessagingManifestDocument::parse(b"[]"),
        Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary)
    );
    assert_eq!(
        NativeMessagingManifestDocument::parse(b"{"),
        Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary)
    );

    let document = NativeMessagingManifestDocument::parse(b" \r\n{\n}\t ")
        .expect("JSON whitespace around an object-shaped document remains accepted");
    assert_eq!(document.as_str(), " \r\n{\n}\t ");
}

#[test]
fn native_messaging_manifest_document_parses_complete_authority_fields() -> Result<(), Box<dyn Error>> {
    let json = format!(
        r#"{{
            "name": "com.contextualwisdom.originweave",
            "description": "OriginWeave native host",
            "path": "{LINUX_HOST_PATH}",
            "type": "stdio",
            "allowed_origins": ["chrome-extension://{ALLOWED_EXTENSION}/"],
            "supports_native_initiated_connections": true
        }}"#
    );
    let document = NativeMessagingManifestDocument::parse(json.as_bytes())?;
    let manifest = document.parse_host_manifest(NativeMessagingHostPlatform::Linux)?;

    assert_eq!(manifest.host_name().as_str(), "com.contextualwisdom.originweave");
    assert_eq!(manifest.platform(), NativeMessagingHostPlatform::Linux);
    assert_eq!(manifest.executable_path(), LINUX_HOST_PATH);
    assert_eq!(manifest.allowed_extension_count(), 1);
    assert!(manifest.supports_native_initiated_connections());
    Ok(())
}

#[test]
fn native_messaging_manifest_document_decodes_json_string_escapes_before_validation()
-> Result<(), Box<dyn Error>> {
    let json = format!(
        r#"{{
            "name": "com.contextualwisdom.origin\u0077eave",
            "description": "Origin\\Weave \"native\" host",
            "path": "\/opt\/originweave\/native-host",
            "type": "st\u0064io",
            "allowed_origins": ["chrome-extension:\/\/{ALLOWED_EXTENSION}\/"],
            "supports_native_initiated_connections": false
        }}"#
    );
    let document = NativeMessagingManifestDocument::parse(json.as_bytes())?;
    let manifest = document.parse_host_manifest(NativeMessagingHostPlatform::Linux)?;

    assert_eq!(manifest.host_name().as_str(), "com.contextualwisdom.originweave");
    assert_eq!(manifest.executable_path(), LINUX_HOST_PATH);
    assert!(!manifest.supports_native_initiated_connections());
    Ok(())
}

#[test]
fn native_messaging_manifest_document_rejects_incomplete_or_ambiguous_json() {
    for malformed in [
        r#"{"name":"com.contextualwisdom.originweave",}"#,
        r#"{"name":"com.contextualwisdom.originweave" "description":"missing comma"}"#,
        r#"{"name":"com.contextualwisdom.originweave","description":"bad\q"}"#,
        r#"{"name":"com.contextualwisdom.originweave","description":"bad\uD800"}"#,
    ] {
        let document = NativeMessagingManifestDocument::parse(malformed.as_bytes())
            .expect("the pre-parser only proves the outer object boundary");
        assert_eq!(
            document.parse_host_manifest(NativeMessagingHostPlatform::Linux),
            Err(NativeMessagingManifestParseError::InvalidJson),
            "unexpected malformed document: {malformed}"
        );
    }
}

#[test]
fn native_messaging_manifest_document_rejects_duplicate_unknown_missing_and_wrong_typed_fields() {
    let duplicate = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "name":"com.contextualwisdom.other",
            "description":"host",
            "path":"/opt/originweave/native-host",
            "type":"stdio",
            "allowed_origins":["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"]
        }"#,
    )
    .expect("outer object boundary remains valid");
    assert_eq!(
        duplicate.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::DuplicateField)
    );

    let unknown = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "description":"host",
            "path":"/opt/originweave/native-host",
            "type":"stdio",
            "allowed_origins":["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"],
            "ambient_authority":true
        }"#,
    )
    .expect("outer object boundary remains valid");
    assert_eq!(
        unknown.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::UnknownField)
    );

    let missing_description = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "path":"/opt/originweave/native-host",
            "type":"stdio",
            "allowed_origins":["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"]
        }"#,
    )
    .expect("outer object boundary remains valid");
    assert_eq!(
        missing_description.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::MissingRequiredField)
    );

    let wrong_type = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "description":"host",
            "path":"/opt/originweave/native-host",
            "type":"stdio",
            "allowed_origins":"chrome-extension://abcdefghijklmnopabcdefghijklmnop/"
        }"#,
    )
    .expect("outer object boundary remains valid");
    assert_eq!(
        wrong_type.parse_host_manifest(NativeMessagingHostPlatform::Linux),
        Err(NativeMessagingManifestParseError::InvalidFieldType)
    );
}

#[test]
fn native_messaging_manifest_document_preserves_typed_manifest_validation_failure() {
    let document = NativeMessagingManifestDocument::parse(
        br#"{
            "name":"com.contextualwisdom.originweave",
            "description":"host",
            "path":"/opt/originweave/native-host",
            "type":"pipe",
            "allowed_origins":["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"]
        }"#,
    )
    .expect("outer object boundary remains valid");

    let error = document
        .parse_host_manifest(NativeMessagingHostPlatform::Linux)
        .expect_err("unsupported Chrome interface type must fail closed");
    assert_eq!(
        error,
        NativeMessagingManifestParseError::Manifest(
            NativeMessagingHostManifestError::UnsupportedInterfaceType
        )
    );
    assert!(error.source().is_some());
}

#[test]
fn native_messaging_manifest_document_errors_are_standard_and_source_free() {
    for (error, expected) in [
        (
            NativeMessagingManifestDocumentError::EmptyDocument,
            "native messaging host manifest document is empty",
        ),
        (
            NativeMessagingManifestDocumentError::DocumentTooLarge,
            "native messaging host manifest document exceeds the OriginWeave safety budget",
        ),
        (
            NativeMessagingManifestDocumentError::InvalidUtf8,
            "native messaging host manifest document is not valid UTF-8",
        ),
        (
            NativeMessagingManifestDocumentError::InvalidObjectBoundary,
            "native messaging host manifest document must have one outer JSON object boundary",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
