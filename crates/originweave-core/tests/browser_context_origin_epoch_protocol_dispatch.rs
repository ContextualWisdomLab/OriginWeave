use std::{cell::Cell, error::Error, io};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolRuntimeMetadata, DocumentEpoch, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse,
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
        &[BrowserProtocolCapability::TypedInput],
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
fn exact_context_origin_epoch_and_protocol_metadata_gate_one_dispatch_call()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let expected_origin = origin("https://app.example")?;
    let expected_epoch = registry.bind_context_origin(session, context, &expected_origin)?;
    let context_origin = BrowserContextOriginDispatchTarget::new(
        BrowserContextDispatchTarget::new(session, context),
        &expected_origin,
    );
    let target = BrowserContextOriginEpochDispatchTarget::new(context_origin, expected_epoch);

    assert_eq!(target.context_origin(), context_origin);
    assert_eq!(target.expected_epoch(), expected_epoch);
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_origin_epoch_current(
        &registry,
        target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        BrowserProtocolCapability::TypedInput,
        successful_dispatch as DispatchFn,
    )?;

    assert!(dispatch_was_called());
    assert_eq!(result, Ok((1, BrowserProtocolCapability::TypedInput)));
    Ok(())
}

#[test]
fn same_origin_new_document_epoch_fails_before_protocol_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let expected_origin = origin("https://app.example")?;
    let observed_epoch = registry.bind_context_origin(session, context, &expected_origin)?;
    let context_origin = BrowserContextOriginDispatchTarget::new(
        BrowserContextDispatchTarget::new(session, context),
        &expected_origin,
    );
    let target = BrowserContextOriginEpochDispatchTarget::new(context_origin, observed_epoch);

    let current_epoch = registry.advance_document(context)?;
    registry.bind_context_origin(session, context, &expected_origin)?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_origin_epoch_current(
        &registry,
        target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        BrowserProtocolCapability::TypedInput,
        successful_dispatch as DispatchFn,
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => {
            return Err(Box::new(io::Error::other(
                "stale document epoch unexpectedly dispatched",
            )))
        }
    };

    assert_eq!(
        error,
        BrowserContextProtocolDispatchError::DocumentEpochMismatch {
            expected: observed_epoch,
            current: current_epoch,
        }
    );
    assert_eq!(
        error.to_string(),
        "browser document epoch 2 no longer matches observed epoch 1"
    );
    assert!(error.source().is_none());
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn epoch_dispatch_preserves_authority_and_protocol_failures_before_callback()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let current_origin = origin("https://app.example")?;
    let other_origin = origin("https://other.example")?;
    let expected_epoch = registry.bind_context_origin(session, context, &current_origin)?;

    let wrong_origin_target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &other_origin,
        ),
        expected_epoch,
    );
    reset_dispatch_marker();
    let authority_result = descriptor.dispatch_if_context_origin_epoch_current(
        &registry,
        wrong_origin_target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        BrowserProtocolCapability::TypedInput,
        successful_dispatch as DispatchFn,
    );
    assert!(matches!(
        authority_result,
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(_))
    ));
    assert!(!dispatch_was_called());

    let current_target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &current_origin,
        ),
        expected_epoch,
    );
    let drifted_runtime = BrowserProtocolRuntimeMetadata::new(
        BrowserProtocolKind::WebDriverBiDi,
        "originweave-bidi-v2",
        PROTOCOL_REVISION,
        BROWSER_REVISION,
    );
    reset_dispatch_marker();
    let protocol_result = descriptor.dispatch_if_context_origin_epoch_current(
        &registry,
        current_target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        drifted_runtime,
        BrowserProtocolCapability::TypedInput,
        successful_dispatch as DispatchFn,
    );
    assert!(matches!(
        protocol_result,
        Err(BrowserContextProtocolDispatchError::ProtocolValidation(_))
    ));
    assert!(!dispatch_was_called());
    Ok(())
}
