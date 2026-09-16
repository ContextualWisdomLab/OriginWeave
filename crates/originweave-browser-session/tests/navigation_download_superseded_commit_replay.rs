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

struct SupersededDownloadCommitProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SupersededDownloadCommitProbePort {
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

impl AuthorizedContextOperationPort for SupersededDownloadCommitProbePort {
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
fn newer_navigation_rejects_late_commit_from_download_consumed_witness() {
    let context = BrowsingContextId::new(1242).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SupersededDownloadCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("download-superseded-commit-user-context-1242")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1242).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation start issues a pending witness");
    bound
        .record_observed_navigation_download_started(&first_pending)
        .expect("first navigation becomes a download and consumes its witness");

    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("a newer same-context navigation may supersede unused download eligibility");
    let state_with_second_pending = bound.browser_session().state();
    let recovery_with_second_pending = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "late commit from the download-consumed witness must not be remapped into the newer navigation"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_with_second_pending,
        "late commit from the consumed witness must leave the newer navigation pending"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_with_second_pending.as_slice(),
        "late commit replay must not mutate recovery evidence while the newer navigation is pending"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-while-second-navigation-pending"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "late commit replay must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the newer pending navigation must remain the authority gate after stale commit rejection"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "stale commit, retained-authority rejection, and premature re-establishment must remain zero-I/O"
    );

    bound
        .record_observed_navigation_settled(&second_pending)
        .expect("only the newer navigation witness may close the current navigation");
    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("the newer navigation's own closure permits exactly one fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "late commit replay must not spend a hidden presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the newer navigation's re-establishment opportunity remains single-use"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-superseded-late-commit"),
        Ok(context),
        "fresh authority from the newer navigation must remain executable"
    );
    assert_eq!(adapter_calls.get(), calls_after_create + 1);
}
