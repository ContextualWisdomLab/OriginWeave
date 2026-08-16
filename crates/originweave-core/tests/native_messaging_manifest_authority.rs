#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    ExtensionId, MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS, NativeMessagingAccessRequest,
    NativeMessagingHostManifest, NativeMessagingHostManifestAccessDecision,
    NativeMessagingHostManifestError, NativeMessagingHostName, NativeMessagingHostPlatform,
};

const ALLOWED_EXTENSION: &str = "abcdefghijklmnopabcdefghijklmnop";
const OTHER_EXTENSION: &str = "bcdefghijklmnopabcdefghijklmnopa";
const LINUX_HOST_PATH: &str = "/opt/originweave/native-host";

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn host_name(value: &str) -> NativeMessagingHostName {
    NativeMessagingHostName::parse(value).expect("valid native messaging host name")
}

fn extension_origin(value: &str) -> String {
    format!("chrome-extension://{value}/")
}

#[test]
fn manifest_binds_stdio_host_path_and_exact_allowed_extension_origins() -> Result<(), Box<dyn Error>>
{
    let host = host_name("com.contextualwisdom.originweave");
    let allowed_origin = extension_origin(ALLOWED_EXTENSION);
    let manifest = NativeMessagingHostManifest::parse(
        host.clone(),
        NativeMessagingHostPlatform::Linux,
        LINUX_HOST_PATH,
        "stdio",
        &[allowed_origin.as_str(), allowed_origin.as_str()],
    )?;

    assert_eq!(manifest.host_name(), &host);
    assert_eq!(manifest.platform(), NativeMessagingHostPlatform::Linux);
    assert_eq!(manifest.executable_path(), LINUX_HOST_PATH);
    assert_eq!(manifest.allowed_extension_count(), 1);

    let exact = NativeMessagingAccessRequest::new(extension_id(ALLOWED_EXTENSION), host.clone());
    assert_eq!(
        manifest.evaluate(&exact),
        NativeMessagingHostManifestAccessDecision::Allow
    );

    let wrong_host = NativeMessagingAccessRequest::new(
        extension_id(ALLOWED_EXTENSION),
        host_name("com.contextualwisdom.other_host"),
    );
    assert_eq!(
        manifest.evaluate(&wrong_host),
        NativeMessagingHostManifestAccessDecision::DenyHostMismatch
    );

    let wrong_extension =
        NativeMessagingAccessRequest::new(extension_id(OTHER_EXTENSION), host.clone());
    assert_eq!(
        manifest.evaluate(&wrong_extension),
        NativeMessagingHostManifestAccessDecision::DenyExtensionNotAllowed
    );
    Ok(())
}

#[test]
fn manifest_enforces_platform_specific_executable_path_shape() -> Result<(), Box<dyn Error>> {
    let host = host_name("com.contextualwisdom.originweave");
    let allowed_origin = extension_origin(ALLOWED_EXTENSION);

    let windows = NativeMessagingHostManifest::parse(
        host.clone(),
        NativeMessagingHostPlatform::Windows,
        "native-host.exe",
        "stdio",
        &[allowed_origin.as_str()],
    )?;
    assert_eq!(windows.platform(), NativeMessagingHostPlatform::Windows);
    assert_eq!(windows.executable_path(), "native-host.exe");

    for platform in [
        NativeMessagingHostPlatform::Linux,
        NativeMessagingHostPlatform::MacOs,
    ] {
        assert_eq!(
            NativeMessagingHostManifest::parse(
                host.clone(),
                platform,
                "relative/native-host",
                "stdio",
                &[allowed_origin.as_str()],
            ),
            Err(NativeMessagingHostManifestError::RelativeExecutablePathUnsupported)
        );
    }

    for invalid_path in ["", "bad\0path"] {
        assert_eq!(
            NativeMessagingHostManifest::parse(
                host.clone(),
                NativeMessagingHostPlatform::Windows,
                invalid_path,
                "stdio",
                &[allowed_origin.as_str()],
            ),
            Err(NativeMessagingHostManifestError::InvalidExecutablePath)
        );
    }
    Ok(())
}

#[test]
fn manifest_rejects_non_stdio_empty_and_oversized_allowlists() {
    let host = host_name("com.contextualwisdom.originweave");
    let allowed_origin = extension_origin(ALLOWED_EXTENSION);

    assert_eq!(
        NativeMessagingHostManifest::parse(
            host.clone(),
            NativeMessagingHostPlatform::Linux,
            LINUX_HOST_PATH,
            "pipe",
            &[allowed_origin.as_str()],
        ),
        Err(NativeMessagingHostManifestError::UnsupportedInterfaceType)
    );
    assert_eq!(
        NativeMessagingHostManifest::parse(
            host.clone(),
            NativeMessagingHostPlatform::Linux,
            LINUX_HOST_PATH,
            "stdio",
            &[],
        ),
        Err(NativeMessagingHostManifestError::MissingAllowedOrigin)
    );

    let oversized = vec![allowed_origin.as_str(); MAX_NATIVE_MESSAGING_ALLOWED_ORIGINS + 1];
    assert_eq!(
        NativeMessagingHostManifest::parse(
            host,
            NativeMessagingHostPlatform::Linux,
            LINUX_HOST_PATH,
            "stdio",
            &oversized,
        ),
        Err(NativeMessagingHostManifestError::TooManyAllowedOrigins)
    );
}

#[test]
fn manifest_rejects_ambiguous_or_wildcard_extension_origins() {
    let host = host_name("com.contextualwisdom.originweave");
    let invalid_origins = [
        "chrome-extension://*/",
        "https://abcdefghijklmnopabcdefghijklmnop/",
        "chrome-extension://abcdefghijklmnopabcdefghijklmnop",
        "chrome-extension://abcdefghijklmnopabcdefghijklmnop/path",
        "chrome-extension://ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP/",
        "chrome-extension://abcdefghijklmnopabcdefghijklmnop/?query=1",
    ];

    for invalid in invalid_origins {
        assert_eq!(
            NativeMessagingHostManifest::parse(
                host.clone(),
                NativeMessagingHostPlatform::Linux,
                LINUX_HOST_PATH,
                "stdio",
                &[invalid],
            ),
            Err(NativeMessagingHostManifestError::InvalidAllowedOrigin),
            "unexpected allowed origin: {invalid:?}"
        );
    }
}

#[test]
fn manifest_error_messages_are_deterministic_and_source_free() {
    let cases = [
        (
            NativeMessagingHostManifestError::UnsupportedInterfaceType,
            "native messaging host manifest interface type must be stdio",
        ),
        (
            NativeMessagingHostManifestError::InvalidExecutablePath,
            "native messaging host manifest contains an invalid executable path",
        ),
        (
            NativeMessagingHostManifestError::RelativeExecutablePathUnsupported,
            "native messaging host executable path must be absolute on this platform",
        ),
        (
            NativeMessagingHostManifestError::MissingAllowedOrigin,
            "native messaging host manifest must allow at least one exact extension origin",
        ),
        (
            NativeMessagingHostManifestError::TooManyAllowedOrigins,
            "native messaging host manifest exceeds the OriginWeave allowed-origin safety budget",
        ),
        (
            NativeMessagingHostManifestError::InvalidAllowedOrigin,
            "native messaging host manifest contains an invalid extension origin",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
