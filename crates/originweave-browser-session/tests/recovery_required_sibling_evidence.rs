use std::collections::VecDeque;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct FailingDestroyPort {
    handles: VecDeque<DisposableContextHandle>,
}

impl DisposableContextPort for FailingDestroyPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.handles
            .pop_front()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Err(DisposableContextDestroyError::DestroyFailed)
    }
}

fn handle(context: u64, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        BrowsingContextId::new(context).expect("valid browsing context"),
    )
}

fn existing_exact_handle(
    evidence: &BrowserSessionRecoveryEvidence,
) -> Option<&DisposableContextHandle> {
    match evidence {
        BrowserSessionRecoveryEvidence::DuplicateAdapterHandle(handle)
        | BrowserSessionRecoveryEvidence::UnsettledAdapterHandle(handle)
        | BrowserSessionRecoveryEvidence::UnprovenDestruction(handle)
        | BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(handle)
        | BrowserSessionRecoveryEvidence::TransportLossOwnedHandle(handle) => Some(handle),
        BrowserSessionRecoveryEvidence::PartialCreationIsolation(_) => None,
    }
}

#[test]
fn recovery_required_projects_exact_handles_for_indirectly_uncertain_siblings() {
    let first = handle(5070, "recovery-user-context-a");
    let sibling = handle(5071, "recovery-user-context-b");
    let port = FailingDestroyPort {
        handles: VecDeque::from([first.clone(), sibling.clone()]),
    };
    let session = BrowserSession::start(BrowserSessionId::new(507).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first accepted context");
    let _sibling_authority = bound
        .create_disposable_context()
        .expect("second accepted context");

    assert_eq!(
        bound.destroy_disposable_context(&first_authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );

    let evidence = bound.browser_session().recovery_evidence();
    assert!(
        evidence.contains(&BrowserSessionRecoveryEvidence::UnprovenDestruction(
            first.clone()
        )),
        "the directly failed destruction must keep its cause-specific evidence"
    );
    assert!(
        evidence.contains(
            &BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle(sibling.clone())
        ),
        "the indirectly invalidated sibling must be projected as non-authorizing exact recovery evidence"
    );
    assert_eq!(
        evidence
            .iter()
            .filter_map(existing_exact_handle)
            .filter(|candidate| *candidate == &first)
            .count(),
        1,
        "the directly failed context must not be duplicated as generic recovery evidence"
    );
    assert_eq!(
        evidence
            .iter()
            .filter_map(existing_exact_handle)
            .filter(|candidate| *candidate == &sibling)
            .count(),
        1,
        "an indirectly invalidated sibling must be retained exactly once"
    );
}
