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

struct DownloadNavigationProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DownloadNavigationProbePort {
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

impl AuthorizedContextOperationPort for DownloadNavigationProbePort {
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
) -> originweave_browser_session::BoundBrowserSession<DownloadNavigationProbePort> {
    let port = DownloadNavigationProbePort {
        handles,
        adapter_calls,
    };

    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

#[test]
fn matching_download_start_closes_pending_navigation_without_reviving_old_authority() {
    let context = BrowsingContextId::new(1201).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1201,
        VecDeque::from([handle(context, "download-navigation-user-context")]),
        Rc::clone(&adapter_calls),
    );

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");

    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-while-navigation-pending"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "navigation start must revoke retained presentation authority before any download outcome"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    bound
        .record_observed_navigation_download_started(&pending)
        .expect("matching download start closes the pending navigation liveness boundary");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "download-start observation is evidence only and must not perform browser I/O"
    );

    let after_download_start = bound
        .reestablish_presentation_authority(context)
        .expect("qualified download start permits one explicit fresh presentation authority");
    assert_eq!(
        after_download_start.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "download start itself must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "download-start-derived re-establishment opportunity must be single-use"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "duplicate re-establishment rejection must remain zero-I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &after_download_start,
            "usable-after-qualified-download-start",
        ),
        Ok(context),
        "only the explicitly re-established authority is usable after download start"
    );
}

#[test]
fn superseded_download_start_cannot_close_the_current_pending_navigation() {
    let context = BrowsingContextId::new(1202).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1202,
        VecDeque::from([handle(
            context,
            "superseded-download-navigation-user-context",
        )]),
        Rc::clone(&adapter_calls),
    );

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation start issues a pending witness");
    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("later qualified navigation supersedes the earlier pending witness");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation state transitions must remain zero-I/O"
    );
    let recovery_evidence_before_replay = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a delayed download start for the superseded navigation must not close the current pending navigation"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting stale download evidence must leave the newer navigation pending"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_before_replay.as_slice(),
        "stale download evidence must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "stale download evidence and premature re-establishment rejection must perform no browser I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the current pending witness may close on download start");
    let current = bound
        .reestablish_presentation_authority(context)
        .expect("current download start permits one explicit fresh authority");
    assert_eq!(
        current.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "rejected stale download evidence must not spend a presentation epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&current, "usable-after-current-download-start"),
        Ok(context)
    );
}

#[test]
fn settled_sibling_download_witness_cannot_close_another_context_pending_navigation() {
    let first_context = BrowsingContextId::new(1203).expect("valid first context");
    let second_context = BrowsingContextId::new(1204).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1203,
        VecDeque::from([
            handle(first_context, "settled-download-sibling-first"),
            handle(second_context, "settled-download-sibling-second"),
        ]),
        Rc::clone(&adapter_calls),
    );

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
    let calls_after_starts = adapter_calls.get();

    bound
        .record_observed_navigation_settled(&first_pending)
        .expect("first context reaches its own complete-positive terminal outcome");
    let recovery_evidence_after_first_settlement =
        bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(adapter_calls.get(), calls_after_starts);

    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a settled sibling witness must not be reused as download evidence while another context is pending"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting sibling download replay must leave the second context pending"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_first_settlement.as_slice(),
        "sibling download replay must not mutate recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_starts,
        "sibling witness rejection and pending-state checks must be zero-I/O"
    );

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("rejected sibling download replay must preserve the first context's terminal eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "the next authority must use the aggregate-wide epoch after both initial contexts"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-while-its-navigation-remains-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "first-context re-establishment must not revive the second context's retained authority"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("the second context may close only with its own current witness");
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("second context may re-establish after its own qualified download start");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        first_reestablished.context_epoch().value() + 1,
        "sibling witness rejection must preserve the aggregate-wide monotonic epoch sequence"
    );
}

#[test]
fn destroyed_generation_download_witness_cannot_close_surviving_sibling_pending_navigation() {
    let destroyed_context = BrowsingContextId::new(1205).expect("valid destroyed context");
    let surviving_context = BrowsingContextId::new(1206).expect("valid surviving context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1205,
        VecDeque::from([
            handle(destroyed_context, "destroyed-download-generation"),
            handle(surviving_context, "surviving-download-generation"),
        ]),
        Rc::clone(&adapter_calls),
    );

    let destroyed_authority = bound
        .create_disposable_context()
        .expect("destroyed context accepted before navigation");
    let surviving_authority = bound
        .create_disposable_context()
        .expect("surviving context accepted before navigation");
    let destroyed_pending = bound
        .record_observed_navigation(
            destroyed_authority.incarnation(),
            destroyed_context,
            destroyed_authority.context_epoch(),
        )
        .expect("destroyed context enters navigation-pending state");
    let surviving_pending = bound
        .record_observed_navigation(
            surviving_authority.incarnation(),
            surviving_context,
            surviving_authority.context_epoch(),
        )
        .expect("surviving context independently enters navigation-pending state");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(destroyed_context)
        .expect("proven destruction consumes the destroyed context generation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_download_started(&destroyed_pending),
        Err(BrowserSessionError::ContextNotOwned),
        "download evidence from a proven-destroyed ownership generation must fail closed"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(surviving_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed-generation replay must not close the surviving sibling navigation"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &surviving_authority,
            "surviving-stale-while-own-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the surviving context's retained authority must remain revoked"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "destroyed-generation download replay must not mutate recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "destroyed-generation replay and sibling authority rejection must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_download_started(&surviving_pending)
        .expect("the surviving context may close only with its own current witness");
    let surviving_reestablished = bound
        .reestablish_presentation_authority(surviving_context)
        .expect("surviving context may re-establish after its own qualified download start");
    assert_eq!(
        surviving_reestablished.context_epoch().value(),
        surviving_authority.context_epoch().value() + 1,
        "proven destruction and rejected replay must not spend a presentation epoch"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "valid surviving download transition must not manufacture recovery evidence"
    );
}
