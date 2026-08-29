#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, NodeHandleError,
    ObservedNodeHandle, Origin, OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesAdmissionError,
    WebDriverBiDiRemoteNodeReferenceError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn loopback_origin() -> Origin {
    Origin::parse("http://127.0.0.1:43127").expect("valid loopback fixture origin")
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
        .bind_context_origin(browser_session, browsing_context, origin)
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
fn external_protocol_identifiers_are_scoped_and_never_become_authority()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();

    let first_session = registry.register_session("webdriver-session-A")?;
    let repeated_session = registry.register_session("webdriver-session-A")?;
    let second_session = registry.register_session("webdriver-session-B")?;

    assert_eq!(first_session, repeated_session);
    assert_ne!(first_session, second_session);

    let first_context = registry.register_context(first_session, "frame-root")?;
    let repeated_context = registry.register_context(first_session, "frame-root")?;
    let second_context = registry.register_context(second_session, "frame-root")?;

    assert_eq!(first_context, repeated_context);
    assert_ne!(first_context, second_context);
    assert_eq!(
        registry.current_epoch(first_context)?,
        DocumentEpoch::new(1)?
    );
    Ok(())
}

#[test]
fn public_default_and_error_contracts_are_usable_from_an_adapter() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::default();
    assert!(registry.register_session("adapter-session")?.value() > 0);

    let first_session = BrowserSessionId::new(1)?;
    let second_session = BrowserSessionId::new(2)?;
    let cases = [
        (
            BrowserRegistryError::InvalidExternalIdentifier,
            "external browser identifier must contain 1 to 512 UTF-8 bytes without control, whitespace, or Unicode format characters".to_owned(),
        ),
        (
            BrowserRegistryError::UnknownBrowserSession,
            "browser session is not registered in this authority registry".to_owned(),
        ),
        (
            BrowserRegistryError::UnknownBrowsingContext,
            "browsing context is not registered in this authority registry".to_owned(),
        ),
        (
            BrowserRegistryError::ContextSessionMismatch {
                expected: first_session,
                actual: second_session,
            },
            "browsing context belongs to session 1, not session 2".to_owned(),
        ),
        (
            BrowserRegistryError::OriginChangedWithoutDocumentAdvance,
            "browsing context origin changed without advancing the document epoch".to_owned(),
        ),
        (
            BrowserRegistryError::IdentifierSpaceExhausted,
            "browser authority identifier space is exhausted".to_owned(),
        ),
        (
            BrowserRegistryError::DocumentEpochExhausted,
            "browser document epoch space is exhausted".to_owned(),
        ),
        (
            BrowserRegistryError::InternalAuthorityInvariant,
            "browser authority registry violated a nonzero invariant".to_owned(),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
    Ok(())
}

#[test]
fn document_rotation_invalidates_old_external_node_bindings() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = loopback_origin();

    let first = bind_observed_node(&mut registry, session, context, &origin, "backend-node-17")?;
    let same = bind_observed_node(&mut registry, session, context, &origin, "backend-node-17")?;
    assert_eq!(first.node_id(), same.node_id());

    let next_epoch = registry.advance_document(context)?;
    assert_eq!(next_epoch.value(), 2);
    assert_eq!(
        first.validate_current(session, context, &origin, next_epoch),
        Err(NodeHandleError::StaleDocumentEpoch {
            observed: first.document_epoch(),
            current: next_epoch,
        })
    );

    let rebound = bind_observed_node(&mut registry, session, context, &origin, "backend-node-17")?;
    assert_eq!(rebound.document_epoch(), next_epoch);
    assert_ne!(first.node_id(), rebound.node_id());
    Ok(())
}

#[test]
fn retired_context_and_session_authority_cannot_be_reused() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = loopback_origin();
    let first_node =
        bind_observed_node(&mut registry, session, context, &origin, "backend-node-17")?;

    registry.remove_context(context)?;
    assert_eq!(
        registry.current_epoch(context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    assert_eq!(
        bind_observed_node(&mut registry, session, context, &origin, "backend-node-17"),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::UnknownBrowsingContext
        ))
    );
    assert_eq!(
        registry.remove_context(context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );

    let replacement_context = registry.register_context(session, "top-level-context")?;
    assert_ne!(replacement_context, context);
    let replacement_node = bind_observed_node(
        &mut registry,
        session,
        replacement_context,
        &origin,
        "backend-node-17",
    )?;
    assert_ne!(replacement_node.node_id(), first_node.node_id());

    registry.remove_session(session)?;
    assert_eq!(
        registry.register_context(session, "after-session-retirement"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );
    assert_eq!(
        registry.current_epoch(replacement_context),
        Err(BrowserRegistryError::UnknownBrowsingContext)
    );
    assert_eq!(
        registry.remove_session(session),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let replacement_session = registry.register_session("webdriver-session")?;
    assert_ne!(replacement_session, session);
    let next_context = registry.register_context(replacement_session, "top-level-context")?;
    assert_ne!(next_context, replacement_context);
    Ok(())
}

#[test]
fn context_cannot_be_reused_by_another_session() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let owner = registry.register_session("owner-session")?;
    let attacker = registry.register_session("attacker-session")?;
    let context = registry.register_context(owner, "shared-looking-context")?;
    let origin = loopback_origin();

    assert_eq!(
        bind_observed_node(&mut registry, attacker, context, &origin, "node"),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::ContextSessionMismatch {
                expected: owner,
                actual: attacker,
            }
        ))
    );
    Ok(())
}

#[test]
fn context_origin_cannot_change_without_document_rotation() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let first_origin = loopback_origin();
    let second_origin =
        Origin::parse("http://localhost:43127").expect("valid loopback fixture origin");

    bind_observed_node(
        &mut registry,
        session,
        context,
        &first_origin,
        "backend-node-17",
    )?;
    assert_eq!(
        bind_observed_node(
            &mut registry,
            session,
            context,
            &second_origin,
            "backend-node-18"
        ),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::OriginChangedWithoutDocumentAdvance
        ))
    );
    Ok(())
}

#[test]
fn external_identifiers_are_bounded_without_assuming_protocol_syntax() -> Result<(), Box<dyn Error>>
{
    let mut registry = BrowserAuthorityRegistry::new();

    assert_eq!(
        registry.register_session(""),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session(&"x".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1)),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );

    let unicode = registry.register_session("세션-opaque-✓")?;
    assert!(unicode.value() > 0);

    assert_eq!(
        registry.register_session(" "),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session("webdriver-session\n"),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session("webdriver-session\u{0000}"),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session("webdriver-session\u{200B}"),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );
    assert_eq!(
        registry.register_session("webdriver-session\u{202E}"),
        Err(BrowserRegistryError::InvalidExternalIdentifier)
    );

    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, "top-level-context")?;
    let origin = loopback_origin();
    assert_eq!(
        bind_observed_node(
            &mut registry,
            session,
            context,
            &origin,
            "backend-node-17\n",
        ),
        Err(WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId
        ))
    );
    Ok(())
}

#[test]
fn authority_identifier_capacity_is_bounded_and_testable() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(1);
    let session = registry.register_session("session-one")?;
    assert_eq!(
        registry.register_session("session-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let context = registry.register_context(session, "context-one")?;
    assert_eq!(
        registry.register_context(session, "context-two"),
        Err(BrowserRegistryError::IdentifierSpaceExhausted)
    );

    let origin = loopback_origin();
    assert!(bind_observed_node(&mut registry, session, context, &origin, "node-one").is_ok());
    assert_eq!(
        bind_observed_node(&mut registry, session, context, &origin, "node-two"),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::IdentifierSpaceExhausted
        ))
    );
    Ok(())
}

#[test]
fn unknown_internal_authority_is_rejected_before_node_binding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let unknown = BrowserSessionId::new(999)?;

    assert_eq!(
        registry.register_context(unknown, "context"),
        Err(BrowserRegistryError::UnknownBrowserSession)
    );

    let known = registry.register_session("known-session")?;
    let context = registry.register_context(known, "known-context")?;
    let origin = loopback_origin();
    assert_eq!(
        bind_observed_node(&mut registry, unknown, context, &origin, "node"),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::UnknownBrowserSession
        ))
    );
    Ok(())
}
