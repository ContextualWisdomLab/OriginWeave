use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationSettlementAuthority, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct SecondNavigationReplayProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SecondNavigationReplayProbePort {
    /// Count real lifecycle I/O so rejected evidence cannot hide an adapter side effect.
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handles
            .pop_front()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    /// Treat completion as adapter I/O for the same zero-I/O rejection invariant.
    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }

    /// Count proven destruction separately from later in-memory navigation transitions.
    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(())
    }
}

impl AuthorizedContextOperationPort for SecondNavigationReplayProbePort {
    type Operation = &'static str;
    type Output = BrowsingContextId;
    type Error = ();

    /// Make capability executability observable instead of relying on projection equality alone.
    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        Ok(request.context().browsing_context())
    }
}

#[derive(Clone, Copy)]
enum SecondNavigationProgress {
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

/// Build one adapter-issued disposable handle with an explicit isolation identity.
fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

/// Replay one navigation event that remains bound to the destroyed ownership generation.
fn replay_old_generation(
    bound: &mut originweave_browser_session::BoundBrowserSession<SecondNavigationReplayProbePort>,
    pending: &NavigationSettlementAuthority,
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

/// Prove predecessor evidence stays stale during a later navigation in the recreated generation.
fn assert_predecessor_replay_cannot_hijack_second_navigation(
    progress: SecondNavigationProgress,
    replay: OldGenerationReplay,
    session_id: u64,
) {
    let context = BrowsingContextId::new(session_id * 10 + 1).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SecondNavigationReplayProbePort {
        handles: VecDeque::from([
            handle(context, "second-navigation-old-generation"),
            handle(context, "second-navigation-new-generation"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_id).expect("valid browser session id"),
    )
    .expect("incarnation capacity")
    .bind_lifecycle_port(port);

    let old_authority = bound
        .create_disposable_context()
        .expect("old ownership generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            context,
            old_authority.context_epoch(),
        )
        .expect("old ownership generation enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&old_pending)
        .expect("old ownership generation records commit progress");
    bound
        .destroy_owned_disposable_context(context)
        .expect("proven destruction consumes only the old ownership generation");

    let retained_new_authority = bound
        .create_disposable_context()
        .expect("same raw context id accepted as a new ownership generation");
    assert_eq!(retained_new_authority.context(), old_authority.context());
    assert!(
        retained_new_authority.context_epoch().value() > old_authority.context_epoch().value(),
        "recreated ownership must receive a newer aggregate-issued epoch",
    );

    let first_new_pending = bound
        .record_observed_navigation(
            retained_new_authority.incarnation(),
            context,
            retained_new_authority.context_epoch(),
        )
        .expect("new ownership generation starts its first navigation");
    bound
        .record_observed_navigation_committed(&first_new_pending)
        .expect("first new-generation navigation records commit progress");
    bound
        .record_observed_navigation_settled(&first_new_pending)
        .expect("first new-generation navigation closes with its own witness");
    let first_fresh_authority = bound
        .reestablish_presentation_authority(context)
        .expect("first new-generation navigation mints one fresh authority");
    assert_eq!(
        bound.execute_authorized_context_operation(
            &retained_new_authority,
            "retained-after-first-reestablishment",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the recreated generation's pre-navigation authority stays revoked after first re-establishment",
    );

    let second_new_pending = bound
        .record_observed_navigation(
            first_fresh_authority.incarnation(),
            context,
            first_fresh_authority.context_epoch(),
        )
        .expect("recreated generation starts a second navigation after successful re-establishment");
    if matches!(progress, SecondNavigationProgress::Committed) {
        bound
            .record_observed_navigation_committed(&second_new_pending)
            .expect("second navigation records its own first commit progress");
    }

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_replay = adapter_calls.get();

    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed predecessor evidence must stay generation-stale during a later navigation in B-new",
    );
    assert_eq!(bound.browser_session().state(), state_before_replay);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "predecessor replay must not close the recreated generation's second navigation",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-while-second-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the second navigation keeps the first fresh authority revoked after stale predecessor replay",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    if matches!(progress, SecondNavigationProgress::PendingBeforeCommit) {
        bound
            .record_observed_navigation_committed(&second_new_pending)
            .expect("stale predecessor replay must leave the second navigation's first commit slot available");
        assert_eq!(bound.browser_session().state(), state_before_replay);
        assert_eq!(
            bound.browser_session().recovery_evidence(),
            recovery_before_replay.as_slice(),
        );
        assert_eq!(
            bound.reestablish_presentation_authority(context),
            Err(BrowserSessionError::AuthorityMismatch),
            "second-navigation commit progress remains non-terminal",
        );
        assert_eq!(
            bound.execute_authorized_context_operation(
                &first_fresh_authority,
                "first-fresh-after-second-navigation-commit",
            ),
            Err(AuthorizedContextOperationError::BrowserSession(
                BrowserSessionError::AuthorityMismatch,
            )),
            "second-navigation commit progress must keep the prior fresh authority revoked",
        );
        assert_eq!(adapter_calls.get(), calls_before_replay);
    }

    assert_eq!(
        bound.record_observed_navigation_committed(&second_new_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the second navigation's duplicate commit remains stale regardless of predecessor replay",
    );
    assert_eq!(bound.browser_session().state(), state_before_replay);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "duplicate second-navigation commit must not manufacture eligibility",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-after-second-navigation-duplicate",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "duplicate second-navigation commit must keep the prior authority revoked",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    bound
        .record_observed_navigation_settled(&second_new_pending)
        .expect("only the recreated generation's current second-navigation witness may close it");
    let second_fresh_authority = bound
        .reestablish_presentation_authority(context)
        .expect("second navigation mints exactly one next authority");
    assert_eq!(
        second_fresh_authority.context_epoch().value(),
        first_fresh_authority.context_epoch().value() + 1,
        "stale predecessor replay must not spend a hidden aggregate epoch between navigation cycles",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "second-navigation re-establishment remains single-use",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-after-second-reestablishment",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the authority consumed by the second navigation stays permanently revoked",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    let state_after_second_reestablishment = bound.browser_session().state();
    let recovery_after_second_reestablishment = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "predecessor evidence remains stale after a second complete navigation cycle in B-new",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_second_reestablishment,
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_second_reestablishment.as_slice(),
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "post-cycle predecessor replay must not manufacture a third eligibility",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-after-post-cycle-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "post-cycle predecessor replay must not reactivate the prior navigation authority",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    let projected = bound
        .presentation_authority(context)
        .expect("second fresh authority remains current after stale replay");
    assert_eq!(
        projected, second_fresh_authority,
        "post-cycle stale replay must preserve the exact current authority projection",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_fresh_authority,
            "second-fresh-after-post-cycle-replay",
        ),
        Ok(context),
        "only the authority minted for the second completed navigation remains executable",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay + 1);
}

/// Destroyed-generation replay stays stale across repeated navigations in the recreated generation.
#[test]
fn predecessor_replay_cannot_hijack_a_later_navigation_cycle_after_raw_id_recreation() {
    let progress_states = [
        SecondNavigationProgress::PendingBeforeCommit,
        SecondNavigationProgress::Committed,
    ];
    let replays = [
        OldGenerationReplay::Committed,
        OldGenerationReplay::Settled,
        OldGenerationReplay::Aborted,
        OldGenerationReplay::Failed,
        OldGenerationReplay::DownloadStarted,
    ];
    let mut session_id = 1521;

    for progress in progress_states {
        for replay in replays {
            assert_predecessor_replay_cannot_hijack_second_navigation(progress, replay, session_id);
            session_id += 1;
        }
    }
}
