use std::{cell::Cell, collections::BTreeSet, error::Error};

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, AdmittedNodeAuthorityError, ApprovalEvidence,
    BoundedWebDriverBiDiResponseDocument, BrowserAuthorityRegistry, BrowserContextDispatchTarget,
    BrowserContextOriginDispatchTarget, BrowserContextOriginEpochDispatchTarget,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability, BrowserProtocolKind,
    BrowsingContextId, ExecutionPurpose, InstructionSource, NodeActionKind, Origin,
    OriginWeaveProtocolVersion, PolicyContext, RobotsDecision, SecretDelivery,
    SemanticNodeActionBinding, SessionMode, ValidatedBrowserProtocolUse,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesCommand,
};
use originweave_policy::PolicyAuthorizedSemanticNodeAction;

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";
const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct AuthorizedFixture {
    registry: BrowserAuthorityRegistry,
    context: BrowsingContextId,
    authorized: PolicyAuthorizedSemanticNodeAction,
}

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

fn authorized_action() -> Result<AuthorizedFixture, Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let session = registry.register_session("semantic-dispatch-session")?;
    let context = registry.register_context(session, "semantic-dispatch-context")?;
    let site = Origin::parse("https://app.example")
        .map_err(|error| std::io::Error::other(format!("fixture origin rejected: {error:?}")))?;
    let epoch = registry.bind_context_origin(session, context, &site)?;
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            &site,
        ),
        epoch,
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Continue"), 1)?;
    let command = WebDriverBiDiLocateNodesCommand::new(81, "semantic-dispatch-context", &query)?;
    let document = BoundedWebDriverBiDiResponseDocument::new(
        r#"{"type":"success","id":81,"result":{"nodes":[{"type":"node","sharedId":"semantic-dispatch-node"}]}}"#,
    )?;
    let handle = command
        .bind_response_document_nodes(
            document,
            semantic_observation_proof()?,
            &mut registry,
            target,
        )?
        .into_iter()
        .next()
        .ok_or("locateNodes fixture did not bind its node")?;
    let request = ActionRequest::new(
        ActionKind::Navigate,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        ActionIntentDigest::parse(VALID_INTENT)
            .map_err(|error| std::io::Error::other(format!("intent rejected: {error:?}")))?,
    );
    let binding = SemanticNodeActionBinding::new(handle, NodeActionKind::Click, request)?;
    let policy_context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::UserDelegatedTask,
        BTreeSet::from([ActionKind::Navigate.required_capability()]),
        BTreeSet::from([site.clone()]),
        BTreeSet::from([site]),
        RobotsDecision::Allowed,
        ApprovalEvidence::None,
    );
    let authorized = PolicyAuthorizedSemanticNodeAction::authorize(binding, &policy_context)?;

    Ok(AuthorizedFixture {
        registry,
        context,
        authorized,
    })
}

fn dispatch_unit_callback(
    authorized: &PolicyAuthorizedSemanticNodeAction,
    registry: &BrowserAuthorityRegistry,
    called: &Cell<bool>,
) -> Result<(), AdmittedNodeAuthorityError> {
    authorized.dispatch_if_current(registry, |_binding| called.set(true))
}

#[test]
fn dispatch_callback_runs_only_after_registry_owned_browser_revalidation()
-> Result<(), Box<dyn Error>> {
    let fixture = authorized_action()?;
    let called = Cell::new(false);

    let result = fixture
        .authorized
        .dispatch_if_current(&fixture.registry, |binding| {
            called.set(true);
            (binding.node_action(), binding.request().action())
        })?;

    assert!(called.get());
    assert_eq!(result, (NodeActionKind::Click, ActionKind::Navigate));
    Ok(())
}

#[test]
fn stale_registry_authority_never_reaches_dispatch_callback() -> Result<(), Box<dyn Error>> {
    let mut fixture = authorized_action()?;
    let called = Cell::new(false);

    dispatch_unit_callback(&fixture.authorized, &fixture.registry, &called)?;
    assert!(called.replace(false));

    fixture.registry.advance_document(fixture.context)?;

    assert_eq!(
        fixture
            .authorized
            .dispatch_if_current(&fixture.registry, |_binding| called.set(true))
            .err(),
        Some(AdmittedNodeAuthorityError::NotAdmitted)
    );
    assert!(!called.get());
    Ok(())
}

#[test]
fn adapter_failure_remains_separate_after_successful_revalidation() -> Result<(), Box<dyn Error>> {
    let fixture = authorized_action()?;

    let adapter_result = fixture
        .authorized
        .dispatch_if_current(&fixture.registry, |_binding| -> Result<(), &'static str> {
            Err("adapter failed")
        })?;

    assert_eq!(adapter_result, Err("adapter failed"));
    Ok(())
}
