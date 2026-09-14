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

struct HistoricalAuthorityProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for HistoricalAuthorityProbePort {
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

impl AuthorizedContextOperationPort for HistoricalAuthorityProbePort {
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
    bound: &mut originweave_browser_session::BoundBrowserSession<HistoricalAuthorityProbePort>,
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

fn assert_predecessor_replay_never_reactivates_any_historical_new_generation_authority(
    replay: OldGenerationReplay,
    session_id: u64,
) {
    let context = BrowsingContextId::new(session_id * 10 + 1).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = HistoricalAuthorityProbePort {
        handles: VecDeque::from([
            handle(context, "historical-authority-old-generation"),
            handle(context, "historical-authority-new-generation"),
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
        .expect("old generation enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&old_pending)
        .expect("old generation records commit progress");
    bound
        .destroy_owned_disposable_context(context)
        .expect("old ownership generation is proven destroyed");

    let retained_new_authority = bound
        .create_disposable_context()
        .expect("same raw context id is recreated under a new ownership generation");

    let first_pending = bound
        .record_observed_navigation(
            retained_new_authority.incarnation(),
            context,
            retained_new_authority.context_epoch(),
        )
        .expect("new generation starts navigation one");
    bound
        .record_observed_navigation_committed(&first_pending)
        .expect("navigation one records commit progress");
    bound
        .record_observed_navigation_settled(&first_pending)
        .expect("navigation one closes with its own witness");
    let first_fresh_authority = bound
        .reestablish_presentation_authority(context)
        .expect("navigation one mints its fresh authority");

    let second_pending = bound
        .record_observed_navigation(
            first_fresh_authority.incarnation(),
            context,
            first_fresh_authority.context_epoch(),
        )
        .expect("new generation starts navigation two");
    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("navigation two records commit progress");
    bound
        .record_observed_navigation_settled(&second_pending)
        .expect("navigation two closes with its own witness");
    let second_fresh_authority = bound
        .reestablish_presentation_authority(context)
        .expect("navigation two mints its fresh authority");

    let calls_before_replay = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(
            &retained_new_authority,
            "retained-before-predecessor-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the recreated generation's original authority must already be permanently stale",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-before-predecessor-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the authority consumed by navigation two must already be permanently stale",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        replay_old_generation(&mut bound, &old_pending, replay),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed predecessor evidence remains generation-stale after multiple new-generation cycles",
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
        "predecessor replay must never reactivate the recreated generation's oldest retained capability",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_fresh_authority,
            "first-fresh-after-predecessor-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "predecessor replay must never reactivate an authority consumed by a later navigation",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay);

    let projected = bound
        .presentation_authority(context)
        .expect("the newest fresh authority remains current");
    assert_eq!(projected, second_fresh_authority);
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_fresh_authority,
            "second-fresh-after-predecessor-replay",
        ),
        Ok(context),
        "only the newest authority remains executable after stale predecessor replay",
    );
    assert_eq!(adapter_calls.get(), calls_before_replay + 1);
}

#[test]
fn predecessor_replay_cannot_reactivate_any_historical_authority_after_multiple_navigation_cycles() {
    let replays = [
        OldGenerationReplay::Committed,
        OldGenerationReplay::Settled,
        OldGenerationReplay::Aborted,
        OldGenerationReplay::Failed,
        OldGenerationReplay::DownloadStarted,
    ];

    for (offset, replay) in replays.into_iter().enumerate() {
        assert_predecessor_replay_never_reactivates_any_historical_new_generation_authority(
            replay,
            1641 + offset as u64,
        );
    }
}
