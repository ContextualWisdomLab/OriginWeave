#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserContextProtocolDispatchError,
    BrowserProtocolAdapterDescriptor, BrowserProtocolCapability,
    BrowserProtocolCapabilityRequirementError, BrowserProtocolKind, BrowserProtocolOperation,
    BrowserProtocolRuntimeMetadata, BrowserProtocolUseValidationError, BrowserSessionId,
    BrowsingContextId, DocumentEpoch, ObservedNodeHandle, Origin, OriginWeaveProtocolVersion,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiLocateNodesAdmissionError,
    WebDriverBiDiQueryNodesAdmissionError, WebDriverBiDiRemoteNodeReferenceError,
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

fn controlled_origin() -> Origin {
    Origin::parse("https://app.example").expect("valid controlled fixture origin")
}

fn current_target<'a>(
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

fn admit_query_nodes<'a>(
    descriptor: &BrowserProtocolAdapterDescriptor,
    registry: &mut BrowserAuthorityRegistry,
    target: BrowserContextOriginEpochDispatchTarget<'a>,
    query: &WebDriverBiDiAccessibilityQuery,
    items: &[(&str, Option<&str>)],
) -> Result<Vec<ObservedNodeHandle>, WebDriverBiDiQueryNodesAdmissionError> {
    descriptor.admit_query_nodes(
        registry,
        target,
        ORIGINWEAVE_PROTOCOL_VERSION,
        runtime_metadata(),
        query,
        items,
    )
}

#[test]
fn query_nodes_admission_requires_semantic_observation_and_binds_current_handles()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::SemanticObservation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task text"), 2)?;

    let handles = admit_query_nodes(
        &descriptor,
        &mut registry,
        target,
        &query,
        &[
            ("node", Some("shared-task-text")),
            ("node", Some("shared-task-text-shadow")),
        ],
    )?;

    assert_eq!(handles.len(), 2);
    assert_eq!(
        handles[0].browser_session(),
        target.context_origin().context().browser_session()
    );
    assert_eq!(
        handles[0].browsing_context(),
        target.context_origin().context().browsing_context()
    );
    assert_eq!(handles[0].origin(), &expected_origin);
    assert_eq!(handles[0].document_epoch(), target.expected_epoch());
    assert_ne!(handles[0].node_id(), handles[1].node_id());
    handles[0].validate_current(
        target.context_origin().context().browser_session(),
        target.context_origin().context().browsing_context(),
        &expected_origin,
        target.expected_epoch(),
    )?;
    Ok(())
}

#[test]
fn navigation_only_adapter_cannot_admit_query_nodes() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::Navigation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            target,
            &query,
            &[("node", Some("shared-submit"))],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch(
            BrowserContextProtocolDispatchError::ProtocolValidation(
                BrowserProtocolUseValidationError::Capability(
                    BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                        BrowserProtocolCapability::SemanticObservation,
                    ),
                ),
            ),
        ))
    );
    Ok(())
}

#[test]
fn typed_input_only_adapter_cannot_admit_query_nodes() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::TypedInput])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            target,
            &query,
            &[("node", Some("shared-submit"))],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch(
            BrowserContextProtocolDispatchError::ProtocolValidation(
                BrowserProtocolUseValidationError::Capability(
                    BrowserProtocolCapabilityRequirementError::UnsupportedCapability(
                        BrowserProtocolCapability::SemanticObservation,
                    ),
                ),
            ),
        ))
    );
    Ok(())
}

#[test]
fn query_nodes_admission_rejects_control_bearing_and_omitted_shared_ids()
-> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::SemanticObservation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            target,
            &query,
            &[("node", Some("shared-submit\n"))],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::LocateNodes(
            WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
                WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId
            )
        ))
    );
    assert_eq!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            target,
            &query,
            &[("node", None)],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::LocateNodes(
            WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
                WebDriverBiDiRemoteNodeReferenceError::MissingSharedId
            )
        ))
    );
    Ok(())
}

#[test]
fn query_nodes_admission_fails_closed_on_stale_document_epoch() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::SemanticObservation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let stale_target = current_target(&mut registry, &expected_origin)?;
    let context = stale_target.context_origin().context().browsing_context();
    let current_epoch = registry.advance_document(context)?;
    registry.bind_context_origin(
        stale_target.context_origin().context().browser_session(),
        context,
        &expected_origin,
    )?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_ne!(current_epoch, stale_target.expected_epoch());
    assert_eq!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            stale_target,
            &query,
            &[("node", Some("shared-submit"))],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch(
            BrowserContextProtocolDispatchError::DocumentEpochMismatch {
                expected: stale_target.expected_epoch(),
                current: current_epoch,
            }
        ))
    );
    Ok(())
}

#[test]
fn query_nodes_admission_fails_closed_on_unknown_session() -> Result<(), Box<dyn Error>> {
    let descriptor = descriptor(&[BrowserProtocolCapability::SemanticObservation])?;
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = BrowserContextOriginEpochDispatchTarget::new(
        BrowserContextOriginDispatchTarget::new(
            BrowserContextDispatchTarget::new(
                BrowserSessionId::new(99).expect("nonzero fixture session"),
                BrowsingContextId::new(7).expect("nonzero fixture context"),
            ),
            &expected_origin,
        ),
        DocumentEpoch::new(1).expect("nonzero fixture epoch"),
    );
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert!(matches!(
        admit_query_nodes(
            &descriptor,
            &mut registry,
            target,
            &query,
            &[("node", Some("shared-submit"))],
        ),
        Err(WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch(
            BrowserContextProtocolDispatchError::BrowserAuthority(_)
        ))
    ));
    Ok(())
}

#[test]
fn query_nodes_maps_to_semantic_observation_and_error_contract_is_source_aware() {
    assert_eq!(
        BrowserProtocolOperation::QueryNodes.required_capability(),
        BrowserProtocolCapability::SemanticObservation
    );

    let expected = DocumentEpoch::new(1).expect("nonzero fixture epoch");
    let current = DocumentEpoch::new(2).expect("nonzero fixture epoch");
    let errors = [
        WebDriverBiDiQueryNodesAdmissionError::ProtocolDispatch(
            BrowserContextProtocolDispatchError::DocumentEpochMismatch { expected, current },
        ),
        WebDriverBiDiQueryNodesAdmissionError::LocateNodes(
            WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
                WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
            ),
        ),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_some());
    }
}
