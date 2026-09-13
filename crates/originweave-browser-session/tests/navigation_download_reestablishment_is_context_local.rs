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

struct CrossContextDownloadProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CrossContextDownloadProbePort {
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

impl AuthorizedContextOperationPort for CrossContextDownloadProbePort {
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

#[derive(Clone, Copy)]
enum ConsumedFirstReplay {
    DownloadStarted,
    Settled,
    Aborted,
    Failed,
}

fn assert_consumed_first_replay_is_non_mutating(replay: ConsumedFirstReplay, session_id: u64) {
    let first_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid first context");
    let second_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CrossContextDownloadProbePort {
        handles: VecDeque::from([
            handle(first_context, "cross-context-download-replay-first"),
            handle(second_context, "cross-context-download-replay-second"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_id).expect("valid browser session id"),
    )
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
    bound
        .record_observed_navigation_download_started(&first_pending)
        .expect("first download start closes only the first navigation");
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("sibling context independently enters navigation-pending state");

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    let replay_result = match replay {
        ConsumedFirstReplay::DownloadStarted => {
            bound.record_observed_navigation_download_started(&first_pending)
        }
        ConsumedFirstReplay::Settled => bound.record_observed_navigation_settled(&first_pending),
        ConsumedFirstReplay::Aborted => bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        ConsumedFirstReplay::Failed => bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Failed,
        ),
    };

    assert_eq!(
        replay_result,
        Err(BrowserSessionError::AuthorityMismatch),
        "consumed first-context evidence must remain non-authorizing while its sibling is pending"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "one rejected replay must not change aggregate lifecycle state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "one rejected replay must not mutate recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "one rejected replay must not create sibling re-establishment eligibility"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-remains-stale-after-one-first-context-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "one rejected replay must not reactivate sibling retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "one rejected replay and rejected sibling authority checks must remain zero-I/O"
    );

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("one rejected replay must preserve first-context download eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "one rejected replay must not spend an aggregate presentation epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_reestablished,
            "first-fresh-authority-usable-while-sibling-remains-pending",
        ),
        Ok(first_context),
        "one rejected replay must not produce a hollow first-context authority while its sibling remains pending"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create + 1,
        "only the explicitly authorized first-context operation may cross the adapter boundary"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "using the first-context fresh authority must not close sibling navigation"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the sibling's own witness may close its pending navigation");
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("sibling may re-establish only after its own qualified closing evidence");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        first_reestablished.context_epoch().value() + 1,
        "sibling closure must continue the aggregate-wide epoch sequence exactly once"
    );
}

#[test]
fn sibling_navigation_preserves_download_reestablishment_eligibility() {
    let first_context = BrowsingContextId::new(1233).expect("valid first context");
    let second_context = BrowsingContextId::new(1234).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CrossContextDownloadProbePort {
        handles: VecDeque::from([
            handle(first_context, "cross-context-download-first"),
            handle(second_context, "cross-context-download-second"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1233).expect("valid session id"))
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
    bound
        .record_observed_navigation_download_started(&first_pending)
        .expect("first context download start creates one re-establishment opportunity");
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("sibling navigation may start while first download eligibility remains unused");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation and download-start observations must remain zero-I/O"
    );

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("sibling navigation must not erase first context download-derived eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "sibling navigation must not spend an aggregate presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the sibling remains pending until its own closing evidence arrives"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-while-own-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "first-context re-establishment must not revive sibling retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "sibling authority gating must remain zero-I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the sibling's own current witness may close its navigation liveness boundary");
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("sibling may re-establish only after its own qualified download start");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        first_reestablished.context_epoch().value() + 1,
        "independent sibling re-establishment must continue the aggregate-wide epoch sequence"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "first-still-usable"),
        Ok(first_context),
        "sibling navigation closure must not revoke the first context's current authority"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-now-usable"),
        Ok(second_context),
        "the sibling authority becomes usable only after its own explicit re-establishment"
    );
}

#[test]
fn duplicate_download_replay_is_individually_non_mutating_across_sibling_navigation() {
    assert_consumed_first_replay_is_non_mutating(ConsumedFirstReplay::DownloadStarted, 1235);
}

#[test]
fn complete_positive_replay_is_individually_non_mutating_across_sibling_navigation() {
    assert_consumed_first_replay_is_non_mutating(ConsumedFirstReplay::Settled, 1236);
}

#[test]
fn aborted_replay_is_individually_non_mutating_across_sibling_navigation() {
    assert_consumed_first_replay_is_non_mutating(ConsumedFirstReplay::Aborted, 1237);
}

#[test]
fn failed_replay_is_individually_non_mutating_across_sibling_navigation() {
    assert_consumed_first_replay_is_non_mutating(ConsumedFirstReplay::Failed, 1238);
}
