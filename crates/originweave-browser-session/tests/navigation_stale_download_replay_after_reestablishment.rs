use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationPort, AuthorizedContextOperationRequest, BrowserSession,
    BrowserSessionError, DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
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
) -> originweave_browser_session::BoundBrowserSession<PostReestablishmentReplayProbePort> {
    BrowserSession::start(BrowserSessionId::new(session_id).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(PostReestablishmentReplayProbePort {
            handles,
            adapter_calls,
        })
}

#[test]
fn prior_incarnation_download_replay_after_reestablishment_is_non_mutating() {
    let reused_context = BrowsingContextId::new(1230).expect("valid browsing context");
    let prior_calls = Rc::new(Cell::new(0));
    let current_calls = Rc::new(Cell::new(0));
    let mut prior = bound_session(
        1230,
        VecDeque::from([handle(
            reused_context,
            "post-reestablishment-prior-incarnation",
        )]),
        Rc::clone(&prior_calls),
    );
    let mut current = bound_session(
        1230,
        VecDeque::from([handle(
            reused_context,
            "post-reestablishment-current-incarnation",
        )]),
        Rc::clone(&current_calls),
    );

    let prior_authority = prior
        .create_disposable_context()
        .expect("prior incarnation accepts its context");
    let current_authority = current
        .create_disposable_context()
        .expect("current incarnation accepts its context");
    assert_ne!(prior_authority.incarnation(), current_authority.incarnation());

    let prior_pending = prior
        .record_observed_navigation(
            prior_authority.incarnation(),
            reused_context,
            prior_authority.context_epoch(),
        )
        .expect("prior incarnation issues its own pending witness");
    let current_pending = current
        .record_observed_navigation(
            current_authority.incarnation(),
            reused_context,
            current_authority.context_epoch(),
        )
        .expect("current incarnation enters navigation-pending state");
    current
        .record_observed_navigation_download_started(&current_pending)
        .expect("current download start closes current navigation liveness");
    let fresh = current
        .reestablish_presentation_authority(reused_context)
        .expect("current incarnation explicitly re-establishes presentation authority");

    let state_before_replay = current.browser_session().state();
    let recovery_before_replay = current.browser_session().recovery_evidence().to_vec();
    let calls_before_replay = current_calls.get();
    assert_eq!(
        current.record_observed_navigation_download_started(&prior_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "prior-incarnation download evidence must stay stale after current authority is re-established"
    );
    assert_eq!(current.browser_session().state(), state_before_replay);
    assert_eq!(
        current.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "post-re-establishment stale replay must not manufacture recovery evidence"
    );
    assert_eq!(
        current.reestablish_presentation_authority(reused_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale replay must not manufacture a second re-establishment opportunity"
    );
    assert_eq!(
        current_calls.get(),
        calls_before_replay,
        "rejected stale replay and eligibility probe must remain zero-I/O"
    );
    assert_eq!(
        current
            .execute_authorized_context_operation(&fresh, "fresh-after-prior-replay")
            .expect("stale replay must not revoke current fresh authority"),
        reused_context
    );

    let next_pending = current
        .record_observed_navigation(fresh.incarnation(), reused_context, fresh.context_epoch())
        .expect("fresh authority may begin the next current navigation");
    current
        .record_observed_navigation_download_started(&next_pending)
        .expect("next current download start closes only its own witness");
    let next = current
        .reestablish_presentation_authority(reused_context)
        .expect("next current navigation may re-establish exactly once");
    assert_eq!(
        next.context_epoch().value(),
        fresh.context_epoch().value() + 1,
        "rejected post-reestablishment stale replay must not spend a presentation epoch"
    );
}

#[test]
fn destroyed_generation_download_replay_after_reestablishment_is_non_mutating() {
    let reused_context = BrowsingContextId::new(1231).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let mut bound = bound_session(
        1231,
        VecDeque::from([
            handle(reused_context, "post-reestablishment-old-generation"),
            handle(reused_context, "post-reestablishment-new-generation"),
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
        .expect("old ownership generation issues a pending witness");
    bound
        .destroy_owned_disposable_context(reused_context)
        .expect("proven destruction consumes old ownership");

    let new_authority = bound
        .create_disposable_context()
        .expect("same raw context id is accepted as a fresh ownership generation");
    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            reused_context,
            new_authority.context_epoch(),
        )
        .expect("fresh ownership generation enters navigation-pending state");
    bound
        .record_observed_navigation_download_started(&new_pending)
        .expect("fresh ownership download start closes its own liveness boundary");
    let fresh = bound
        .reestablish_presentation_authority(reused_context)
        .expect("fresh ownership explicitly re-establishes presentation authority");

    let state_before_replay = bound.browser_session().state();
    let recovery_before_replay = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_replay = adapter_calls.get();
    assert_eq!(
        bound.record_observed_navigation_download_started(&old_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed-generation download evidence must remain stale after recreated authority is re-established"
    );
    assert_eq!(bound.browser_session().state(), state_before_replay);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_replay.as_slice(),
        "destroyed-generation replay must not mutate recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(reused_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroyed-generation replay must not recreate eligibility after it was already consumed"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_replay,
        "rejected destroyed-generation replay must fail before adapter I/O"
    );
    assert_eq!(
        bound
            .execute_authorized_context_operation(&fresh, "fresh-after-destroyed-generation-replay")
            .expect("destroyed-generation replay must not revoke current fresh authority"),
        reused_context
    );

    let next_pending = bound
        .record_observed_navigation(fresh.incarnation(), reused_context, fresh.context_epoch())
        .expect("fresh recreated authority may begin the next navigation");
    bound
        .record_observed_navigation_download_started(&next_pending)
        .expect("next current download start closes only the next witness");
    let next = bound
        .reestablish_presentation_authority(reused_context)
        .expect("next current navigation may re-establish exactly once");
    assert_eq!(
        next.context_epoch().value(),
        fresh.context_epoch().value() + 1,
        "rejected destroyed-generation replay must not spend a presentation epoch"
    );
}
