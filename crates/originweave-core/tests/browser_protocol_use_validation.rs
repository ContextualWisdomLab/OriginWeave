use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolUseValidationError, BrowserProtocolVersionRequirementError,
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
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?)
}

#[test]
fn validated_use_binds_all_required_adapter_metadata() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let validated = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
    )?;

    assert_eq!(validated.kind(), BrowserProtocolKind::WebDriverBiDi);
    assert_eq!(
        validated.originweave_protocol_version(),
        ORIGINWEAVE_PROTOCOL_VERSION
    );
    assert_eq!(validated.adapter_version(), ADAPTER_VERSION);
    assert_eq!(validated.protocol_revision(), PROTOCOL_REVISION);
    assert_eq!(validated.browser_revision(), BROWSER_REVISION);
    assert_eq!(
        validated.capability(),
        BrowserProtocolCapability::Navigation
    );
    Ok(())
}

#[test]
fn protocol_generation_mismatch_precedes_runtime_and_capability_checks()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let wrong_generation = OriginWeaveProtocolVersion::new(0, 2);

    assert_eq!(
        descriptor.validate_use(
            wrong_generation,
            "runtime revision with spaces",
            "browser/revision",
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::ProtocolVersion(
            BrowserProtocolVersionRequirementError::ProtocolVersionMismatch {
                required: wrong_generation,
                actual: ORIGINWEAVE_PROTOCOL_VERSION,
            }
        ))
    );
    Ok(())
}

#[test]
fn runtime_revision_validation_precedes_capability_check() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    assert_eq!(
        descriptor.validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            "webdriver-bidi-wd-2026-07-01",
            BROWSER_REVISION,
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::RuntimeRevision(
            originweave_core::BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch,
        ))
    );
    Ok(())
}

#[test]
fn undeclared_capability_cannot_produce_validated_use() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    assert_eq!(
        descriptor.validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            PROTOCOL_REVISION,
            BROWSER_REVISION,
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::Capability(
            originweave_core::BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                BrowserProtocolCapability::NetworkObservation,
            ),
        ))
    );
    Ok(())
}
