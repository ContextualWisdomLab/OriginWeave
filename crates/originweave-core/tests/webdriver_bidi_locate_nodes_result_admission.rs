use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserProtocolAdapterDescriptor,
    BrowserProtocolCapability, BrowserProtocolKind, BrowserRegistryError, Origin,
    OriginWeaveProtocolVersion, ValidatedBrowserProtocolUse, WebDriverBiDiAccessibilityQuery,
    WebDriverBiDiAccessibilityQueryError, WebDriverBiDiCommandResponseKind,
    WebDriverBiDiLocateNodesAdmissionError, WebDriverBiDiLocateNodesCommand,
    WebDriverBiDiLocateNodesResultAdmissionError, WebDriverBiDiRemoteNodeReferenceError,
};

const ORIGINWEAVE_PROTOCOL_VERSION: OriginWeaveProtocolVersion =
    OriginWeaveProtocolVersion::new(0, 1);
const ADAPTER_VERSION: &str = "originweave-bidi-v1";
const PROTOCOL_REVISION: &str = "webdriver-bidi-wd-2026-06-01";
const BROWSER_REVISION: &str = "chromium-r1639810";

fn correlated_success(
    max_node_count: u16,
) -> Result<originweave_core::ValidatedWebDriverBiDiLocateNodesResponse, Box<dyn Error>> {
    let query =
        WebDriverBiDiAccessibilityQuery::new(Some("button"), Some("Submit task"), max_node_count)?;
    Ok(
        WebDriverBiDiLocateNodesCommand::new(42, "context-a", &query)?
            .correlate_response_envelope(WebDriverBiDiCommandResponseKind::Success, Some(42))?
            .into_validated_success()?,
    )
}

fn current_target<'a>(
    registry: &mut BrowserAuthorityRegistry,
    origin: &'a Origin,
    external_context: &str,
) -> Result<BrowserContextOriginEpochDispatchTarget<'a>, Box<dyn Error>> {
    let session = registry.register_session("webdriver-session")?;
    let context = registry.register_context(session, external_context)?;
    let epoch = registry.bind_context_origin(session, context, origin)?;
    Ok(BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(session, context),
            origin,
        ),
        epoch,
    ))
}

fn controlled_origin() -> Result<Origin, Box<dyn Error>> {
    Origin::parse("https://app.example").map_err(|_error| "valid controlled fixture origin".into())
}

fn protocol_proof(
    kind: BrowserProtocolKind,
    capability: BrowserProtocolCapability,
) -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    let descriptor = BrowserProtocolAdapterDescriptor::new(
        kind,
        ORIGINWEAVE_PROTOCOL_VERSION,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        &[capability],
    )?;
    Ok(descriptor.validate_use(
        ORIGINWEAVE_PROTOCOL_VERSION,
        kind,
        ADAPTER_VERSION,
        PROTOCOL_REVISION,
        BROWSER_REVISION,
        capability,
    )?)
}

fn semantic_observation_proof() -> Result<ValidatedBrowserProtocolUse, Box<dyn Error>> {
    protocol_proof(
        BrowserProtocolKind::WebDriverBiDi,
        BrowserProtocolCapability::SemanticObservation,
    )
}

#[test]
fn correlated_result_admission_retains_exact_command_and_normalized_nodes()
-> Result<(), Box<dyn Error>> {
    let result = correlated_success(2)?.admit_result_nodes(&[
        ("node", Some("shared-node-a")),
        ("node", Some("shared-node-b")),
    ])?;

    assert_eq!(result.command_id(), 42);
    assert_eq!(result.browsing_context(), "context-a");
    assert_eq!(result.max_node_count(), 2);
    assert_eq!(result.nodes().len(), 2);
    assert_eq!(result.nodes()[0].remote_type(), "node");
    assert_eq!(result.nodes()[0].shared_id(), "shared-node-a");
    assert_eq!(result.nodes()[1].shared_id(), "shared-node-b");
    Ok(())
}

#[test]
fn correlated_result_admission_rejects_over_budget_batch_before_node_normalization()
-> Result<(), Box<dyn Error>> {
    let error =
        correlated_success(1)?.admit_result_nodes(&[("not-a-node", None), ("not-a-node", None)]);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResultAdmissionError::Query(
            WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
        ))
    );
    Ok(())
}

#[test]
fn correlated_result_admission_rejects_invalid_remote_node_shape() -> Result<(), Box<dyn Error>> {
    let error = correlated_success(1)?.admit_result_nodes(&[("string", Some("shared-node-a"))]);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesResultAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType,
        ))
    );
    Ok(())
}

#[test]
fn correlated_result_admission_error_preserves_typed_source() {
    let query_error = WebDriverBiDiLocateNodesResultAdmissionError::Query(
        WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
    );
    assert!(query_error.source().is_some());
    assert!(!query_error.to_string().is_empty());

    let remote_error = WebDriverBiDiLocateNodesResultAdmissionError::RemoteNode(
        WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
    );
    assert!(remote_error.source().is_some());
    assert!(!remote_error.to_string().is_empty());
}

#[test]
fn correlated_result_binds_only_to_its_exact_registered_context() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;
    let result = correlated_success(2)?.admit_result_nodes(&[
        ("node", Some("shared-node-a")),
        ("node", Some("shared-node-b")),
    ])?;

    let handles =
        result.bind_current_nodes(semantic_observation_proof()?, &mut registry, target)?;

    assert_eq!(handles.len(), 2);
    assert_eq!(
        handles[0].browsing_context(),
        target.context_origin().context().browsing_context()
    );
    assert_eq!(handles[0].origin(), &origin);
    assert_eq!(handles[0].document_epoch(), target.expected_epoch());
    Ok(())
}

#[test]
fn correlated_result_rejects_cross_context_rebinding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-b")?;
    let result = correlated_success(1)?.admit_result_nodes(&[("node", Some("shared-node-a"))])?;

    let error = result.bind_current_nodes(semantic_observation_proof()?, &mut registry, target);

    assert_eq!(
        error,
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::ContextExternalIdentifierMismatch,
        ))
    );
    let error = error.err().ok_or("expected context mismatch")?;
    assert!(error.to_string().contains("external identifier"));
    Ok(())
}

#[test]
fn correlated_result_rejects_non_bidi_protocol_proof() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;
    let result = correlated_success(1)?.admit_result_nodes(&[("node", Some("shared-node-a"))])?;

    assert_eq!(
        result.bind_current_nodes(
            protocol_proof(
                BrowserProtocolKind::ChromeDevToolsProtocol,
                BrowserProtocolCapability::SemanticObservation,
            )?,
            &mut registry,
            target,
        ),
        Err(
            WebDriverBiDiLocateNodesAdmissionError::UnsupportedProtocolKind(
                BrowserProtocolKind::ChromeDevToolsProtocol,
            )
        )
    );
    Ok(())
}

#[test]
fn correlated_result_rejects_non_observation_protocol_proof() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;
    let result = correlated_success(1)?.admit_result_nodes(&[("node", Some("shared-node-a"))])?;

    assert_eq!(
        result.bind_current_nodes(
            protocol_proof(
                BrowserProtocolKind::WebDriverBiDi,
                BrowserProtocolCapability::TypedInput,
            )?,
            &mut registry,
            target,
        ),
        Err(
            WebDriverBiDiLocateNodesAdmissionError::UnsupportedCapability(
                BrowserProtocolCapability::TypedInput,
            )
        )
    );
    Ok(())
}

#[test]
fn correlated_result_rejects_stale_document_epoch() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;
    let context = target.context_origin().context().browsing_context();
    let current_epoch = registry.advance_document(context)?;
    registry.bind_context_origin(
        target.context_origin().context().browser_session(),
        context,
        &origin,
    )?;
    let result = correlated_success(1)?.admit_result_nodes(&[("node", Some("shared-node-a"))])?;

    assert_eq!(
        result.bind_current_nodes(semantic_observation_proof()?, &mut registry, target),
        Err(
            WebDriverBiDiLocateNodesAdmissionError::DocumentEpochMismatch {
                expected: target.expected_epoch(),
                current: current_epoch,
            }
        )
    );
    Ok(())
}

#[test]
fn correlated_result_keeps_node_binding_transactional_on_identifier_exhaustion()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::with_identifier_limit(1);
    let origin = controlled_origin()?;
    let target = current_target(&mut registry, &origin, "context-a")?;
    let result = correlated_success(2)?.admit_result_nodes(&[
        ("node", Some("shared-node-a")),
        ("node", Some("shared-node-b")),
    ])?;

    assert_eq!(
        result.bind_current_nodes(semantic_observation_proof()?, &mut registry, target),
        Err(WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::IdentifierSpaceExhausted,
        ))
    );
    Ok(())
}
