use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolAdapterVersionRequirementError,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserProtocolUseValidationError,
    OriginWeaveProtocolVersion,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn descriptor() -> Result<BrowserProtocolAdapterDescriptor, Box<dyn Error>> {
    Ok(BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::Navigation],
    )?)
}

#[test]
fn runtime_adapter_version_is_bound_into_atomic_use_validation() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    let validated = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
    )?;

    assert_eq!(validated.adapter_version(), ADAPTER_VERSION);
    Ok(())
}

#[test]
fn runtime_adapter_version_mismatch_precedes_revision_and_capability_checks()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    assert_eq!(
        descriptor.validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            BrowserProtocolKind::WebDriverBiDi,
            "originweave-bidi-v2",
            "runtime revision with spaces",
            "browser/revision",
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::AdapterVersion(
            BrowserProtocolAdapterVersionRequirementError::AdapterVersionMismatch,
        ))
    );
    Ok(())
}

#[test]
fn malformed_runtime_adapter_version_fails_closed_before_revision_checks()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    assert_eq!(
        descriptor.validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            BrowserProtocolKind::WebDriverBiDi,
            "runtime adapter/version",
            "runtime revision with spaces",
            "browser/revision",
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::AdapterVersion(
            BrowserProtocolAdapterVersionRequirementError::InvalidAdapterVersion,
        ))
    );
    Ok(())
}
