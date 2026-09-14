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

struct SiblingDestructionEligibilityProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SiblingDestructionEligibilityProbePort {
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

impl AuthorizedContextOperationPort for SiblingDestructionEligibilityProbePort {
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

fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

fn assert_sibling_destruction_preserves_reestablishment_eligibility(
    closure: NavigationClosure,
    session_id: u64,
) {
    let first_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid first browsing context");
    let second_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid second browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SiblingDestructionEligibilityProbePort {
        handles: VecDeque::from([
            handle(first_context, "eligibility-survivor-user-context"),
            handle(second_context, "eligibility-destroyed-sibling-user-context"),
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
            .expect("positive closure creates one first-context re-establishment opportunity"),
        NavigationClosure::Aborted => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Aborted,
            )
            .expect("aborted closure creates one first-context re-establishment opportunity"),
        NavigationClosure::Failed => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Failed,
            )
            .expect("failed closure creates one first-context re-establishment opportunity"),
        NavigationClosure::DownloadStarted => bound
            .record_observed_navigation_download_started(&first_pending)
            .expect("download start creates one first-context re-establishment opportunity"),
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
        .expect("sibling context records its own non-terminal commit progress");

    let state_before_destroy = bound.browser_session().state();
    let recovery_before_destroy = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_destroy = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_authority,
            "first-retained-authority-before-sibling-destruction",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "terminal eligibility must not reactivate the first context's retained authority",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the committed sibling remains ineligible before its own qualified closure",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-retained-authority-before-own-destruction",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the committed sibling's retained authority remains revoked",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy,
        "pre-destruction authority probes must fail before adapter I/O",
    );

    bound
        .destroy_owned_disposable_context(second_context)
        .expect("proven destruction consumes only the sibling ownership generation");
    assert_eq!(
        bound.browser_session().state(),
        state_before_destroy,
        "destroying a sibling must preserve aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_destroy.as_slice(),
        "proven sibling destruction must not manufacture recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven sibling destruction performs exactly one lifecycle adapter call",
    );
    let calls_after_destroy = adapter_calls.get();

    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_authority,
            "first-retained-authority-after-sibling-destruction",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "destroying a sibling must not reactivate the first context's retained authority",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "retained first-context authority must still fail before adapter I/O",
    );

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("sibling destruction must preserve the already-earned first-context eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "sibling destruction must not spend or reset the aggregate presentation epoch",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_destroy,
        "successful first-context re-establishment must not mutate aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_destroy.as_slice(),
        "successful first-context re-establishment must not rewrite lifecycle recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "successful re-establishment is an in-memory authority transition and must remain zero-I/O",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the preserved first-context eligibility remains single-use",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::ContextNotOwned),
        "destroyed sibling ownership cannot be resurrected by navigation state",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "re-establishment gating remains zero-I/O",
    );

    assert_eq!(
        bound.record_observed_navigation_committed(&second_pending),
        Err(BrowserSessionError::ContextNotOwned),
        "late commit evidence for the destroyed sibling is permanently stale",
    );
    assert_eq!(
        bound.browser_session().state(),
        state_before_destroy,
        "late destroyed-sibling evidence must not alter aggregate lifecycle state",
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_destroy.as_slice(),
        "late destroyed-sibling evidence must not rewrite recovery evidence",
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);

    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "first-current"),
        Ok(first_context),
        "the eligibility preserved across sibling destruction must yield executable authority",
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy + 1);
}

#[test]
fn positive_reestablishment_eligibility_survives_sibling_destruction() {
    assert_sibling_destruction_preserves_reestablishment_eligibility(
        NavigationClosure::Settled,
        1371,
    );
}

#[test]
fn aborted_reestablishment_eligibility_survives_sibling_destruction() {
    assert_sibling_destruction_preserves_reestablishment_eligibility(
        NavigationClosure::Aborted,
        1372,
    );
}

#[test]
fn failed_reestablishment_eligibility_survives_sibling_destruction() {
    assert_sibling_destruction_preserves_reestablishment_eligibility(
        NavigationClosure::Failed,
        1373,
    );
}

#[test]
fn download_reestablishment_eligibility_survives_sibling_destruction() {
    assert_sibling_destruction_preserves_reestablishment_eligibility(
        NavigationClosure::DownloadStarted,
        1374,
    );
}
