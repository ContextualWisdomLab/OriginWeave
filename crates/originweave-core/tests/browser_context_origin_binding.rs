use std::error::Error;
use std::io;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin, OriginWeaveProtocolVersion,
    ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiLocateNodesAdmissionError, WebDriverBiDiRemoteNodeReferenceError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn first_origin() -> Result<Origin, Box<dyn Error>> {
    Origin::parse("http://127.0.0.1:43127")
        .map_err(|_error| io::Error::other("controlled first origin must be valid").into())
}

fn second_origin() -> Result<Origin, Box<dyn Error>> {
    Origin::parse("http://localhost:43127")
        .map_err(|_error| io::Error::other("controlled second origin must be valid").into())
}

fn semantic_observation_proof() -> ValidatedBrowserProtocolUse {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        BrowserProtocolKind::WebDriverBiDi,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[BrowserProtocolCapability::SemanticObservation],
    )
    .expect("valid semantic-observation descriptor");
    descriptor
        .validate_use(
            ORIGINWEAVE_PROTOCOL_VERSION,
            BrowserProtocolKind::WebDriverBiDi,
            ADAPTER_VERSION,
            PROTOCOL_REVISION,
            BROWSER_REVISION,
            BrowserProtocolCapability::SemanticObservation,
        )
        .expect("valid semantic-observation proof")
}

fn bind_observed_node(
    registry: &mut BrowserAuthorityRegistry,
    browser_session: BrowserSessionId,
    browsing_context: BrowsingContextId,
    origin: &Origin,
    external_identifier: &str,
) -> Result<ObservedNodeHandle, WebDriverBiDiLocateNodesAdmissionError> {
    let epoch = registry
        .require_context_origin(browser_session, browsing_context, origin)
        .map_err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(browser_session, browsing_context),
            origin,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("generic"), None, 1)
        .expect("valid bounded semantic-node query");
    query
        .bind_current_nodes(
            semantic_observation_proof(),
            registry,
            target,
            &[("node", Some(external_identifier))],
        )?
        .into_iter()
        .next()
        .ok_or(WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
        ))
}

#[test]
fn context_origin_can_be_bound_before_node_discovery() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = first_origin()?;

    let epoch = registry.bind_context_origin(session, context, &origin)?;
    assert_eq!(epoch, DocumentEpoch::new(1)?);
    assert_eq!(
        registry.bind_context_origin(session, context, &origin)?,
        epoch
    );

    let node = bind_observed_node(&mut registry, session, context, &origin, "backend-node-17")?;
    assert_eq!(node.document_epoch(), epoch);
    assert_eq!(node.origin(), &origin);
    Ok(())
}

#[test]
fn context_origin_change_requires_document_rotation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first = first_origin()?;
    let second = second_origin()?;

    registry.bind_context_origin(session, context, &first)?;
    assert_eq!(
        registry.bind_context_origin(session, context, &second),
        Err(BrowserRegistryError::OriginChangedWithoutDocumentAdvance)
    );

    let next_epoch = registry.advance_document(context)?;
    assert_eq!(next_epoch, DocumentEpoch::new(2)?);
    assert_eq!(
        registry.bind_context_origin(session, context, &second)?,
        next_epoch
    );
    Ok(())
}

#[test]
fn context_origin_binding_rejects_cross_session_and_unknown_authority() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "top-level-context")?;
    let origin = first_origin()?;

    assert_eq!(
        registry.bind_context_origin(attacker, context, &origin),
        Err(BrowserRegistryError::ContextSessionMismatch {
            expected: owner,
            actual: attacker,
        })
    );

    let unknown_session = BrowserSessionId::new(999)?;
    assert_eq!(
        registry.bind_context_origin(unknown_session, context, &origin),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let unknown_context = BrowsingContextId::new(999)?;
    assert_eq!(
        registry.bind_context_origin(owner, unknown_context, &origin),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    Ok(())
}
