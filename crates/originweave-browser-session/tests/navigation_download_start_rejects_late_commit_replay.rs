use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct LateCommitAfterDownloadProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for LateCommitAfterDownloadProbePort {
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

impl AuthorizedContextOperationPort for LateCommitAfterDownloadProbePort {
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
fn download_start_consumption_rejects_late_commit_before_and_after_reestablishment() {
    let context = BrowsingContextId::new(1241).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = LateCommitAfterDownloadProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("download-late-commit-user-context-1241")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1241).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");
    bound
        .record_observed_navigation_download_started(&pending)
        .expect("matching download start consumes the navigation witness for authority transitions");

    let state_after_download = bound.browser_session().state();
    let recovery_after_download = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigationCommitted replay must not reuse a witness already consumed by download start"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_download,
        "late commit replay before re-establishment must not reopen or alter navigation state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_download.as_slice(),
        "late commit replay before re-establishment must not manufacture recovery evidence"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-download-late-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "late commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "late commit replay and retained-authority rejection must fail before adapter I/O"
    );

    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("late commit replay must preserve the one download-derived re-establishment opportunity");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "late commit replay before re-establishment must not spend the aggregate presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the preserved download-derived re-establishment opportunity must remain single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    let state_after_reestablishment = bound.browser_session().state();
    let recovery_after_reestablishment = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_committed(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed witness must remain dead when navigationCommitted arrives after fresh authority is minted"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_reestablishment,
        "post-re-establishment late commit replay must not alter aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_reestablishment.as_slice(),
        "post-re-establishment late commit replay must not alter recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "post-re-establishment late commit replay must fail before adapter I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-download-late-commit"),
        Ok(context),
        "late commit replay for the consumed witness must leave the fresh authority executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);

    let calls_before_next_navigation = adapter_calls.get();
    let next_pending = bound
        .record_observed_navigation(fresh.incarnation(), context, fresh.context_epoch())
        .expect("a later current-generation navigation may start normally");
    bound
        .record_observed_navigation_settled(&next_pending)
        .expect("the later current-generation navigation may settle normally");
    let next = bound
        .reestablish_presentation_authority(context)
        .expect("the later current-generation settlement permits one fresh authority");
    assert_eq!(
        next.context_epoch().value(),
        fresh.context_epoch().value() + 1,
        "stale navigationCommitted replay must not perturb the aggregate epoch allocator"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_next_navigation,
        "navigation observation and re-establishment bookkeeping remain zero-I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&next, "usable-after-next-navigation"),
        Ok(context)
    );
}
