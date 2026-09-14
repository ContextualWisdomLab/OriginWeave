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

struct DestroyedSiblingReplayProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DestroyedSiblingReplayProbePort {
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

impl AuthorizedContextOperationPort for DestroyedSiblingReplayProbePort {
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
enum NavigationClosure {
    Settled,
    Aborted,
    Failed,
    DownloadStarted,
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

fn assert_destroyed_sibling_replay_preserves_survivor_eligibility(
    closure: NavigationClosure,
    replay: DestroyedSiblingReplay,
    session_id: u64,
) {
    let first_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid first browsing context");
    let second_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid second browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DestroyedSiblingReplayProbePort {
        handles: VecDeque::from([
            handle(first_context, "destroyed-replay-survivor-user-context"),
            handle(second_context, "destroyed-replay-sibling-user-context"),
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
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first context enters navigation-pending state");

    match closure {
        NavigationClosure::Settled => bound
            .record_observed_navigation_settled(&first_pending)
            .expect("positive closure creates survivor re-establishment eligibility"),
        NavigationClosure::Aborted => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Aborted,
            )
            .expect("aborted closure creates survivor re-establishment eligibility"),
        NavigationClosure::Failed => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Failed,
            )
            .expect("failed closure creates survivor re-establishment eligibility"),
        NavigationClosure::DownloadStarted => bound
            .record_observed_navigation_download_started(&first_pending)
            .expect("download start creates survivor re-establishment eligibility"),
    }

    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("sibling context independently enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("sibling context records non-terminal commit progress");
    bound
        .destroy_owned_disposable_context(second_context)
        .expect("proven destruction consumes only the sibling ownership generation");

    let state_after_destroy = bound.browser_session().state();
    let recovery_after_destroy = bound.browser_session().recovery_evidence().to_vec();
    let calls_after_destroy = adapter_calls.get();

    let replay_result = match replay {
        DestroyedSiblingReplay::Committed => {
            bound.record_observed_navigation_committed(&second_pending)
        }
        DestroyedSiblingReplay::Settled => {
            bound.record_observed_navigation_settled(&second_pending)
        }
        DestroyedSiblingReplay::Aborted => bound.record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        DestroyedSiblingReplay::Failed => bound.record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Failed,
        ),
        DestroyedSiblingReplay::DownloadStarted => {
            bound.record_observed_navigation_download_started(&second_pending)
        }
    };
    assert_eq!(
        replay_result,
        Err(BrowserSessionError::ContextNotOwned),
        "late navigation evidence for a proven-destroyed sibling must fail on ownership before mutation",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_destroy,
        "destroyed-sibling replay must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_destroy.as_slice(),
        "destroyed-sibling replay must not manufacture or rewrite recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "destroyed-sibling replay must fail before adapter I/O",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_authority,
            "survivor-retained-authority-after-destroyed-sibling-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "stale sibling evidence must not reactivate the survivor's retained authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "retained survivor authority must remain rejected before adapter I/O",
    );

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("destroyed-sibling replay must preserve the survivor's unused eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "stale sibling evidence must neither spend nor reset the aggregate presentation epoch",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_destroy,
        "survivor re-establishment remains an in-memory authority transition",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_destroy.as_slice(),
        "survivor re-establishment must not rewrite lifecycle recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "survivor re-establishment must remain zero-I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "survivor eligibility remains single-use after hostile sibling replay",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "duplicate survivor re-establishment must fail before adapter I/O",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "survivor-current"),
        Ok(first_context),
        "the preserved survivor eligibility must mint executable fresh authority",
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy + 1);
}

#[test]
fn destroyed_sibling_replay_cannot_consume_survivor_reestablishment_eligibility() {
    let closures = [
        NavigationClosure::Settled,
        NavigationClosure::Aborted,
        NavigationClosure::Failed,
        NavigationClosure::DownloadStarted,
    ];
    let replays = [
        DestroyedSiblingReplay::Committed,
        DestroyedSiblingReplay::Settled,
        DestroyedSiblingReplay::Aborted,
        DestroyedSiblingReplay::Failed,
        DestroyedSiblingReplay::DownloadStarted,
    ];
    let mut session_id = 1391;

    for closure in closures {
        for replay in replays {
            assert_destroyed_sibling_replay_preserves_survivor_eligibility(
                closure, replay, session_id,
            );
            session_id += 1;
        }
    }
}
