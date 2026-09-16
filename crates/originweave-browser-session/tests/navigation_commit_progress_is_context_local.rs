use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ContextLocalCommitProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ContextLocalCommitProbePort {
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

impl AuthorizedContextOperationPort for ContextLocalCommitProbePort {
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

fn assert_pending_authority_remains_revoked(
    bound: &mut originweave_browser_session::BoundBrowserSession<ContextLocalCommitProbePort>,
    context: BrowsingContextId,
    retained_authority: &originweave_browser_session::PresentationMutationAuthority,
    operation: &'static str,
    adapter_calls: &Rc<Cell<usize>>,
    expected_calls: usize,
) {
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "commit bookkeeping must not manufacture re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(retained_authority, operation),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "commit bookkeeping must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        expected_calls,
        "re-establishment and retained-authority checks must fail before adapter I/O"
    );
}

#[test]
fn navigation_commit_progress_and_duplicate_rejection_are_context_local() {
    let first_context = BrowsingContextId::new(1341).expect("valid first context");
    let second_context = BrowsingContextId::new(1342).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = ContextLocalCommitProbePort {
        handles: VecDeque::from([
            handle(first_context, "commit-local-first-user-context-1341"),
            handle(second_context, "commit-local-second-user-context-1342"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1341).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let calls_after_create = adapter_calls.get();

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

    bound
        .record_observed_navigation_committed(&first_pending)
        .expect("first context records its own non-terminal commit progress");
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        first_context,
        &first_authority,
        "first-stale-after-own-commit",
        &adapter_calls,
        calls_after_create,
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        second_context,
        &second_authority,
        "second-stale-after-sibling-commit",
        &adapter_calls,
        calls_after_create,
    );

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit for the first context must be stale"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        first_context,
        &first_authority,
        "first-stale-after-duplicate-first-commit",
        &adapter_calls,
        calls_after_create,
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        second_context,
        &second_authority,
        "second-stale-after-duplicate-first-commit",
        &adapter_calls,
        calls_after_create,
    );

    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("duplicate rejection on the first context must not consume the sibling commit slot");
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        first_context,
        &first_authority,
        "first-stale-after-second-commit",
        &adapter_calls,
        calls_after_create,
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        second_context,
        &second_authority,
        "second-stale-after-own-commit",
        &adapter_calls,
        calls_after_create,
    );

    assert_eq!(
        bound.record_observed_navigation_committed(&second_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the sibling context also accepts commit progress exactly once"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(bound.browser_session().state(), state_after_starts);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_starts.as_slice()
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        first_context,
        &first_authority,
        "first-stale-after-duplicate-second-commit",
        &adapter_calls,
        calls_after_create,
    );
    assert_pending_authority_remains_revoked(
        &mut bound,
        second_context,
        &second_authority,
        "second-stale-after-duplicate-own-commit",
        &adapter_calls,
        calls_after_create,
    );

    bound
        .record_observed_navigation_settled(&first_pending)
        .expect("first context may close independently after its own commit");
    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("first context receives one explicit fresh authority");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "first re-establishment continues the aggregate-wide epoch sequence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "closing the first context must not unlock a committed-but-pending sibling"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-while-first-reestablished",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "first-context re-establishment must not reactivate the sibling's retained authority"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    bound
        .record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Aborted,
        )
        .expect("the sibling may independently terminate after its own commit");
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("the sibling receives one explicit fresh authority");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        first_reestablished.context_epoch().value() + 1,
        "sibling re-establishment receives the next aggregate-issued epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the first context's re-establishment opportunity remains single-use"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the sibling context's re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "first-current"),
        Ok(first_context)
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-current"),
        Ok(second_context)
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 2);
}
