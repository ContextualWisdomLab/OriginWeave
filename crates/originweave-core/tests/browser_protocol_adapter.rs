#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolCapabilityRequirementError, BrowserProtocolDescriptorError, BrowserProtocolKind,
    BrowserProtocolVersionRequirementError, MAX_BROWSER_PROTOCOL_METADATA_BYTES,
    OriginWeaveProtocolVersion,
};

const CURRENT_ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const FUTURE_ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 2);
const BIDI_ADAPTER_VERSION: &str = "originweave-bidi-v1";
const BIDI_PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const CDP_ADAPTER_VERSION: &str = "originweave-cdp-v1";
const CDP_PROTOCOL_REVISION: &str = "cdp-browser-r1639810";
const BROWSER_REVISION: &str = "chromium-r1639810";

#[test]
fn originweave_protocol_version_is_explicit_and_canonical() {
    assert_eq!(CURRENT_ORIGINWEAVE_PROTOCOL_VERSION.major(), 0);
    assert_eq!(CURRENT_ORIGINWEAVE_PROTOCOL_VERSION.minor(), 1);
    assert_eq!(
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION.to_string(),
        "originweave/0.1"
    );
    assert_ne!(
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
        FUTURE_ORIGINWEAVE_PROTOCOL_VERSION
    );
}

#[test]
fn webdriver_bidi_descriptor_is_explicit_and_capability_bounded() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
    assert_eq!(
        descriptor.originweave_protocol_version(),
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION
    );
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
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
fn required_capability_fails_closed_without_side_effectful_fallback() -> Result<(), Box<dyn Error>>
{
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
        BIDI_ADAPTER_VERSION,
        BIDI_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::Navigation],
    )?;

    assert_eq!(
        descriptor.require_capability(BrowserProtocolCapability::Navigation),
        Ok(())
    );
    assert_eq!(
        descriptor.require_capability(BrowserProtocolCapability::NetworkObservation),
        Err(
            BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                BrowserProtocolCapability::NetworkObservation,
            )
        )
    );
    Ok(())
}

#[test]
fn required_originweave_protocol_version_fails_closed() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
        BIDI_ADAPTER_VERSION,
        BIDI_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::Navigation],
    )?;

    assert_eq!(
        descriptor.require_originweave_protocol_version(CURRENT_ORIGINWEAVE_PROTOCOL_VERSION),
        Ok(())
    );
    assert_eq!(
        descriptor.require_originweave_protocol_version(FUTURE_ORIGINWEAVE_PROTOCOL_VERSION),
        Err(
            BrowserProtocolVersionRequirementError::ProtocolVersionMismatch {
                required: FUTURE_ORIGINWEAVE_PROTOCOL_VERSION,
                actual: CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
            }
        )
    );
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
                CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
                CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
                CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
            CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
            CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
            CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
            CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
            CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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
        CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
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

#[test]
fn capability_requirement_errors_are_stable_and_source_free() {
    let cases = [
        (
            BrowserProtocolCapability::Navigation,
            "browser protocol adapter does not declare required navigation capability",
        ),
        (
            BrowserProtocolCapability::SemanticObservation,
            "browser protocol adapter does not declare required semantic-observation capability",
        ),
        (
            BrowserProtocolCapability::TypedInput,
            "browser protocol adapter does not declare required typed-input capability",
        ),
        (
            BrowserProtocolCapability::NetworkObservation,
            "browser protocol adapter does not declare required network-observation capability",
        ),
    ];

    for (capability, expected) in cases {
        let error = BrowserProtocolCapabilityRequirementError::UnsupportedCapability(capability);
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}

#[test]
fn protocol_version_requirement_error_is_stable_and_source_free() {
    let error = BrowserProtocolVersionRequirementError::ProtocolVersionMismatch {
        required: FUTURE_ORIGINWEAVE_PROTOCOL_VERSION,
        actual: CURRENT_ORIGINWEAVE_PROTOCOL_VERSION,
    };

    assert_eq!(
        error.to_string(),
        "browser protocol adapter targets originweave/0.1 but originweave/0.2 is required"
    );
    assert!(error.source().is_none());
}
