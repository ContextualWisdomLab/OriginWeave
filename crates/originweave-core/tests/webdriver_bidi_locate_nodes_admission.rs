#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    BrowserAuthorityRegistry, BrowserContextDispatchTarget, BrowserContextOriginDispatchTarget,
    BrowserContextOriginEpochDispatchTarget, BrowserRegistryError, DocumentEpoch, Origin,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiAccessibilityQueryError,
    WebDriverBiDiLocateNodesAdmissionError, WebDriverBiDiRemoteNodeReferenceError,
};

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

#[test]
fn locate_nodes_result_binds_admitted_shared_ids_to_current_authority()
-> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task text"), 2)?;

    let handles = query.bind_current_nodes(
        &mut registry,
        target,
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
fn over_budget_locate_nodes_result_fails_before_node_binding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        query.bind_current_nodes(
            &mut registry,
            target,
            &[
                ("node", Some("shared-submit")),
                ("node", Some("shared-extra")),
            ],
        ),
        Err(WebDriverBiDiLocateNodesAdmissionError::Query(
            WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded
        ))
    );
    Ok(())
}

#[test]
fn stale_document_epoch_fails_before_locate_nodes_binding() -> Result<(), Box<dyn Error>> {
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
        query.bind_current_nodes(
            &mut registry,
            stale_target,
            &[("node", Some("shared-submit"))],
        ),
        Err(
            WebDriverBiDiLocateNodesAdmissionError::DocumentEpochMismatch {
                expected: stale_target.expected_epoch(),
                current: current_epoch,
            }
        )
    );
    Ok(())
}

#[test]
fn untrusted_non_node_item_fails_before_registry_binding() -> Result<(), Box<dyn Error>> {
    let mut registry = BrowserAuthorityRegistry::new();
    let expected_origin = controlled_origin();
    let target = current_target(&mut registry, &expected_origin)?;
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;

    assert_eq!(
        query.bind_current_nodes(&mut registry, target, &[("object", Some("shared-submit"))],)
        Err(WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType
        ))
    );
    Ok(())
}

#[test]
fn locate_nodes_admission_error_contract_is_source_aware() {
    let expected = DocumentEpoch::new(1).expect("nonzero fixture epoch");
    let current = DocumentEpoch::new(2).expect("nonzero fixture epoch");
    let errors = [
        WebDriverBiDiLocateNodesAdmissionError::Query(
            WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
        ),
        WebDriverBiDiLocateNodesAdmissionError::RemoteNode(
            WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
        ),
        WebDriverBiDiLocateNodesAdmissionError::DocumentEpochMismatch { expected, current },
        WebDriverBiDiLocateNodesAdmissionError::BrowserAuthority(
            BrowserRegistryError::UnknownBrowserSession,
        ),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
    }
    assert!(errors[0].source().is_some());
    assert!(errors[1].source().is_some());
    assert!(errors[2].source().is_none());
    assert!(errors[3].source().is_some());
}
