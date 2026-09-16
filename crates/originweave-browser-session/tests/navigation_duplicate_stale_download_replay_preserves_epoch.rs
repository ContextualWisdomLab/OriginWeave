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

struct DuplicateDownloadReplayProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DuplicateDownloadReplayProbePort {
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

impl AuthorizedContextOperationPort for DuplicateDownloadReplayProbePort {
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
fn duplicate_stale_download_replay_does_not_spend_presentation_epoch() {
    let context = BrowsingContextId::new(1239).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DuplicateDownloadReplayProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("duplicate-download-epoch-context")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1239).expect("valid session id"))
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
        .expect("first download start closes the first navigation");
    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("second navigation supersedes the unused first download eligibility");

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed first witness must remain stale while the second navigation is pending"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "duplicate stale download-start replay must not change aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "duplicate stale download-start replay must not manufacture recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate stale download-start replay must not create re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-duplicate-download"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "duplicate stale download-start replay must not reactivate retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "duplicate stale download-start replay and rejected authority checks must remain zero-I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the current pending witness may close the navigation liveness boundary");
    let current = bound
        .reestablish_presentation_authority(context)
        .expect("current download start permits one explicit re-establishment");
    assert_eq!(
        current.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "duplicate stale download-start replay must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the current download-derived re-establishment opportunity remains single-use"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&current, "current-authority-usable"),
        Ok(context),
        "the current re-established authority must remain usable"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create + 1,
        "only the explicitly authorized current operation may cross the adapter boundary"
    );
}
