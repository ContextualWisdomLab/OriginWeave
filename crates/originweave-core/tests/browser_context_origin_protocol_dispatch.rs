use std::{cell::Cell, error::Error, io};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolRuntimeMetadata, BrowserProtocolUseValidationError, BrowserRegistryError,
    DocumentEpoch, Origin, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

type DispatchOutcome = Result<(u64, BrowserProtocolCapability), &'static str>;
type DispatchFn = fn(ValidatedBrowserProtocolUse, DocumentEpoch) -> DispatchOutcome;

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
        &[BrowserProtocolCapability::SemanticObservation],
    )?)
}

fn runtime_metadata(adapter_version: &str) -> BrowserProtocolRuntimeMetadata<'_> {
    BrowserProtocolRuntimeMetadata::new(
        BrowserProtocolKind::WebDriverBiDi,
        adapter_version,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
    )
}

fn origin(value: &str) -> Result<Origin, Box<dyn Error>> {
    Origin::parse(value).map_err(|_| {
        Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid controlled origin fixture",
        )) as Box<dyn Error>
    })
}

fn reset_dispatch_marker() {
    DISPATCH_CALLED.with(|called| called.set(false));
}

fn dispatch_was_called() -> bool {
    DISPATCH_CALLED.with(Cell::get)
}

fn successful_dispatch(
    validated: ValidatedBrowserProtocolUse,
    current_epoch: DocumentEpoch,
) -> DispatchOutcome {
    DISPATCH_CALLED.with(|called| called.set(true));
    Ok((current_epoch.value(), validated.capability()))
}

#[test]
fn exact_current_origin_and_protocol_metadata_gate_one_dispatch_call() -> Result<(), Box<dyn Error>>
{
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let expected_origin = origin("https://app.example")?;
    registry.bind_context_origin(session, context, &expected_origin)?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_origin_current(
        &registry,
        BrowserContextDispatchTarget::new(session, context),
        &expected_origin,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(ADAPTER_VERSION),
        BrowserProtocolCapability::SemanticObservation,
        successful_dispatch as DispatchFn,
    )?;

    assert!(dispatch_was_called());
    assert_eq!(
        result,
        Ok((1, BrowserProtocolCapability::SemanticObservation))
    );
    Ok(())
}

#[test]
fn origin_mismatch_or_unbound_origin_fails_before_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let expected_origin = origin("https://app.example")?;
    let other_origin = origin("https://other.example")?;
    registry.bind_context_origin(session, context, &expected_origin)?;

    reset_dispatch_marker();
    assert_eq!(
        descriptor.dispatch_if_context_origin_current(
            &registry,
            BrowserContextDispatchTarget::new(session, context),
            &other_origin,
            ORIGINWEAVE_PROTOCOL_VERSION,
            runtime_metadata(ADAPTER_VERSION),
            BrowserProtocolCapability::SemanticObservation,
            successful_dispatch as DispatchFn,
        ),
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(
            BrowserRegistryError::OriginChangedWithoutDocumentAdvance
        ))
    );
    assert!(!dispatch_was_called());

    registry.advance_document(context)?;
    reset_dispatch_marker();
    assert_eq!(
        descriptor.dispatch_if_context_origin_current(
            &registry,
            BrowserContextDispatchTarget::new(session, context),
            &expected_origin,
            ORIGINWEAVE_PROTOCOL_VERSION,
            runtime_metadata(ADAPTER_VERSION),
            BrowserProtocolCapability::SemanticObservation,
            successful_dispatch as DispatchFn,
        ),
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(
            BrowserRegistryError::ContextOriginNotBound
        ))
    );
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn protocol_mismatch_after_origin_revalidation_still_prevents_dispatch()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let expected_origin = origin("https://app.example")?;
    registry.bind_context_origin(session, context, &expected_origin)?;
    reset_dispatch_marker();

    assert_eq!(
        descriptor.dispatch_if_context_origin_current(
            &registry,
            BrowserContextDispatchTarget::new(session, context),
            &expected_origin,
            ORIGINWEAVE_PROTOCOL_VERSION,
            runtime_metadata("originweave-bidi-v2"),
            BrowserProtocolCapability::SemanticObservation,
            successful_dispatch as DispatchFn,
        ),
        Err(BrowserContextProtocolDispatchError::ProtocolValidation(
            BrowserProtocolUseValidationError::AdapterVersionMismatch
        ))
    );
    assert!(!dispatch_was_called());
    Ok(())
}
