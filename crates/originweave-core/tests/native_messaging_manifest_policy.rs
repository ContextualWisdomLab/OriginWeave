#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    ExtensionId, NativeMessagingAllowedOrigin, NativeMessagingAllowedOriginError,
    NativeMessagingHostName, NativeMessagingInterfaceType, NativeMessagingInterfaceTypeError,
    NativeMessagingManifestAccessPolicy, NativeMessagingManifestAccessPolicyError,
};

const ALLOWED_EXTENSION_ID: &str = "abcdefghijklmnopabcdefghijklmnop";
const OTHER_EXTENSION_ID: &str = "bcdefghijklmnopabcdefghijklmnopa";

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn allowed_origin(value: &str) -> NativeMessagingAllowedOrigin {
    NativeMessagingAllowedOrigin::parse(value).expect("valid allowed origin")
}

fn host_name() -> NativeMessagingHostName {
    NativeMessagingHostName::parse("com.contextualwisdom.originweave")
        .expect("valid native messaging host name")
}

#[test]
fn native_messaging_allowed_origin_requires_one_exact_chrome_extension_origin() {
    let raw = format!("chrome-extension://{ALLOWED_EXTENSION_ID}/");
    let origin = allowed_origin(&raw);

    assert_eq!(origin.extension_id(), &extension_id(ALLOWED_EXTENSION_ID));
    assert_eq!(origin.as_str(), raw);

    for invalid in [
        "chrome-extension://*/",
        "chrome-extension://abcdefghijklmnopabcdefghijklmnop",
        "chrome-extension://abcdefghijklmnopabcdefghijklmnop/path",
        "https://abcdefghijklmnopabcdefghijklmnop/",
        "chrome-extension://ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP/",
    ] {
        assert_eq!(
            NativeMessagingAllowedOrigin::parse(invalid),
            Err(NativeMessagingAllowedOriginError::InvalidAllowedOrigin),
            "unexpected allowed origin: {invalid:?}"
        );
    }
}

#[test]
fn native_messaging_manifest_interface_is_exactly_stdio() {
    let interface = NativeMessagingInterfaceType::parse("stdio").expect("stdio is the Chrome type");
    assert_eq!(interface.as_str(), "stdio");

    for invalid in ["", "STDIO", "stdio ", "pipe"] {
        assert_eq!(
            NativeMessagingInterfaceType::parse(invalid),
            Err(NativeMessagingInterfaceTypeError::UnsupportedInterfaceType)
        );
    }
}

#[test]
fn native_messaging_manifest_policy_grants_only_exact_listed_extensions() {
    let allowed_extension = extension_id(ALLOWED_EXTENSION_ID);
    let other_extension = extension_id(OTHER_EXTENSION_ID);
    let host = host_name();
    let interface = NativeMessagingInterfaceType::parse("stdio").expect("stdio is valid");
    let origin = allowed_origin(&format!("chrome-extension://{ALLOWED_EXTENSION_ID}/"));

    let policy = NativeMessagingManifestAccessPolicy::new(
        host.clone(),
        interface,
        vec![origin.clone()],
    )
    .expect("non-empty exact allow-list is valid");

    assert_eq!(policy.host_name(), &host);
    assert_eq!(policy.interface_type(), interface);
    assert_eq!(policy.allowed_origins(), &[origin]);

    let grant = policy
        .grant_for(&allowed_extension)
        .expect("listed extension receives an exact host grant");
    assert_eq!(grant.extension_id(), &allowed_extension);
    assert_eq!(grant.host_name(), &host);
    assert!(policy.grant_for(&other_extension).is_none());
}

#[test]
fn native_messaging_manifest_policy_rejects_empty_and_duplicate_allow_lists() {
    let interface = NativeMessagingInterfaceType::parse("stdio").expect("stdio is valid");
    assert_eq!(
        NativeMessagingManifestAccessPolicy::new(host_name(), interface, Vec::new()),
        Err(NativeMessagingManifestAccessPolicyError::MissingAllowedOrigin)
    );

    let origin = allowed_origin(&format!("chrome-extension://{ALLOWED_EXTENSION_ID}/"));
    assert_eq!(
        NativeMessagingManifestAccessPolicy::new(
            host_name(),
            interface,
            vec![origin.clone(), origin],
        ),
        Err(NativeMessagingManifestAccessPolicyError::DuplicateAllowedOrigin)
    );
}

#[test]
fn native_messaging_manifest_policy_errors_are_stable_and_source_free() {
    for (message, error) in [
        (
            "native-messaging allowed origin must be one exact chrome-extension origin",
            NativeMessagingAllowedOriginError::InvalidAllowedOrigin.to_string(),
        ),
        (
            "native-messaging manifest interface type must be stdio",
            NativeMessagingInterfaceTypeError::UnsupportedInterfaceType.to_string(),
        ),
        (
            "native-messaging manifest allowed_origins must contain at least one exact extension",
            NativeMessagingManifestAccessPolicyError::MissingAllowedOrigin.to_string(),
        ),
        (
            "native-messaging manifest allowed_origins must not repeat an extension",
            NativeMessagingManifestAccessPolicyError::DuplicateAllowedOrigin.to_string(),
        ),
    ] {
        assert_eq!(error, message);
    }

    assert!(Error::source(&NativeMessagingAllowedOriginError::InvalidAllowedOrigin).is_none());
    assert!(
        Error::source(&NativeMessagingInterfaceTypeError::UnsupportedInterfaceType).is_none()
    );
    assert!(Error::source(&NativeMessagingManifestAccessPolicyError::MissingAllowedOrigin).is_none());
    assert!(Error::source(&NativeMessagingManifestAccessPolicyError::DuplicateAllowedOrigin).is_none());
}
