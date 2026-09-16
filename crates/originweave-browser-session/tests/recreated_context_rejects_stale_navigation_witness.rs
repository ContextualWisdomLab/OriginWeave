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

struct RecreatedContextProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for RecreatedContextProbePort {
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

impl AuthorizedContextOperationPort for RecreatedContextProbePort {
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
fn recreated_raw_context_id_rejects_navigation_witness_from_destroyed_ownership_generation() {
    let context = BrowsingContextId::new(971).expect("valid context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = RecreatedContextProbePort {
        handles: VecDeque::from([
            handle(context, "recreated-context-old-ownership-971"),
            handle(context, "recreated-context-new-ownership-971"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(971).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let old_authority = bound
        .create_disposable_context()
        .expect("first ownership generation accepted");
    let old_pending = bound
        .record_observed_navigation(
            old_authority.incarnation(),
            context,
            old_authority.context_epoch(),
        )
        .expect("first ownership generation enters navigation-pending state");

    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(context)
        .expect("proven destruction consumes first ownership generation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven destruction performs exactly one lifecycle adapter call"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    let calls_after_destroy = adapter_calls.get();
    let recovery_evidence_after_destroy = bound.browser_session().recovery_evidence().to_vec();

    let new_authority = bound
        .create_disposable_context()
        .expect("same raw browsing-context id may be accepted as a fresh ownership generation");
    assert_eq!(
        new_authority.context(),
        old_authority.context(),
        "the hostile case intentionally reuses the same raw browsing-context id"
    );
    assert!(
        new_authority.context_epoch().value() > old_authority.context_epoch().value(),
        "fresh ownership must carry a newer aggregate-issued context epoch"
    );
    let new_pending = bound
        .record_observed_navigation(
            new_authority.incarnation(),
            context,
            new_authority.context_epoch(),
        )
        .expect("fresh ownership generation enters its own navigation-pending state");
    let calls_after_new_pending = adapter_calls.get();

    assert_eq!(
        bound.record_observed_navigation_settled(&old_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale positive terminal evidence from the destroyed ownership generation must not settle the recreated context"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &old_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale aborted evidence from the destroyed ownership generation must not terminate the recreated context"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &old_pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale failed evidence from the destroyed ownership generation must not terminate the recreated context"
    );
    assert_eq!(
        bound
            .execute_authorized_context_operation(&old_authority, "stale-old-generation-authority"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the destroyed generation's retained presentation authority must not authorize the recreated raw context id"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "stale witness replay must not clear the recreated context's current pending navigation"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::Active,
        "ABA-style raw context reuse must not rewrite aggregate trust"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "stale prior-generation evidence must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_new_pending,
        "stale prior-generation witness and authority checks must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_settled(&new_pending)
        .expect("only the recreated ownership generation's current witness may settle navigation");
    assert_eq!(adapter_calls.get(), calls_after_new_pending);
    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("fresh ownership may re-establish after its own terminal outcome");
    assert!(
        reestablished.context_epoch().value() > new_authority.context_epoch().value(),
        "re-establishment must advance the aggregate-issued epoch rather than reviving an earlier ownership generation"
    );
    assert_eq!(bound.browser_session().state(), BrowserSessionState::Active);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_destroy.as_slice(),
        "valid fresh-generation settlement and re-establishment must not fabricate recovery evidence"
    );
    assert_eq!(adapter_calls.get(), calls_after_new_pending);
}
