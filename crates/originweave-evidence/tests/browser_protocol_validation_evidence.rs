use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    OriginWeaveProtocolVersion,
};
use originweave_evidence::BrowserProtocolValidationEvidence;

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

#[test]
fn records_exact_metadata_from_one_validated_browser_protocol_use() -> Result<(), Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    let validated = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::SemanticObservation,
    )?;

    let evidence = BrowserProtocolValidationEvidence::from_validated_use(&validated);

    assert_eq!(evidence.kind(), BrowserProtocolKind::WebDriverBiDi);
    assert_eq!(
        evidence.originweave_protocol_version(),
        ORIGINWEAVE_PROTOCOL_VERSION
    );
    assert_eq!(evidence.adapter_version(), ADAPTER_VERSION);
    assert_eq!(evidence.protocol_revision(), PROTOCOL_REVISION);
    assert_eq!(evidence.browser_revision(), BROWSER_REVISION);
    assert_eq!(
        evidence.capability(),
        BrowserProtocolCapability::SemanticObservation
    );
    Ok(())
}

#[test]
fn evidence_is_owned_audit_metadata_not_reusable_validation_authority() -> Result<(), Box<dyn Error>>
{
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::ChromeDevToolsProtocol,
        ORIGINWEAVE_PROTOCOL_VERSION,
        "originweave-cdp-v1",
        "cdp-1-3-r1639810",
        BROWSER_REVISION,
        &[BrowserProtocolCapability::NetworkObservation],
    )?;
    let validated = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::ChromeDevToolsProtocol,
        "cdp-1-3-r1639810",
        BROWSER_REVISION,
        BrowserProtocolCapability::NetworkObservation,
    )?;

    let evidence = BrowserProtocolValidationEvidence::from_validated_use(&validated);
    let cloned = evidence.clone();

    assert_eq!(cloned, evidence);
    assert_eq!(cloned.kind(), BrowserProtocolKind::ChromeDevToolsProtocol);
    assert_eq!(
        cloned.capability(),
        BrowserProtocolCapability::NetworkObservation
    );
    Ok(())
}
