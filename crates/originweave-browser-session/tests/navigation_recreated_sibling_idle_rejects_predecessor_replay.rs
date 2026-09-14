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

struct IdleRecreationReplayProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for IdleRecreationReplayProbePort {
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

impl AuthorizedContextOperationPort for IdleRecreationReplayProbePort {
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
    bound: &mut originweave_browser_session::BoundBrowserSession<IdleRecreationReplayProbePort>,
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

fn assert_idle_recreated_generation_rejects_predecessor_replay(
    replay: OldGenerationReplay,
    session_id: u64,
) {
    let sibling_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid sibling browsing context");
    let recreated_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid recreated browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = IdleRecreationReplayProbePort {
        handles: VecDeque::from([
            handle(sibling_context, "idle-aba-independent-sibling"),
            handle(recreated_context, "idle-aba-old-generation"),
            handle(recreated_context, "idle-aba-new-generation"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_id).expect("valid browser session id"),
    )
    .expect("incarnation capacity")
    .bind_lifecycle_port(port);

    let sibling_authority = bound
        .create_disposable_context()
        .expect("independent sibling accepted");
    let old_authority = bound
        .create_disposable_context()
        .expect("old generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            recreated_context,
            old_authority.context_epoch(),
        )
        .expect("old generation enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&old_pending)
        .expect("old generation records its first commit progress");
    bound
        .destroy_owned_disposable_context(recreated_context)
        .expect("old generation is proven destroyed");

    let new_authority = bound
        .create_disposable_context()
        .expect("same raw context id is accepted as a fresh ownership generation");
    assert_eq!(new_authority.context(), old_authority.context());
    assert!(new_authority.context_epoch().value() > old_authority.context_epoch().value());

    let state_before_idle_replay = bound.browser_session().state();
    let recovery_before_idle_replay = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_idle_replay = adapter_calls.get();

    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "old-generation evidence must remain stale even while the recreated generation is idle",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_idle_replay,
        "idle stale replay must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_idle_replay.as_slice(),
        "idle stale replay must not rewrite recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_idle_replay,
        "idle stale replay must fail before adapter I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale predecessor evidence must not manufacture eligibility for an idle recreated generation",
    );
    let new_current_after_idle_replay = bound
        .presentation_authority(recreated_context)
        .expect("idle stale replay must preserve the recreated generation's current authority");
    assert_eq!(
        new_current_after_idle_replay, new_authority,
        "idle stale replay must preserve the complete recreated-generation authority identity",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_idle_replay,
        "idle eligibility and authority projection probes remain zero-I/O",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&new_authority, "idle-new-generation-current"),
        Ok(recreated_context),
        "the idle recreated-generation authority must remain executable after stale predecessor replay",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&sibling_authority, "idle-independent-sibling"),
        Ok(sibling_context),
        "stale predecessor replay must not revoke an unrelated sibling authority",
    );
    let calls_after_idle_operations = adapter_calls.get();
    assert_eq!(calls_after_idle_operations, calls_before_idle_replay + 2);

    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            recreated_context,
            new_authority.context_epoch(),
        )
        .expect("idle recreated generation must still be able to start its own navigation");
    assert_eq!(
        bound.execute_authorized_context_operation(
            &new_authority,
            "new-generation-retained-authority-after-navigation-start",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the recreated generation's own navigation must revoke its retained authority",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigation start alone must not create re-establishment eligibility",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_idle_operations,
        "navigation start and immediate authority probes remain zero-I/O",
    );

    bound
        .record_observed_navigation_committed(&new_pending)
        .expect("the recreated generation must retain its own first commit slot after idle stale replay");
    assert_eq!(
        bound.record_observed_navigation_committed(&new_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the recreated generation's duplicate commit remains stale",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "commit progress remains non-terminal after idle stale replay",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_idle_operations,
        "commit progress, duplicate rejection, and eligibility probe remain zero-I/O",
    );

    bound
        .record_observed_navigation_settled(&new_pending)
        .expect("only the recreated generation's own witness may close its navigation");
    let new_reestablished = bound
        .reestablish_presentation_authority(recreated_context)
        .expect("the recreated generation receives exactly one fresh authority after its own closure");
    assert_eq!(
        new_reestablished.context_epoch().value(),
        new_authority.context_epoch().value() + 1,
        "idle predecessor replay must not spend a hidden aggregate presentation epoch",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "recreated-generation eligibility remains single-use",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_idle_operations,
        "closure and re-establishment remain zero-I/O",
    );

    let state_before_post_reestablishment_replay = bound.browser_session().state();
    let recovery_before_post_reestablishment_replay =
        bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "old-generation evidence remains stale after the recreated generation re-establishes authority",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_post_reestablishment_replay,
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_post_reestablishment_replay.as_slice(),
    );
    assert_eq!(
        bound.reestablish_presentation_authority(recreated_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "post-reestablishment stale replay must not manufacture another eligibility",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_idle_operations,
        "post-reestablishment stale replay and eligibility probe remain zero-I/O",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&new_reestablished, "new-generation-fresh"),
        Ok(recreated_context),
        "the fresh recreated-generation authority must remain executable after stale replay",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&sibling_authority, "independent-sibling-still-current"),
        Ok(sibling_context),
        "the independent sibling authority must remain executable throughout the ABA sequence",
    );
    assert_eq!(adapter_calls.get(), calls_after_idle_operations + 2);
}

#[test]
fn idle_recreated_generation_rejects_every_predecessor_navigation_replay() {
    let replays = [
        OldGenerationReplay::Committed,
        OldGenerationReplay::Settled,
        OldGenerationReplay::Aborted,
        OldGenerationReplay::Failed,
        OldGenerationReplay::DownloadStarted,
    ];
    let mut session_id = 1441;

    for replay in replays {
        assert_idle_recreated_generation_rejects_predecessor_replay(replay, session_id);
        session_id += 1;
    }
}
