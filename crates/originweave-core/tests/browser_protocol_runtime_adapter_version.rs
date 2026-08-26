use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolUseValidationError, OriginWeaveProtocolVersion,
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

    let error = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        "originweave-bidi-v2",
        "runtime revision with spaces",
        "browser/revision",
        BrowserProtocolCapability::NetworkObservation,
    );

    assert_eq!(
        error,
        Err(BrowserProtocolUseValidationError::AdapterVersionMismatch)
    );
    let error = error.err().ok_or("expected adapter version mismatch")?;
    assert_eq!(
        error.to_string(),
        "runtime browser adapter version does not match the pinned adapter version"
    );
    assert!(error.source().is_none());
    Ok(())
}

#[test]
fn malformed_runtime_adapter_version_fails_closed_before_revision_checks()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    let error = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        "runtime adapter/version",
        "runtime revision with spaces",
        "browser/revision",
        BrowserProtocolCapability::NetworkObservation,
    );

    assert_eq!(
        error,
        Err(BrowserProtocolUseValidationError::InvalidAdapterVersion)
    );
    let error = error.err().ok_or("expected invalid adapter version")?;
    assert_eq!(
        error.to_string(),
        "runtime browser adapter version must be a bounded ASCII metadata token"
    );
    assert!(error.source().is_none());
    Ok(())
}
