use std::{cell::Cell, error::Error};

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowserProtocolRuntimeMetadata, BrowserProtocolUseValidationError, BrowserRegistryError,
    BrowserSessionId, BrowsingContextId, DocumentEpoch, OriginWeaveProtocolVersion,
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
        &[BrowserProtocolCapability::Navigation],
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

fn target(
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
) -> BrowserContextDispatchTarget {
    BrowserContextDispatchTarget::new(browser_session, browsing_context)
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
fn context_dispatch_target_preserves_requested_ids_without_granting_authority()
-> Result<(), Box<dyn Error>> {
    let session = BrowserSessionId::new(7)?;
    let context = BrowsingContextId::new(11)?;
    let target = target(session, context);

    assert_eq!(target.browser_session(), session);
    assert_eq!(target.browsing_context(), context);
    Ok(())
}

#[test]
fn exact_context_and_runtime_metadata_gate_one_dispatch_call() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_current(
        &registry,
        target(session, context),
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(ADAPTER_VERSION),
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    )?;

    assert!(dispatch_was_called());
    assert_eq!(result, Ok((1, BrowserProtocolCapability::Navigation)));

    registry.advance_document(context)?;
    reset_dispatch_marker();
    let next = descriptor.dispatch_if_context_current(
        &registry,
        target(session, context),
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(ADAPTER_VERSION),
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    )?;
    assert!(dispatch_was_called());
    assert_eq!(next, Ok((2, BrowserProtocolCapability::Navigation)));
    Ok(())
}

#[test]
fn cross_session_context_reuse_fails_before_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "top-level-context")?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_current(
        &registry,
        target(attacker, context),
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(ADAPTER_VERSION),
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    );

    assert_eq!(
        result,
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(
            BrowserRegistryError::ContextSessionMismatch {
                expected: owner,
                actual: attacker,
            }
        ))
    );
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn unknown_session_or_context_fails_before_dispatch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let unknown_session = BrowserSessionId::new(999)?;
    let unknown_context = BrowsingContextId::new(999)?;

    reset_dispatch_marker();
    assert_eq!(
        descriptor.dispatch_if_context_current(
            &registry,
            target(unknown_session, context),
            ORIGINWEAVE_PROTOCOL_VERSION,
            runtime_metadata(ADAPTER_VERSION),
            BrowserProtocolCapability::Navigation,
            successful_dispatch as DispatchFn,
        ),
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(
            BrowserRegistryError::UnknownBrowserSession
        ))
    );
    assert!(!dispatch_was_called());

    assert_eq!(
        descriptor.dispatch_if_context_current(
            &registry,
            target(session, unknown_context),
            ORIGINWEAVE_PROTOCOL_VERSION,
            runtime_metadata(ADAPTER_VERSION),
            BrowserProtocolCapability::Navigation,
            successful_dispatch as DispatchFn,
        ),
        Err(BrowserContextProtocolDispatchError::BrowserAuthority(
            BrowserRegistryError::UnknownBrowsingContext
        ))
    );
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn protocol_mismatch_after_context_validation_still_prevents_dispatch() -> Result<(), Box<dyn Error>>
{
    let descriptor = descriptor()?;
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    reset_dispatch_marker();

    let result = descriptor.dispatch_if_context_current(
        &registry,
        target(session, context),
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata("originweave-bidi-v2"),
        BrowserProtocolCapability::Navigation,
        successful_dispatch as DispatchFn,
    );

    assert_eq!(
        result,
        Err(BrowserContextProtocolDispatchError::ProtocolValidation(
            BrowserProtocolUseValidationError::AdapterVersionMismatch
        ))
    );
    assert!(!dispatch_was_called());
    Ok(())
}

#[test]
fn context_protocol_dispatch_errors_preserve_typed_sources() {
    let authority = BrowserContextProtocolDispatchError::BrowserAuthority(
        BrowserRegistryError::UnknownBrowsingContext,
    );
    assert!(authority.source().is_some());
    assert_eq!(
        authority.to_string(),
        "browser context authority denied protocol dispatch: browsing context is not registered in this authority registry"
    );

    let protocol = BrowserContextProtocolDispatchError::ProtocolValidation(
        BrowserProtocolUseValidationError::AdapterVersionMismatch,
    );
    assert!(protocol.source().is_some());
    assert_eq!(
        protocol.to_string(),
        "browser protocol validation denied context dispatch: runtime browser adapter version does not match the pinned adapter version"
    );
}
