#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolDescriptorError,
    BrowserProtocolKind, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
};

const BIDI_ADAPTER_VERSION: &str = "originweave-bidi-v1";
const BIDI_PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const CDP_ADAPTER_VERSION: &str = "originweave-cdp-v1";
const CDP_PROTOCOL_REVISION: &str = "cdp-browser-r1639810";
const BROWSER_REVISION: &str = "chromium-r1639810";

#[test]
fn webdriver_bidi_descriptor_is_explicit_and_capability_bounded() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        BIDI_ADAPTER_VERSION,
        BIDI_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::SemanticObservation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?;

    assert_eq!(descriptor.kind(), BrowserProtocolKind::WebDriverBiDi);
    assert_eq!(descriptor.adapter_version(), BIDI_ADAPTER_VERSION);
    assert_eq!(descriptor.protocol_revision(), BIDI_PROTOCOL_REVISION);
    assert_eq!(descriptor.browser_revision(), BROWSER_REVISION);
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
        CDP_ADAPTER_VERSION,
        CDP_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::NetworkObservation],
    )?;

    assert_eq!(descriptor.protocol_revision(), CDP_PROTOCOL_REVISION);
    assert!(descriptor.supports(BrowserProtocolCapability::NetworkObservation));
    assert!(!descriptor.supports(BrowserProtocolCapability::Navigation));
    assert!(!descriptor.supports(BrowserProtocolCapability::SemanticObservation));
    assert!(!descriptor.supports(BrowserProtocolCapability::TypedInput));
    Ok(())
}

#[test]
fn malformed_or_ambiguous_metadata_fails_closed() {
    let valid_capabilities = [BrowserProtocolCapability::Navigation];
    let invalid_adapter_versions = ["", " ", "bidi version", "bidi/version", "---", "비디"];
    for adapter_version in invalid_adapter_versions {
        assert_eq!(
            BrowserProtocolAdapterDescriptor::new(
                BrowserProtocolKind::WebDriverBiDi,
                adapter_version,
                BIDI_PROTOCOL_REVISION,
                BROWSER_REVISION,
                &valid_capabilities,
            ),
            Err(BrowserProtocolDescriptorError::InvalidAdapterVersion)
        );
    }

    let invalid_protocol_revisions = [
        "",
        " ",
        "webdriver bidi",
        "webdriver/bidi",
        "---",
        "프로토콜",
    ];
    for protocol_revision in invalid_protocol_revisions {
        assert_eq!(
            BrowserProtocolAdapterDescriptor::new(
                BrowserProtocolKind::WebDriverBiDi,
                BIDI_ADAPTER_VERSION,
                protocol_revision,
                BROWSER_REVISION,
                &valid_capabilities,
            ),
            Err(BrowserProtocolDescriptorError::InvalidProtocolRevision)
        );
    }

    let invalid_browser_revisions = [
        "",
        " ",
        "chromium revision",
        "chromium/revision",
        "---",
        "크로미움",
    ];
    for browser_revision in invalid_browser_revisions {
        assert_eq!(
            BrowserProtocolAdapterDescriptor::new(
                BrowserProtocolKind::WebDriverBiDi,
                BIDI_ADAPTER_VERSION,
                BIDI_PROTOCOL_REVISION,
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
            BIDI_PROTOCOL_REVISION,
            BROWSER_REVISION,
            &valid_capabilities,
        ),
        Err(BrowserProtocolDescriptorError::InvalidAdapterVersion)
    );
    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            BIDI_ADAPTER_VERSION,
            &oversized,
            BROWSER_REVISION,
            &valid_capabilities,
        ),
        Err(BrowserProtocolDescriptorError::InvalidProtocolRevision)
    );
    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            BIDI_ADAPTER_VERSION,
            BIDI_PROTOCOL_REVISION,
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
            BIDI_ADAPTER_VERSION,
            BIDI_PROTOCOL_REVISION,
            BROWSER_REVISION,
            &[],
        ),
        Err(BrowserProtocolDescriptorError::EmptyCapabilities)
    );

    assert_eq!(
        BrowserProtocolAdapterDescriptor::new(
            BrowserProtocolKind::WebDriverBiDi,
            BIDI_ADAPTER_VERSION,
            BIDI_PROTOCOL_REVISION,
            BROWSER_REVISION,
            &[
                BrowserProtocolCapability::Navigation,
                BrowserProtocolCapability::Navigation,
            ],
        ),
        Err(BrowserProtocolDescriptorError::DuplicateCapability)
    );
}

#[test]
fn capability_order_does_not_change_descriptor_identity() -> Result<(), Box<dyn Error>> {
    let forward = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        BIDI_ADAPTER_VERSION,
        BIDI_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::SemanticObservation,
            BrowserProtocolCapability::TypedInput,
            BrowserProtocolCapability::NetworkObservation,
        ],
    )?;
    let reverse = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        BIDI_ADAPTER_VERSION,
        BIDI_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[
            BrowserProtocolCapability::NetworkObservation,
            BrowserProtocolCapability::TypedInput,
            BrowserProtocolCapability::SemanticObservation,
            BrowserProtocolCapability::Navigation,
        ],
    )?;

    assert_eq!(forward, reverse);
    Ok(())
}

#[test]
fn descriptor_errors_are_stable_and_source_free() {
    let cases = [
        (
            BrowserProtocolDescriptorError::InvalidAdapterVersion,
            "browser protocol adapter version must be a bounded ASCII metadata token",
        ),
        (
            BrowserProtocolDescriptorError::InvalidProtocolRevision,
            "browser protocol revision must be a bounded ASCII metadata token",
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
