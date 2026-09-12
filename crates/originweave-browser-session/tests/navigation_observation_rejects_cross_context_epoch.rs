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

struct CrossContextEpochProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CrossContextEpochProbePort {
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

impl AuthorizedContextOperationPort for CrossContextEpochProbePort {
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
fn sibling_context_epoch_cannot_invalidate_another_owned_context() {
    let first_context = BrowsingContextId::new(961).expect("valid first context");
    let second_context = BrowsingContextId::new(962).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CrossContextEpochProbePort {
        handles: VecDeque::from([
            handle(first_context, "cross-context-epoch-user-context-961"),
            handle(second_context, "cross-context-epoch-user-context-962"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(961).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    assert_ne!(
        first_authority.context_epoch(),
        second_authority.context_epoch(),
        "independently created owned contexts must carry distinct aggregate-issued epochs"
    );

    let calls_before_cross_context_observation = adapter_calls.get();
    assert_eq!(
        bound.record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            second_authority.context_epoch(),
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "an epoch observed for a sibling context is provenance for that sibling only and cannot revoke another context"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_cross_context_observation,
        "cross-context epoch confusion must fail before adapter I/O"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&first_authority, "first-still-current"),
        Ok(first_context),
        "rejecting sibling provenance must leave the first context authority current"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_authority, "second-still-current"),
        Ok(second_context),
        "rejecting sibling provenance must not mutate the sibling context either"
    );

    let calls_before_real_navigation = adapter_calls.get();
    bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("the exact first-context generation may invalidate its own presentation authority");
    assert_eq!(
        adapter_calls.get(),
        calls_before_real_navigation,
        "valid navigation invalidation is also a zero-I/O domain transition"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&first_authority, "first-now-stale"),
        Err(originweave_browser_session::AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the exact matching navigation invalidates only the first context"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_real_navigation,
        "stale first-context authority must be rejected before adapter I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_authority, "second-remains-current"),
        Ok(second_context),
        "first-context navigation must not revoke the sibling context"
    );
}
