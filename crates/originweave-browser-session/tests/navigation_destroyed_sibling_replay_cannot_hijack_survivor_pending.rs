use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DestroyedSiblingPendingProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DestroyedSiblingPendingProbePort {
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

impl AuthorizedContextOperationPort for DestroyedSiblingPendingProbePort {
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

#[derive(Clone, Copy)]
enum SurvivorProgress {
    PendingBeforeCommit,
    Committed,
}

#[derive(Clone, Copy)]
enum DestroyedSiblingReplay {
    Committed,
    Settled,
    Aborted,
    Failed,
    DownloadStarted,
}

fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

fn assert_destroyed_sibling_replay_cannot_hijack_survivor_pending(
    survivor_progress: SurvivorProgress,
    replay: DestroyedSiblingReplay,
    session_id: u64,
) {
    let survivor_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid survivor browsing context");
    let destroyed_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid destroyed browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DestroyedSiblingPendingProbePort {
        handles: VecDeque::from([
            handle(survivor_context, "destroyed-replay-pending-survivor"),
            handle(destroyed_context, "destroyed-replay-pending-sibling"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_id).expect("valid browser session id"),
    )
    .expect("incarnation capacity")
    .bind_lifecycle_port(port);

    let survivor_authority = bound
        .create_disposable_context()
        .expect("survivor disposable context accepted");
    let destroyed_authority = bound
        .create_disposable_context()
        .expect("sibling disposable context accepted");
    let destroyed_pending = bound
        .record_observed_navigation(
            destroyed_authority.incarnation(),
            destroyed_context,
            destroyed_authority.context_epoch(),
        )
        .expect("sibling enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&destroyed_pending)
        .expect("sibling records commit progress before proven destruction");
    bound
        .destroy_owned_disposable_context(destroyed_context)
        .expect("proven destruction consumes only the sibling ownership generation");

    let survivor_pending = bound
        .record_observed_navigation(
            survivor_authority.incarnation(),
            survivor_context,
            survivor_authority.context_epoch(),
        )
        .expect("survivor independently enters navigation-pending state");
    if matches!(survivor_progress, SurvivorProgress::Committed) {
        bound
            .record_observed_navigation_committed(&survivor_pending)
            .expect("survivor records its own first commit progress");
    }

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_replay = adapter_calls.get();

    let replay_result = match replay {
        DestroyedSiblingReplay::Committed => {
            bound.record_observed_navigation_committed(&destroyed_pending)
        }
        DestroyedSiblingReplay::Settled => {
            bound.record_observed_navigation_settled(&destroyed_pending)
        }
        DestroyedSiblingReplay::Aborted => bound.record_observed_navigation_terminated(
            &destroyed_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        DestroyedSiblingReplay::Failed => bound.record_observed_navigation_terminated(
            &destroyed_pending,
            NavigationTerminationOutcome::Failed,
        ),
        DestroyedSiblingReplay::DownloadStarted => {
            bound.record_observed_navigation_download_started(&destroyed_pending)
        }
    };
    assert_eq!(
        replay_result,
        Err(BrowserSessionError::ContextNotOwned),
        "late evidence for a proven-destroyed sibling must fail on ownership before it can touch the survivor's current navigation",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "destroyed-sibling replay must not alter aggregate lifecycle state while a survivor navigation is current",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "destroyed-sibling replay must not manufacture recovery evidence while the survivor is pending",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "destroyed-sibling replay must fail before adapter I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(destroyed_context),
        Err(BrowserSessionError::ContextNotOwned),
        "stale replay must not reconstruct destroyed sibling ownership",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(survivor_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale sibling replay must not close the survivor's current navigation",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &survivor_authority,
            "survivor-retained-authority-stays-revoked",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale sibling replay must not reactivate the survivor's retained pre-navigation authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "ownership and authority probes must remain zero-I/O",
    );

    match survivor_progress {
        SurvivorProgress::PendingBeforeCommit => {
            bound
                .record_observed_navigation_committed(&survivor_pending)
                .expect("stale sibling replay must leave the survivor's first commit slot available");
            assert_eq!(
                bound.browser_session().state(),
                state_before_replay,
                "the survivor's first commit must not alter aggregate lifecycle state",
            );
            assert_eq!(
                bound.browser_session().recovery_evidence(),
                recovery_before_replay.as_slice(),
                "the survivor's first commit must not rewrite recovery evidence",
            );
            assert_eq!(
                bound.reestablish_presentation_authority(survivor_context),
                Err(BrowserSessionError::AuthorityMismatch),
                "the survivor's first non-terminal commit must not manufacture re-establishment eligibility",
            );
            assert_eq!(
                bound.execute_authorized_context_operation(
                    &survivor_authority,
                    "survivor-retained-authority-after-first-commit",
                ),
                Err(AuthorizedContextOperationError::BrowserSession(
                    BrowserSessionError::AuthorityMismatch,
                )),
                "the survivor's first commit must keep retained pre-navigation authority revoked",
            );
            assert_eq!(
                adapter_calls.get(),
                calls_before_replay,
                "the survivor's first commit and immediate authority probes remain zero-I/O",
            );

            assert_eq!(
                bound.record_observed_navigation_committed(&survivor_pending),
                Err(BrowserSessionError::AuthorityMismatch),
                "only the survivor's first matching commit may consume its commit-progress slot",
            );
            assert_eq!(
                bound.browser_session().state(),
                state_before_replay,
                "duplicate survivor commit rejection must not alter aggregate lifecycle state",
            );
            assert_eq!(
                bound.browser_session().recovery_evidence(),
                recovery_before_replay.as_slice(),
                "duplicate survivor commit rejection must not rewrite recovery evidence",
            );
            assert_eq!(
                bound.reestablish_presentation_authority(survivor_context),
                Err(BrowserSessionError::AuthorityMismatch),
                "duplicate survivor commit rejection must not manufacture re-establishment eligibility",
            );
            assert_eq!(
                bound.execute_authorized_context_operation(
                    &survivor_authority,
                    "survivor-retained-authority-after-duplicate-commit",
                ),
                Err(AuthorizedContextOperationError::BrowserSession(
                    BrowserSessionError::AuthorityMismatch,
                )),
                "duplicate survivor commit rejection must keep retained authority revoked",
            );
            assert_eq!(
                adapter_calls.get(),
                calls_before_replay,
                "duplicate survivor commit rejection and immediate authority probes remain zero-I/O",
            );
        }
        SurvivorProgress::Committed => {
            assert_eq!(
                bound.record_observed_navigation_committed(&survivor_pending),
                Err(BrowserSessionError::AuthorityMismatch),
                "stale sibling replay must not reset an already-consumed survivor commit-progress slot",
            );
            assert_eq!(
                bound.browser_session().state(),
                state_before_replay,
                "duplicate survivor commit rejection must not alter aggregate lifecycle state",
            );
            assert_eq!(
                bound.browser_session().recovery_evidence(),
                recovery_before_replay.as_slice(),
                "duplicate survivor commit rejection must not rewrite recovery evidence",
            );
            assert_eq!(
                bound.reestablish_presentation_authority(survivor_context),
                Err(BrowserSessionError::AuthorityMismatch),
                "duplicate survivor commit rejection must not manufacture re-establishment eligibility",
            );
            assert_eq!(
                bound.execute_authorized_context_operation(
                    &survivor_authority,
                    "committed-survivor-retained-authority-after-duplicate",
                ),
                Err(AuthorizedContextOperationError::BrowserSession(
                    BrowserSessionError::AuthorityMismatch,
                )),
                "duplicate survivor commit rejection must keep retained authority revoked",
            );
            assert_eq!(
                adapter_calls.get(),
                calls_before_replay,
                "duplicate survivor commit rejection and immediate authority probes remain zero-I/O",
            );
        }
    }
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "survivor commit bookkeeping must remain an in-memory navigation transition",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "survivor commit bookkeeping must not rewrite recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "survivor commit progress and duplicate rejection remain zero-I/O",
    );

    bound
        .record_observed_navigation_settled(&survivor_pending)
        .expect("the survivor must still close using its own current witness");
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "survivor settlement must remain zero-I/O",
    );

    let reestablished = bound
        .reestablish_presentation_authority(survivor_context)
        .expect("only the survivor's own qualified closure may mint fresh authority");
    assert_eq!(
        reestablished.context_epoch().value(),
        destroyed_authority.context_epoch().value() + 1,
        "destroyed-sibling replay must not spend a hidden aggregate presentation epoch",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(survivor_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "survivor re-establishment eligibility remains single-use",
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "successful survivor settlement and re-establishment keep aggregate trust active",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "survivor re-establishment must not rewrite lifecycle recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "explicit survivor re-establishment remains zero-I/O",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "survivor-current"),
        Ok(survivor_context),
        "fresh survivor authority must be executable after hostile sibling replay",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay + 1);
}

#[test]
fn destroyed_sibling_replay_cannot_hijack_survivor_pending_or_committed_progress() {
    let progress_states = [SurvivorProgress::PendingBeforeCommit, SurvivorProgress::Committed];
    let replays = [
        DestroyedSiblingReplay::Committed,
        DestroyedSiblingReplay::Settled,
        DestroyedSiblingReplay::Aborted,
        DestroyedSiblingReplay::Failed,
        DestroyedSiblingReplay::DownloadStarted,
    ];
    let mut session_id = 1411;

    for survivor_progress in progress_states {
        for replay in replays {
            assert_destroyed_sibling_replay_cannot_hijack_survivor_pending(
                survivor_progress,
                replay,
                session_id,
            );
            session_id += 1;
        }
    }
}
