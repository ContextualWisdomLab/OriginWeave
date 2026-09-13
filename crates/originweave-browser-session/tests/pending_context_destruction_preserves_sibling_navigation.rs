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

struct PendingContextDestructionProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for PendingContextDestructionProbePort {
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

impl AuthorizedContextOperationPort for PendingContextDestructionProbePort {
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

#[test]
fn destroying_one_pending_context_preserves_sibling_pending_navigation() {
    let first_context = BrowsingContextId::new(969).expect("valid first context");
    let second_context = BrowsingContextId::new(970).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = PendingContextDestructionProbePort {
        handles: VecDeque::from([
            handle(
                first_context,
                "cross-context-destroyed-pending-user-context-969",
            ),
            handle(
                second_context,
                "cross-context-surviving-pending-user-context-970",
            ),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(969).expect("valid session id"))
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
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("second context independently enters navigation-pending state");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(first_context)
        .expect("proven destruction consumes only the first pending context ownership");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "successful destruction of one pending context must keep aggregate trust active"
    );
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::ContextNotOwned),
        "late positive terminal evidence cannot settle a proven-destroyed pending context"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "rejected positive replay must not rewrite aggregate state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "rejected positive replay must not add recovery evidence"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::ContextNotOwned),
        "late aborted terminal evidence cannot terminate a proven-destroyed pending context"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "rejected aborted replay must not rewrite aggregate state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "rejected aborted replay must not add recovery evidence"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::ContextNotOwned),
        "late failed terminal evidence cannot terminate a proven-destroyed pending context"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "rejected failed replay must not rewrite aggregate state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "rejected failed replay must not add recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::ContextNotOwned),
        "destroyed pending context cannot regain presentation authority"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "destroying one pending context must not clear the sibling pending navigation"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "surviving-sibling-stale-while-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the sibling's retained pre-navigation authority stays revoked while its own navigation is pending"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "post-destruction authority rejection must keep aggregate trust active"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "post-destruction authority rejection must preserve recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destruction witness replay and sibling authority checks must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_terminated(
            &second_pending,
            NavigationTerminationOutcome::Failed,
        )
        .expect("the surviving sibling may still terminate its own pending navigation");
    assert_eq!(adapter_calls.get(), calls_after_destroy);
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "the surviving sibling's valid terminal must not fabricate recovery evidence"
    );

    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("the surviving sibling may re-establish after its own terminal outcome");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "destroying a pending sibling context must not consume an aggregate presentation epoch"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "successful sibling re-establishment must not mutate recovery evidence"
    );
    assert_eq!(adapter_calls.get(), calls_after_destroy);
}
