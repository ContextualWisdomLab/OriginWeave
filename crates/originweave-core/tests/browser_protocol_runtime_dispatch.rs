use std::{cell::Cell, error::Error};

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
fn exact_runtime_validation_hands_single_use_proof_to_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let called = Cell::new(false);

    let output = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        |validated| {
            called.set(true);
            (
                validated.adapter_version().to_owned(),
                validated.capability(),
            )
        },
    )?;

    assert!(called.get());
    assert_eq!(output.0, ADAPTER_VERSION);
    assert_eq!(output.1, BrowserProtocolCapability::Navigation);
    Ok(())
}

#[test]
fn runtime_mismatch_prevents_dispatch_callback() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let called = Cell::new(false);

    let result = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        "originweave-bidi-v2",
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        |_| {
            called.set(true);
            "dispatched"
        },
    );

    assert_eq!(
        result,
        Err(BrowserProtocolUseValidationError::AdapterVersionMismatch)
    );
    assert!(!called.get());
    Ok(())
}

#[test]
fn adapter_callback_failure_remains_separate_after_validation() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;

    let dispatch_result = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        |_| Err::<(), _>("adapter-failure"),
    )?;

    assert_eq!(dispatch_result, Err("adapter-failure"));
    Ok(())
}
