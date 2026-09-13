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

struct StaleDownloadOwnershipProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for StaleDownloadOwnershipProbePort {
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

impl AuthorizedContextOperationPort for StaleDownloadOwnershipProbePort {
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

fn bound_session(
    session_id: u64,
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
) -> originweave_browser_session::BoundBrowserSession<StaleDownloadOwnershipProbePort> {
    let port = StaleDownloadOwnershipProbePort {
        handles,
        adapter_calls,
    };

    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn prior_session_download_witness_cannot_close_current_incarnation_pending_navigation() {
    let reused_context = BrowsingContextId::new(1211).expect("valid browsing context");
    let first_adapter_calls = Rc::new(Cell::new(0));
    let second_adapter_calls = Rc::new(Cell::new(0));
    let mut first = bound_session(
        1211,
        VecDeque::from([handle(reused_context, "download-prior-incarnation-first")]),
        Rc::clone(&first_adapter_calls),
    );
    let mut second = bound_session(
        1211,
        VecDeque::from([handle(reused_context, "download-prior-incarnation-second")]),
        Rc::clone(&second_adapter_calls),
    );

    let first_authority = first
        .create_disposable_context()
        .expect("first session accepts its context");
    let second_authority = second
        .create_disposable_context()
        .expect("second session accepts its context");
    assert_ne!(
        first_authority.incarnation(),
        second_authority.incarnation(),
        "reused transport and context identities must still have distinct Browser Session incarnations"
    );

    let first_pending = first
        .record_observed_navigation(
            first_authority.incarnation(),
            reused_context,
            first_authority.context_epoch(),
        )
        .expect("first incarnation enters navigation-pending state");
    let second_pending = second
        .record_observed_navigation(
            second_authority.incarnation(),
            reused_context,
            second_authority.context_epoch(),
        )
        .expect("second incarnation enters its own navigation-pending state");
    let calls_before_replay = second_adapter_calls.get();
    let recovery_before_replay = second.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        second.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "download evidence minted by a prior Browser Session incarnation must not close the current incarnation's pending navigation"
    );
    assert_eq!(
        second.reestablish_presentation_authority(reused_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting prior-incarnation download evidence must leave the current navigation pending"
    );
    assert_eq!(
        second.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "prior-incarnation download replay must not manufacture recovery evidence"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_replay,
        "prior-incarnation download evidence must fail before adapter I/O"
    );

    second
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the current incarnation's witness may close its navigation liveness boundary");
    let reestablished = second
        .reestablish_presentation_authority(reused_context)
        .expect("current incarnation may explicitly re-establish after its own qualified download start");
    assert_eq!(
        reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "rejected prior-incarnation evidence must not spend a presentation epoch"
    );
    assert_eq!(
        second_adapter_calls.get(),
        calls_before_replay,
        "qualified download evidence and authority re-establishment remain zero-I/O domain transitions"
    );
}

#[test]
fn recreated_raw_context_rejects_download_witness_from_destroyed_ownership_generation() {
    let reused_context = BrowsingContextId::new(1212).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1212,
        VecDeque::from([
            handle(reused_context, "download-aba-old-generation"),
            handle(reused_context, "download-aba-new-generation"),
        ]),
        Rc::clone(&adapter_calls),
    );

    let old_authority = bound
        .create_disposable_context()
        .expect("old ownership generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            reused_context,
            old_authority.context_epoch(),
        )
        .expect("old ownership generation enters navigation-pending state");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(reused_context)
        .expect("proven destruction consumes the old ownership generation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);

    let new_authority = bound
        .create_disposable_context()
        .expect("the same raw browsing-context id may be accepted as a fresh ownership generation");
    assert_eq!(
        new_authority.context(),
        old_authority.context(),
        "the hostile case intentionally reuses the same raw browsing-context id"
    );
    assert!(
        new_authority.context_epoch().value() > old_authority.context_epoch().value(),
        "fresh ownership must carry a newer aggregate-issued context epoch"
    );
    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            reused_context,
            new_authority.context_epoch(),
        )
        .expect("fresh ownership generation enters its own navigation-pending state");
    let calls_before_replay = adapter_calls.get();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_download_started(&old_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "download evidence from the destroyed ownership generation must not close the recreated raw context's current navigation"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(reused_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejected old-generation download evidence must leave the recreated context pending"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&old_authority, "stale-download-aba-authority"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the destroyed generation's retained presentation authority must stay unusable after raw-context ABA reuse"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "old-generation download replay must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "old-generation download evidence and stale authority must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_download_started(&new_pending)
        .expect("only the recreated ownership generation's current witness may close on download start");
    let reestablished = bound
        .reestablish_presentation_authority(reused_context)
        .expect("fresh ownership may re-establish after its own qualified download start");
    assert_eq!(
        reestablished.context_epoch().value(),
        new_authority.context_epoch().value() + 1,
        "rejected ABA replay must not spend a presentation epoch"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "valid download evidence and explicit re-establishment remain zero-I/O"
    );
}
