#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolDescriptorError,
    BrowserProtocolKind, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
};

#[test]
fn webdriver_bidi_descriptor_is_explicit_and_capability_bounded() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        "bidi-2026-06-29",
        "chromium-r1639810",
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::SemanticObservation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?;

    assert_eq!(descriptor.kind(), BrowserProtocolKind::WebDriverBiDi);
    assert_eq!(descriptor.adapter_version(), "bidi-2026-06-29");
    assert_eq!(descriptor.browser_revision(), "chromium-r1639810");
    assert_eq!(descriptor.capability_count(), 3);
    assert!(descriptor.supports(BrowserProtocolCapability::Navigation));
    assert!(descriptor.supports(BrowserProtocolCapability::SemanticObservation));
    assert!(descriptor.supports(BrowserProtocolCapability::TypedInput));
    assert!(!descriptor.supports(BrowserProtocolCapability::NetworkObservation));
    Ok(())
}

#[test]
fn cdp_capability_is_not_inferred_from_protocol_kind() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::ChromeDevToolsProtocol,
        "cdp-150",
        "chromium-r1639810",
        &[BrowserProtocolCapability::NetworkObservation],
    )?;

    assert!(descriptor.supports(BrowserProtocolCapability::NetworkObservation));
    assert!(!descriptor.supports(BrowserProtocolCapability::Navigation));
    assert!(!descriptor.supports(BrowserProtocolCapability::SemanticObservation));
    assert!(!descriptor.supports(BrowserProtocolCapability::TypedInput));
    Ok(())
}

#[test]
fn malformed_or_ambiguous_metadata_fails_closed() {
    let valid_capabilities = [BrowserProtocolCapability::Navigation];
    let invalid_adapter_versions = ["", " ", "bidi version", "bidi/version", "---"];
    for adapter_version in invalid_adapter_versions {
        assert_eq!(
            BrowserProtocolAdapterDescriptor::new(
                BrowserProtocolKind::WebDriverBiDi,
                adapter_version,
                "chromium-r1639810",
                &valid_capabilities,
            ),
            Err(BrowserProtocolDescriptorError::InvalidAdapterVersion)
        );
    }

    let invalid_browser_revisions = ["", " ", "chromium revision", "chromium/revision", "---"];
    for browser_revision in invalid_browser_revisions {
        assert_eq!(
            BrowserProtocolAdapterDescriptor::new(
                BrowserProtocolKind::WebDriverBiDi,
                "bidi-2026-06-29",
                browser_revision,
                &valid_capabilities,
            ),
            Err(BrowserProtocolDescriptorError::InvalidBrowserRevision)
        );
    }

    let oversized = "a".repeat(MAX_BROWSER_PROTOCOL_METADATA_BYTES + 1);
    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            &oversized,
            "chromium-r1639810",
            &valid_capabilities,
        ),
        Err(BrowserProtocolDescriptorError::InvalidAdapterVersion)
    );
    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            "bidi-2026-06-29",
            &oversized,
            &valid_capabilities,
        ),
        Err(BrowserProtocolDescriptorError::InvalidBrowserRevision)
    );
}

#[test]
fn capability_set_must_be_nonempty_and_canonical() {
    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            "bidi-2026-06-29",
            "chromium-r1639810",
            &[],
        ),
        Err(BrowserProtocolDescriptorError::EmptyCapabilities)
    );

    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            "bidi-2026-06-29",
            "chromium-r1639810",
            &[
                BrowserProtocolCapability::Navigation,
                BrowserProtocolCapability::Navigation,
            ],
        ),
        Err(BrowserProtocolDescriptorError::DuplicateCapability)
    );
}

#[test]
fn descriptor_errors_are_stable_and_source_free() {
    let cases = [
        (
            BrowserProtocolDescriptorError::InvalidAdapterVersion,
            "browser protocol adapter version must be a bounded ASCII metadata token",
        ),
        (
            BrowserProtocolDescriptorError::InvalidBrowserRevision,
            "browser revision must be a bounded ASCII metadata token",
        ),
        (
            BrowserProtocolDescriptorError::EmptyCapabilities,
            "browser protocol adapter must declare at least one capability",
        ),
        (
            BrowserProtocolDescriptorError::DuplicateCapability,
            "browser protocol adapter capabilities must be unique",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
