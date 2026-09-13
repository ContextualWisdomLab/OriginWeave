use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct SupersededCommitProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SupersededCommitProbePort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.handle
            .take()
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

impl AuthorizedContextOperationPort for SupersededCommitProbePort {
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

#[test]
fn superseded_commit_progress_cannot_poison_the_current_pending_navigation() {
    let context = BrowsingContextId::new(1111).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SupersededCommitProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("superseded-commit-user-context-1111")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1111).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation start issues a pending witness");
    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("newer same-context navigation supersedes the first pending witness");
    let recovery_after_supersession = bound.browser_session().recovery_evidence().to_vec();

    assert_eq!(
        bound.record_observed_navigation_committed(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "commit progress from a superseded witness must not attach to the current pending navigation"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice(),
        "rejected superseded commit evidence must not manufacture recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejected superseded commit evidence must leave the newer navigation pending"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-superseded-commit"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "rejected superseded commit evidence must not reactivate retained pre-navigation authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "superseded commit validation and authority rejection must happen before adapter I/O"
    );

    bound
        .record_observed_navigation_committed(&second_pending)
        .expect("the current pending witness still accepts its own commit progress");
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "current commit progress remains non-terminal and non-authorizing"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_supersession.as_slice(),
        "valid commit progress must not mutate lifecycle recovery evidence"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    bound
        .record_observed_navigation_settled(&second_pending)
        .expect("only the current witness may close the navigation positively");
    let fresh = bound
        .reestablish_presentation_authority(context)
        .expect("current terminal evidence permits one explicit fresh authority");
    assert_eq!(
        fresh.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "rejected stale commit progress and current commit progress must not spend presentation epochs"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    assert_eq!(
        bound.execute_authorized_context_operation(&fresh, "usable-after-current-terminal"),
        Ok(context),
        "the authority minted from the current navigation must remain executable"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create + 1,
        "only the final authorized operation performs browser I/O"
    );
}
