use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct SiblingDestructionCommitProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SiblingDestructionCommitProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handles
            .pop_front()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }
}

impl AuthorizedContextOperationPort for SiblingDestructionCommitProbePort {
    type Operation = &'static str;
    type Output = BrowsingContextId;
    type Error = ();

    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(request.context().browsing_context())
    }
}

fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

#[test]
fn destroying_committed_pending_context_preserves_sibling_commit_slot() {
    let first_context = BrowsingContextId::new(1351).expect("valid first context");
    let second_context = BrowsingContextId::new(1352).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SiblingDestructionCommitProbePort {
        handles: VecDeque::from([
            handle(first_context, "commit-destroy-first-user-context-1351"),
            handle(second_context, "commit-destroy-second-user-context-1352"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1351).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first context enters navigation-pending state");
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("second context independently enters navigation-pending state");
    let state_after_starts = bound.browser_session().state();
    let recovery_after_starts = bound.browser_session().recovery_evidence().to_vec();
    let calls_after_starts = adapter_calls.get();

    bound
        .record_observed_navigation_committed(&first_pending)
        .expect("first context records non-terminal commit progress");
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_starts);

    bound
        .destroy_owned_disposable_context(first_context)
        .expect("proven destruction consumes only the first context ownership");
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "destroying one committed-but-unsettled context must keep aggregate trust active"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice(),
        "proven sibling destruction must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_starts + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    let calls_after_destroy = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::ContextNotOwned),
        "destroyed context commit evidence is permanently stale"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::ContextNotOwned),
        "destroyed committed context cannot regain presentation authority"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroying the committed sibling must not manufacture eligibility for the surviving pending context"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-after-committed-sibling-destruction",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the surviving context's retained authority stays revoked"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "stale destroyed-context evidence and surviving authority probes fail before adapter I/O"
    );

    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("destroying a committed sibling must not consume the surviving context's own commit slot");
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_eq!(
        bound.record_observed_navigation_committed(&second_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the surviving context still accepts commit progress exactly once"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("the surviving committed navigation may close through download start");
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("the surviving context receives one explicit fresh authority");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "destroying a sibling context must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the surviving context's re-establishment remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-current"),
        Ok(second_context),
        "fresh authority after sibling destruction must be executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy + 1);
}
