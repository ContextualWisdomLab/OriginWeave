use std::error::Error;

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolCapabilityRequirementError, BrowserProtocolKind,
    BrowserProtocolRuntimeRequirementError, BrowserProtocolUseValidationError,
    BrowserProtocolVersionRequirementError, OriginWeaveProtocolVersion,
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
        BrowserProtocolKind::WebDriverBiDi,
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
            BrowserProtocolKind::ChromeDevToolsProtocol,
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
fn runtime_protocol_kind_mismatch_precedes_revision_and_capability_checks()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    let error = descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::ChromeDevToolsProtocol,
        "runtime revision with spaces",
        "browser/revision",
        BrowserProtocolCapability::NetworkObservation,
    );

    assert_eq!(
        error,
        Err(BrowserProtocolUseValidationError::ProtocolKindMismatch {
            descriptor_kind: BrowserProtocolKind::WebDriverBiDi,
            runtime_kind: BrowserProtocolKind::ChromeDevToolsProtocol,
        })
    );
    let error = error.err().ok_or("expected protocol kind mismatch")?;
    assert_eq!(
        error.to_string(),
        "runtime browser protocol kind does not match the pinned adapter kind"
    );
    assert!(error.source().is_none());
    Ok(())
}

#[test]
fn runtime_revision_validation_precedes_capability_check() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    assert_eq!(
        descriptor.validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            BrowserProtocolKind::WebDriverBiDi,
            "webdriver-bidi-wd-2026-07-01",
            BROWSER_REVISION,
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::RuntimeRevision(
            BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch,
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
            BrowserProtocolKind::WebDriverBiDi,
            PROTOCOL_REVISION,
            BROWSER_REVISION,
            BrowserProtocolCapability::NetworkObservation,
        ),
        Err(BrowserProtocolUseValidationError::Capability(
            BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                BrowserProtocolCapability::NetworkObservation,
            ),
        ))
    );
    Ok(())
}

#[test]
fn validation_errors_preserve_stable_typed_sources() {
    let wrong_generation = OriginWeaveProtocolVersion::new(0, 2);
    let cases = [
        (
            BrowserProtocolUseValidationError::ProtocolVersion(
                BrowserProtocolVersionRequirementError::ProtocolVersionMismatch {
                    required: wrong_generation,
                    actual: ORIGINWEAVE_PROTOCOL_VERSION,
                },
            ),
            "browser protocol adapter targets originweave/0.1 but originweave/0.2 is required",
        ),
        (
            BrowserProtocolUseValidationError::RuntimeRevision(
                BrowserProtocolRuntimeRequirementError::ProtocolRevisionMismatch,
            ),
            "runtime browser protocol revision does not match the pinned adapter revision",
        ),
        (
            BrowserProtocolUseValidationError::Capability(
                BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                    BrowserProtocolCapability::NetworkObservation,
                ),
            ),
            "browser protocol adapter does not declare required network-observation capability",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some(expected)
        );
    }
}
