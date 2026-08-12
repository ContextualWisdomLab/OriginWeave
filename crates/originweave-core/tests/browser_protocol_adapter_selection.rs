use std::error::Error;

use originweave_core::{
    select_browser_protocol_adapter, BrowserProtocolAdapterDescriptor,
    BrowserProtocolAdapterSelectionError, BrowserProtocolCapability, BrowserProtocolKind,
    OriginWeaveProtocolVersion,
};

const CURRENT_PROTOCOL: OriginWeaveProtocolVersion = OriginWeaveProtocolVersion::new(0, 1);
const FUTURE_PROTOCOL: OriginWeaveProtocolVersion = OriginWeaveProtocolVersion::new(0, 2);
const BROWSER_REVISION: &str = "chromium-r1639810";

fn bidi_descriptor(
    version: OriginWeaveProtocolVersion,
    capabilities: &[BrowserProtocolCapability],
) -> Result<BrowserProtocolAdapterDescriptor, Box<dyn Error>> {
    Ok(BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        version,
        "originweave-bidi-v1",
        "webdriver-bidi-wd-2026-06-01",
        BROWSER_REVISION,
        capabilities,
    )?)
}

fn cdp_descriptor(
    version: OriginWeaveProtocolVersion,
    capabilities: &[BrowserProtocolCapability],
) -> Result<BrowserProtocolAdapterDescriptor, Box<dyn Error>> {
    Ok(BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::ChromeDevToolsProtocol,
        version,
        "originweave-cdp-v1",
        "cdp-browser-r1639810",
        BROWSER_REVISION,
        capabilities,
    )?)
}

#[test]
fn standards_track_bidi_is_preferred_when_it_satisfies_the_contract() -> Result<(), Box<dyn Error>> {
    let bidi = bidi_descriptor(
        CURRENT_PROTOCOL,
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?;
    let cdp = cdp_descriptor(
        CURRENT_PROTOCOL,
        &[
            BrowserProtocolCapability::Navigation,
            BrowserProtocolCapability::TypedInput,
        ],
    )?;

    let selected = select_browser_protocol_adapter(
        CURRENT_PROTOCOL,
        BrowserProtocolCapability::TypedInput,
        &bidi,
        Some(&cdp),
    )?;

    assert!(std::ptr::eq(selected, &bidi));
    assert_eq!(selected.kind(), BrowserProtocolKind::WebDriverBiDi);
    Ok(())
}

#[test]
fn pinned_cdp_is_selected_only_when_bidi_cannot_satisfy_the_same_contract() -> Result<(), Box<dyn Error>> {
    let bidi = bidi_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::Navigation])?;
    let cdp = cdp_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::TypedInput])?;

    let selected = select_browser_protocol_adapter(
        CURRENT_PROTOCOL,
        BrowserProtocolCapability::TypedInput,
        &bidi,
        Some(&cdp),
    )?;

    assert!(std::ptr::eq(selected, &cdp));
    assert_eq!(
        selected.kind(),
        BrowserProtocolKind::ChromeDevToolsProtocol
    );
    Ok(())
}

#[test]
fn exact_fallback_version_can_replace_an_incompatible_bidi_generation() -> Result<(), Box<dyn Error>> {
    let bidi = bidi_descriptor(FUTURE_PROTOCOL, &[BrowserProtocolCapability::TypedInput])?;
    let cdp = cdp_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::TypedInput])?;

    let selected = select_browser_protocol_adapter(
        CURRENT_PROTOCOL,
        BrowserProtocolCapability::TypedInput,
        &bidi,
        Some(&cdp),
    )?;

    assert!(std::ptr::eq(selected, &cdp));
    assert_eq!(selected.originweave_protocol_version(), CURRENT_PROTOCOL);
    Ok(())
}

#[test]
fn no_exact_compatible_adapter_fails_closed() -> Result<(), Box<dyn Error>> {
    let bidi = bidi_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::Navigation])?;
    let wrong_generation_cdp = cdp_descriptor(
        FUTURE_PROTOCOL,
        &[BrowserProtocolCapability::TypedInput],
    )?;

    assert_eq!(
        select_browser_protocol_adapter(
            CURRENT_PROTOCOL,
            BrowserProtocolCapability::TypedInput,
            &bidi,
            Some(&wrong_generation_cdp),
        ),
        Err(BrowserProtocolAdapterSelectionError::NoCompatibleAdapter {
            required_version: CURRENT_PROTOCOL,
            required_capability: BrowserProtocolCapability::TypedInput,
        })
    );
    assert_eq!(
        select_browser_protocol_adapter(
            CURRENT_PROTOCOL,
            BrowserProtocolCapability::TypedInput,
            &bidi,
            None,
        ),
        Err(BrowserProtocolAdapterSelectionError::NoCompatibleAdapter {
            required_version: CURRENT_PROTOCOL,
            required_capability: BrowserProtocolCapability::TypedInput,
        })
    );
    Ok(())
}

#[test]
fn adapter_roles_are_not_reinterpreted_from_caller_order() -> Result<(), Box<dyn Error>> {
    let cdp = cdp_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::TypedInput])?;
    let bidi = bidi_descriptor(CURRENT_PROTOCOL, &[BrowserProtocolCapability::TypedInput])?;

    assert_eq!(
        select_browser_protocol_adapter(
            CURRENT_PROTOCOL,
            BrowserProtocolCapability::TypedInput,
            &cdp,
            Some(&bidi),
        ),
        Err(BrowserProtocolAdapterSelectionError::PreferredAdapterMustBeBiDi)
    );
    assert_eq!(
        select_browser_protocol_adapter(
            CURRENT_PROTOCOL,
            BrowserProtocolCapability::TypedInput,
            &bidi,
            Some(&bidi),
        ),
        Err(BrowserProtocolAdapterSelectionError::FallbackAdapterMustBeCdp)
    );
    Ok(())
}

#[test]
fn selection_errors_are_stable_and_source_free() {
    let no_match = BrowserProtocolAdapterSelectionError::NoCompatibleAdapter {
        required_version: CURRENT_PROTOCOL,
        required_capability: BrowserProtocolCapability::NetworkObservation,
    };
    assert_eq!(
        no_match.to_string(),
        "no browser protocol adapter satisfies originweave/0.1 with required network-observation capability"
    );
    assert!(no_match.source().is_none());

    let preferred_kind = BrowserProtocolAdapterSelectionError::PreferredAdapterMustBeBiDi;
    assert_eq!(
        preferred_kind.to_string(),
        "preferred browser protocol adapter must use WebDriver BiDi"
    );
    assert!(preferred_kind.source().is_none());

    let fallback_kind = BrowserProtocolAdapterSelectionError::FallbackAdapterMustBeCdp;
    assert_eq!(
        fallback_kind.to_string(),
        "fallback browser protocol adapter must use Chrome DevTools Protocol"
    );
    assert!(fallback_kind.source().is_none());
}
