#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesAdmissionError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        BrowserProtocolKind::WebDriverBiDi,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        BrowserProtocolCapability::SemanticObservation,
    )?)
}

#[test]
fn exhausted_locate_nodes_batch_does_not_consume_partial_node_authority()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(1);
    let origin = Origin::parse("https://app.example")?;
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
    let batch_query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), 2)?;

    assert_eq!(
        batch_query.bind_current_nodes(
            semantic_observation_proof()?,
            &mut registry,
            target,
            &[
                ("node", Some("shared-submit")),
                ("node", Some("shared-extra")),
            ],
        ),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::IdentifierSpaceExhausted,
        ))
    );

    let recovery_query =
        WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Recovery action"), 1)?;
    let handles = recovery_query.bind_current_nodes(
        semantic_observation_proof()?,
        &mut registry,
        target,
        &[("node", Some("shared-recovery"))],
    )?;

    assert_eq!(handles.len(), 1);
    assert_eq!(handles[0].node_id(), 1);
    Ok(())
}
