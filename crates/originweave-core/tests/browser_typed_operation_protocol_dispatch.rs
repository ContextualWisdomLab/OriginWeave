use std::{cell::Cell, error::Error, io};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolOperation, BrowserProtocolRuntimeMetadata, DocumentEpoch, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn descriptor(
    capabilities: &[BrowserProtocolCapability],
) -> Result<BrowserProtocolAdapterDescriptor, Box<dyn Error>> {
    Ok(BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        capabilities,
    )?)
}

fn runtime_metadata() -> BrowserProtocolRuntimeMetadata<'static> {
    BrowserProtocolRuntimeMetadata::new(
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
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

fn typed_input_target<'a>(
    registry: &mut BrowserAuthorityRegistry,
    expected_origin: &'a Origin,
) -> Result<BrowserContextOriginEpochDispatchTarget<'a>, Box<dyn Error>> {
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let epoch = registry.bind_context_origin(session, context, expected_origin)?;
    Ok(BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            expected_origin,
        ),
        epoch,
    ))
}

#[test]
fn typed_operations_map_to_exact_transport_capabilities() {
    assert_eq!(
        BrowserProtocolOperation::Navigate.required_capability(),
        BrowserProtocolCapability::Navigation
    );
    assert_eq!(
        BrowserProtocolOperation::ObserveSemantics.required_capability(),
        BrowserProtocolCapability::SemanticObservation
    );
    assert_eq!(
        BrowserProtocolOperation::DispatchTypedInput.required_capability(),
        BrowserProtocolCapability::TypedInput
    );
    assert_eq!(
        BrowserProtocolOperation::ObserveNetwork.required_capability(),
        BrowserProtocolCapability::NetworkObservation
    );
}

#[test]
fn typed_operation_dispatch_derives_the_required_capability() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::TypedInput])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = origin("https://app.example")?;
    let target = typed_input_target(&mut registry, &expected_origin)?;

    let result = descriptor.dispatch_operation_if_context_origin_epoch_current(
        &registry,
        target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        BrowserProtocolOperation::DispatchTypedInput,
        |validated: ValidatedBrowserProtocolUse, operation, epoch: DocumentEpoch| {
            (
                operation,
                validated.capability(),
                epoch.value(),
            )
        },
    )?;

    assert_eq!(
        result,
        (
            BrowserProtocolOperation::DispatchTypedInput,
            BrowserProtocolCapability::TypedInput,
            1,
        )
    );
    Ok(())
}

#[test]
fn unsupported_typed_operation_fails_before_dispatch_callback() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::SemanticObservation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = origin("https://app.example")?;
    let target = typed_input_target(&mut registry, &expected_origin)?;
    let dispatch_called = Cell::new(false);

    let result = descriptor.dispatch_operation_if_context_origin_epoch_current(
        &registry,
        target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        BrowserProtocolOperation::DispatchTypedInput,
        |_validated, _operation, _epoch| dispatch_called.set(true),
    );

    assert!(matches!(
        result,
        Err(BrowserContextProtocolDispatchError::ProtocolValidation(_))
    ));
    assert!(!dispatch_called.get());
    Ok(())
}
