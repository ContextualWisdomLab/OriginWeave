use std::cell::Cell;
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

struct DuplicateCommitProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DuplicateCommitProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handle
            .take()
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

impl AuthorizedContextOperationPort for DuplicateCommitProbePort {
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

#[test]
fn duplicate_commit_replay_is_rejected_without_consuming_the_pending_witness() {
    let context = BrowsingContextId::new(1321).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DuplicateCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("duplicate-commit-user-context-1321")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1321).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues one pending witness");

    bound
        .record_observed_navigation_committed(&pending)
        .expect("the first matching commit records non-terminal progress");
    let state_after_first_commit = bound.browser_session().state();
    let recovery_after_first_commit = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a duplicate navigationCommitted delivery for the same witness must fail closed rather than being accepted as fresh progress"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_first_commit,
        "duplicate commit rejection must not mutate aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_first_commit.as_slice(),
        "duplicate commit rejection must not mutate recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit replay must not manufacture a terminal-derived re-establishment opportunity"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-duplicate-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "duplicate commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "commit progress and duplicate replay rejection must remain zero-I/O"
    );

    bound
        .record_observed_navigation_settled(&pending)
        .expect("duplicate commit rejection must leave the original pending witness available for one qualified closure");
    let state_after_closure = bound.browser_session().state();
    let recovery_after_closure = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the witness may accept the positive closure only once after duplicate commit rejection"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed witness must also reject a conflicting negative closure after duplicate commit rejection"
    );
    assert_eq!(
        bound.record_observed_navigation_download_started(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed witness must reject a conflicting download-start closure after duplicate commit rejection"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_closure,
        "second or conflicting closure rejection must not mutate aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_closure.as_slice(),
        "second or conflicting closure rejection must not mutate recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "all second-closure probes must fail before adapter I/O"
    );

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("the one qualified closure permits exactly one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "duplicate commit replay and rejected second closures must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the closure-derived re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-duplicate-commit"),
        Ok(context),
        "the authority minted after the one qualified closure must remain executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}

#[test]
fn duplicate_commit_rejection_preserves_negative_terminal_closure() {
    let context = BrowsingContextId::new(1322).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DuplicateCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("duplicate-commit-user-context-1322")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1322).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues one pending witness");
    bound
        .record_observed_navigation_committed(&pending)
        .expect("the first commit records progress");
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit must fail closed"
    );

    bound
        .record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Aborted)
        .expect("duplicate commit rejection must preserve a later negative terminal closure");
    let state_after_closure = bound.browser_session().state();
    let recovery_after_closure = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "negative closure remains single-assignment after duplicate commit rejection"
    );
    assert_eq!(
        bound.record_observed_navigation_download_started(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "download closure must not overwrite the accepted negative closure"
    );
    assert_eq!(bound.browser_session().state(), state_after_closure);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_closure.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("negative closure permits exactly one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "negative closure derived from the duplicate-commit path must remain single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-negative-closure"),
        Ok(context)
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}

#[test]
fn duplicate_commit_rejection_preserves_download_liveness_closure() {
    let context = BrowsingContextId::new(1323).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DuplicateCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("duplicate-commit-user-context-1323")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1323).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues one pending witness");
    bound
        .record_observed_navigation_committed(&pending)
        .expect("the first commit records progress");
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit must fail closed"
    );

    bound
        .record_observed_navigation_download_started(&pending)
        .expect("duplicate commit rejection must preserve a later download-start liveness closure");
    let state_after_closure = bound.browser_session().state();
    let recovery_after_closure = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "positive closure must not overwrite the accepted download-start closure"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed),
        Err(BrowserSessionError::AuthorityMismatch),
        "negative closure must not overwrite the accepted download-start closure"
    );
    assert_eq!(bound.browser_session().state(), state_after_closure);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_closure.as_slice()
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("download liveness closure permits exactly one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "download closure derived from the duplicate-commit path must remain single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-download-closure"),
        Ok(context)
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}
