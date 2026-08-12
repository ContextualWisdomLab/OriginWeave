use std::{cell::Cell, error::Error};

use originweave_core::{
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolUseValidationError, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type DispatchOutcome = Result<(String, BrowserProtocolCapability), &'static str>;
type DispatchFn = fn(ValidatedBrowserProtocolUse) -> DispatchOutcome;

thread_local! {
    static DISPATCH_CALLED: Cell<bool> = const { Cell::new(false) };
}

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

fn reset_dispatch_marker() {
    DISPATCH_CALLED.with(|called| called.set(false));
}

fn dispatch_was_called() -> bool {
    DISPATCH_CALLED.with(Cell::get)
}

fn successful_dispatch(validated: ValidatedBrowserProtocolUse) -> DispatchOutcome {
    DISPATCH_CALLED.with(|called| called.set(true));
    Ok((
        validated.adapter_version().to_owned(),
        validated.capability(),
    ))
}

fn failing_dispatch(_: ValidatedBrowserProtocolUse) -> DispatchOutcome {
    DISPATCH_CALLED.with(|called| called.set(true));
    Err("adapter-failure")
}

#[test]
fn exact_runtime_validation_hands_single_use_proof_to_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    reset_dispatch_marker();

    let dispatch_result = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    )?;

    assert!(dispatch_was_called());
    assert_eq!(
        dispatch_result,
        Ok((
            ADAPTER_VERSION.to_owned(),
            BrowserProtocolCapability::Navigation
        ))
    );
    Ok(())
}

#[test]
fn runtime_mismatch_prevents_dispatch_callback() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        "originweave-bidi-v2",
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    );

    assert_eq!(
        result,
        Err(BrowserProtocolUseValidationError::AdapterVersionMismatch)
    );
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn adapter_callback_failure_remains_separate_after_validation() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    reset_dispatch_marker();

    let dispatch_result = descriptor.dispatch_if_runtime_matches(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::Navigation,
        failing_dispatch as DispatchFn,
    )?;

    assert!(dispatch_was_called());
    assert_eq!(dispatch_result, Err("adapter-failure"));
    Ok(())
}
