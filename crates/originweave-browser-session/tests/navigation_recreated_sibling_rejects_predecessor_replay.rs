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

struct RecreatedSiblingReplayProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for RecreatedSiblingReplayProbePort {
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

impl AuthorizedContextOperationPort for RecreatedSiblingReplayProbePort {
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
enum RecreatedSiblingProgress {
    PendingBeforeCommit,
    Committed,
}

#[derive(Clone, Copy)]
enum OldGenerationReplay {
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

fn replay_old_generation(
    bound: &mut originweave_browser_session::BoundBrowserSession<RecreatedSiblingReplayProbePort>,
    pending: &originweave_browser_session::NavigationSettlementAuthority,
    replay: OldGenerationReplay,
) -> Result<(), BrowserSessionError> {
    match replay {
        OldGenerationReplay::Committed => bound.record_observed_navigation_committed(pending),
        OldGenerationReplay::Settled => bound.record_observed_navigation_settled(pending),
        OldGenerationReplay::Aborted => bound.record_observed_navigation_terminated(
            pending,
            NavigationTerminationOutcome::Aborted,
        ),
        OldGenerationReplay::Failed => bound.record_observed_navigation_terminated(
            pending,
            NavigationTerminationOutcome::Failed,
        ),
        OldGenerationReplay::DownloadStarted => {
            bound.record_observed_navigation_download_started(pending)
        }
    }
}

fn assert_old_generation_replay_cannot_hijack_recreated_sibling(
    progress: RecreatedSiblingProgress,
    replay: OldGenerationReplay,
    session_id: u64,
) {
    let survivor_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid survivor browsing context");
    let recreated_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid recreated browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = RecreatedSiblingReplayProbePort {
        handles: VecDeque::from([
            handle(survivor_context, "aba-replay-survivor-context"),
            handle(recreated_context, "aba-replay-old-generation"),
            handle(recreated_context, "aba-replay-new-generation"),
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
    let old_authority = bound
        .create_disposable_context()
        .expect("old sibling generation accepted");

    let survivor_pending = bound
        .record_observed_navigation(
            survivor_authority.incarnation(),
            survivor_context,
            survivor_authority.context_epoch(),
        )
        .expect("survivor enters navigation-pending state");
    bound
        .record_observed_navigation_settled(&survivor_pending)
        .expect("survivor earns one unused re-establishment opportunity");

    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            recreated_context,
            old_authority.context_epoch(),
        )
        .expect("old sibling generation enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&old_pending)
        .expect("old sibling generation records its first commit progress");
    bound
        .destroy_owned_disposable_context(recreated_context)
        .expect("proven destruction consumes only the old ownership generation");

    let new_authority = bound
        .create_disposable_context()
        .expect("the same raw context id is accepted as a fresh ownership generation");
    assert_eq!(new_authority.context(), old_authority.context());
    assert!(new_authority.context_epoch().value() > old_authority.context_epoch().value());

    let survivor_reestablished = bound
        .reestablish_presentation_authority(survivor_context)
        .expect("sibling recreation must preserve the survivor's unused eligibility");
    assert_eq!(
        survivor_reestablished.context_epoch().value(),
        new_authority.context_epoch().value() + 1,
        "survivor re-establishment must use the next aggregate-issued epoch",
    );

    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            recreated_context,
            new_authority.context_epoch(),
        )
        .expect("new ownership generation enters its own navigation-pending state");
    if matches!(progress, RecreatedSiblingProgress::Committed) {
        bound
            .record_observed_navigation_committed(&new_pending)
            .expect("new ownership generation records its own first commit progress");
    }

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_replay = adapter_calls.get();

    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigation evidence from the destroyed ownership generation must remain stale after same-raw-id recreation",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "old-generation replay must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "old-generation replay must not rewrite lifecycle recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "old-generation replay must fail before adapter I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "old-generation replay must not close the new generation's current navigation",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &new_authority,
            "new-generation-retained-authority-after-old-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "old-generation replay must not reactivate the new generation's retained authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "authority and eligibility probes after old-generation replay remain zero-I/O",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&survivor_reestablished, "survivor-current"),
        Ok(survivor_context),
        "old-generation replay must not revoke the independent survivor authority",
    );
    let calls_after_survivor_operation = adapter_calls.get();

    match progress {
        RecreatedSiblingProgress::PendingBeforeCommit => {
            bound
                .record_observed_navigation_committed(&new_pending)
                .expect("old-generation replay must leave the new generation's first commit slot available");
            assert_eq!(
                bound.browser_session().state(),
                state_before_replay,
                "new-generation first commit must not alter aggregate lifecycle state",
            );
            assert_eq!(
                bound.browser_session().recovery_evidence(),
                recovery_before_replay.as_slice(),
                "new-generation first commit must not rewrite recovery evidence",
            );
            assert_eq!(
                bound.reestablish_presentation_authority(recreated_context),
                Err(BrowserSessionError::AuthorityMismatch),
                "new-generation first commit remains non-terminal",
            );
            assert_eq!(
                bound.execute_authorized_context_operation(
                    &new_authority,
                    "new-generation-retained-authority-after-first-commit",
                ),
                Err(AuthorizedContextOperationError::BrowserSession(
                    BrowserSessionError::AuthorityMismatch,
                )),
                "new-generation first commit must keep retained authority revoked",
            );
            assert_eq!(
                adapter_calls.get(),
                calls_after_survivor_operation,
                "new-generation commit bookkeeping and authority probes remain zero-I/O",
            );
        }
        RecreatedSiblingProgress::Committed => {}
    }

    assert_eq!(
        bound.record_observed_navigation_committed(&new_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "old-generation replay must neither reset nor consume the new generation's own commit-progress slot",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_replay,
        "duplicate new-generation commit rejection must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "duplicate new-generation commit rejection must not rewrite recovery evidence",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate commit rejection must not manufacture new-generation eligibility",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &new_authority,
            "new-generation-retained-authority-after-duplicate",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "duplicate commit rejection must keep the retained new-generation authority revoked",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_survivor_operation,
        "duplicate commit rejection and immediate authority probes remain zero-I/O",
    );

    bound
        .record_observed_navigation_settled(&new_pending)
        .expect("only the new generation's own witness may close its current navigation");
    assert_eq!(
        adapter_calls.get(),
        calls_after_survivor_operation,
        "new-generation settlement remains zero-I/O",
    );

    let new_reestablished = bound
        .reestablish_presentation_authority(recreated_context)
        .expect("new generation receives exactly one authority after its own closure");
    assert_eq!(
        new_reestablished.context_epoch().value(),
        survivor_reestablished.context_epoch().value() + 1,
        "old-generation replay must not spend a hidden aggregate presentation epoch",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "new-generation re-establishment eligibility remains single-use",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &new_authority,
            "new-generation-retained-authority-after-reestablishment",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "re-establishment must not reactivate the pre-navigation new-generation authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_survivor_operation,
        "new-generation re-establishment and retained-authority rejection remain zero-I/O",
    );

    let state_after_reestablishment = bound.browser_session().state();
    let recovery_after_reestablishment = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_post_reestablishment_replay = adapter_calls.get();
    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "old-generation evidence remains stale after the recreated generation re-establishes authority",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_reestablishment,
        "post-reestablishment old-generation replay must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_reestablishment.as_slice(),
        "post-reestablishment old-generation replay must not rewrite recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_post_reestablishment_replay,
        "post-reestablishment old-generation replay must fail before adapter I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "post-reestablishment old-generation replay must not manufacture a second eligibility",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &new_authority,
            "new-generation-retained-authority-after-post-reestablishment-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "old-generation replay must not reactivate the retained new-generation authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_post_reestablishment_replay,
        "post-reestablishment authority probes remain zero-I/O",
    );

    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.execute_authorized_context_operation(&new_reestablished, "new-generation-current"),
        Ok(recreated_context),
        "only the fresh authority minted for the recreated generation may execute",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&survivor_reestablished, "survivor-still-current"),
        Ok(survivor_context),
        "recreated-generation navigation and stale predecessor replay must not revoke the sibling survivor",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_post_reestablishment_replay + 2,
    );
}

#[test]
fn old_generation_replay_cannot_hijack_recreated_sibling_navigation() {
    let progress_states = [
        RecreatedSiblingProgress::PendingBeforeCommit,
        RecreatedSiblingProgress::Committed,
    ];
    let replays = [
        OldGenerationReplay::Committed,
        OldGenerationReplay::Settled,
        OldGenerationReplay::Aborted,
        OldGenerationReplay::Failed,
        OldGenerationReplay::DownloadStarted,
    ];
    let mut session_id = 1421;

    for progress in progress_states {
        for replay in replays {
            assert_old_generation_replay_cannot_hijack_recreated_sibling(
                progress, replay, session_id,
            );
            session_id += 1;
        }
    }
}
