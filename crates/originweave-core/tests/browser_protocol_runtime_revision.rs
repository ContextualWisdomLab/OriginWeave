use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolRuntimeRequirementError, OriginWeaveProtocolVersion,
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
fn exact_runtime_revisions_are_required_before_adapter_use() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    assert_eq!(
        descriptor.require_runtime_revisions(PROTOCOL_REVISION, BROWSER_REVISION),
        Ok(())
    );
    Ok(())
}

#[test]
fn runtime_revision_drift_fails_closed() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    assert_eq!(
        descriptor.require_runtime_revisions("webdriver-bidi-wd-2026-07-01", BROWSER_REVISION),
        Err(BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch)
    );
    assert_eq!(
        descriptor.require_runtime_revisions(PROTOCOL_REVISION, "chromium-r1639811"),
        Err(BrowserProtocolRuntimeRequirementError::BrowserRevisionMismatch)
    );
    assert_eq!(
        descriptor.require_runtime_revisions("webdriver-bidi-wd-2026-07-01", "chromium-r1639811"),
        Err(BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch)
    );
    Ok(())
}

#[test]
fn malformed_runtime_revision_evidence_fails_before_comparison() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    assert_eq!(
        descriptor.require_runtime_revisions("webdriver bidi current", BROWSER_REVISION),
        Err(BrowserProtocolRuntimeRequirementError::InvalidProtocolRevision)
    );
    assert_eq!(
        descriptor.require_runtime_revisions(PROTOCOL_REVISION, "chromium/current"),
        Err(BrowserProtocolRuntimeRequirementError::InvalidBrowserRevision)
    );
    assert_eq!(
        descriptor.require_runtime_revisions("", ""),
        Err(BrowserProtocolRuntimeRequirementError::InvalidProtocolRevision)
    );
    Ok(())
}

#[test]
fn runtime_requirement_errors_are_stable_and_source_free() {
    let cases = [
        (
            BrowserProtocolRuntimeRequirementError::InvalidProtocolRevision,
            "runtime browser protocol revision must be a bounded ASCII metadata token",
        ),
        (
            BrowserProtocolRuntimeRequirementError::InvalidBrowserRevision,
            "runtime browser revision must be a bounded ASCII metadata token",
        ),
        (
            BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch,
            "runtime browser protocol revision does not match the pinned adapter revision",
        ),
        (
            BrowserProtocolRuntimeRequirementError::BrowserRevisionMismatch,
            "runtime browser revision does not match the pinned adapter browser revision",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
