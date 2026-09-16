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

struct PostReestablishmentReplayProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for PostReestablishmentReplayProbePort {
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

impl AuthorizedContextOperationPort for PostReestablishmentReplayProbePort {
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
    bound: &mut originweave_browser_session::BoundBrowserSession<PostReestablishmentReplayProbePort>,
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

fn assert_post_reestablishment_replay_keeps_retained_revoked(
    replay: OldGenerationReplay,
    session_id: u64,
) {
    let context = BrowsingContextId::new(session_id * 10 + 1).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = PostReestablishmentReplayProbePort {
        handles: VecDeque::from([
            handle(context, "aba-retained-old-generation"),
            handle(context, "aba-retained-new-generation"),
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
        .expect("old generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            context,
            old_authority.context_epoch(),
        )
        .expect("old generation navigation accepted");
    bound
        .record_observed_navigation_committed(&old_pending)
        .expect("old generation commit accepted");
    bound
        .destroy_owned_disposable_context(context)
        .expect("old generation destroyed");

    let retained_new_authority = bound
        .create_disposable_context()
        .expect("same raw context id accepted as a new generation");
    let new_pending = bound
        .record_observed_navigation(
            retained_new_authority.incarnation(),
            context,
            retained_new_authority.context_epoch(),
        )
        .expect("new generation navigation accepted");
    bound
        .record_observed_navigation_committed(&new_pending)
        .expect("new generation commit accepted");
    bound
        .record_observed_navigation_settled(&new_pending)
        .expect("new generation settles with its own witness");
    let fresh_authority = bound
        .reestablish_presentation_authority(context)
        .expect("new generation receives one fresh authority");

    assert_eq!(
        bound.execute_authorized_context_operation(
            &retained_new_authority,
            "retained-before-predecessor-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the pre-navigation new-generation authority remains revoked after re-establishment",
    );
    let calls_before_replay = adapter_calls.get();
    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed predecessor evidence remains generation-stale after new-generation re-establishment",
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
        "stale predecessor replay must not manufacture another re-establishment opportunity",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &retained_new_authority,
            "retained-after-predecessor-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale predecessor replay must not reactivate the retained pre-navigation authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "rejected replay and stale retained-capability probe remain zero-I/O",
    );

    let projected = bound
        .presentation_authority(context)
        .expect("fresh authority remains current");
    assert_eq!(projected, fresh_authority);
    assert_eq!(
        bound.execute_authorized_context_operation(&fresh_authority, "fresh-after-stale-replay"),
        Ok(context),
        "only the fresh re-established authority remains executable",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay + 1);
}

#[test]
fn predecessor_replay_after_reestablishment_never_reactivates_retained_authority() {
    let replays = [
        OldGenerationReplay::Committed,
        OldGenerationReplay::Settled,
        OldGenerationReplay::Aborted,
        OldGenerationReplay::Failed,
        OldGenerationReplay::DownloadStarted,
    ];
    let mut session_id = 1491;

    for replay in replays {
        assert_post_reestablishment_replay_keeps_retained_revoked(replay, session_id);
        session_id += 1;
    }
}
