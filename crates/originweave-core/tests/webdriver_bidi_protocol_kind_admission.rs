#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesAdmissionError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const CDP_ADAPTER_VERSION: &str = "originweave-cdp-v1";
const CDP_PROTOCOL_REVISION: &str = "cdp-pdl-2026-08-17";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn cdp_semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::ChromeDevToolsProtocol,
        ORIGINWEAVE_PROTOCOL_VERSION,
        CDP_ADAPTER_VERSION,
        CDP_PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::ChromeDevToolsProtocol,
        CDP_ADAPTER_VERSION,
        CDP_PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::SemanticObservation,
    )?)
}

#[test]
fn webdriver_bidi_locate_nodes_rejects_cdp_semantic_observation_proof() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = Origin::parse("https://app.example").expect("valid controlled fixture origin");
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let epoch = registry.bind_context_origin(session, context, &origin)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task text"), 1)?;

    let error = query
        .bind_current_nodes(
            cdp_semantic_observation_proof()?,
            &mut registry,
            target,
            &[("node", Some("shared-task-text"))],
        )
        .expect_err("CDP proof must not authorize WebDriver BiDi locateNodes admission");
    assert_eq!(
        error,
        WebDriverBiDiLocateNodesAdmissionError::UnsupportedProtocolKind(
            BrowserProtocolKind::ChromeDevToolsProtocol,
        )
    );
    assert_eq!(
        error.to_string(),
        "locateNodes admission requires a WebDriverBiDi protocol-use proof, not ChromeDevToolsProtocol"
    );
    assert!(error.source().is_none());
    Ok(())
}
