use std::cell::{Cell, RefCell};
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    BrowserSessionIncarnation, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct OperationPort {
    handle: Option<DisposableContextHandle>,
    operation_calls: Rc<Cell<usize>>,
    observed_operations: Rc<RefCell<Vec<&'static str>>>,
    observed_sessions: Rc<RefCell<Vec<BrowserSessionId>>>,
    observed_incarnations: Rc<RefCell<Vec<BrowserSessionIncarnation>>>,
    fail_operation: Rc<Cell<bool>>,
}

impl DisposableContextPort for OperationPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.handle
            .take()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

impl AuthorizedContextOperationPort for OperationPort {
    type Operation = &'static str;
    type Output = BrowsingContextId;
    type Error = ();

    fn execute_authorized_context_operation(
        &mut self,
        request: &AuthorizedContextOperationRequest<Self::Operation>,
    ) -> Result<Self::Output, Self::Error> {
        self.operation_calls.set(self.operation_calls.get() + 1);
        self.observed_operations
            .borrow_mut()
            .push(*request.operation());
        self.observed_sessions
            .borrow_mut()
            .push(request.browser_session());
        self.observed_incarnations
            .borrow_mut()
            .push(request.incarnation());
        if self.fail_operation.get() {
            Err(())
        } else {
            Ok(request.context().browsing_context())
        }
    }
}

#[test]
fn authorized_operation_uses_exact_bound_port_and_rejects_stale_authority_before_io() {
    let operation_calls = Rc::new(Cell::new(0));
    let observed_operations = Rc::new(RefCell::new(Vec::new()));
    let observed_sessions = Rc::new(RefCell::new(Vec::new()));
    let observed_incarnations = Rc::new(RefCell::new(Vec::new()));
    let fail_operation = Rc::new(Cell::new(false));
    let context = BrowsingContextId::new(503).expect("valid browsing context");
    let port = OperationPort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("operation-user-context-503")
                .expect("valid isolation id"),
            context,
        )),
        operation_calls: Rc::clone(&operation_calls),
        observed_operations: Rc::clone(&observed_operations),
        observed_sessions: Rc::clone(&observed_sessions),
        observed_incarnations: Rc::clone(&observed_incarnations),
        fail_operation: Rc::clone(&fail_operation),
    };
    let session_id = BrowserSessionId::new(503).expect("valid session id");
    let session = BrowserSession::start(session_id).expect("incarnation capacity");
    let incarnation = session.incarnation();
    let mut bound = session.bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&authority, "set-viewport"),
        Ok(context)
    );
    assert_eq!(operation_calls.get(), 1);
    assert_eq!(observed_operations.borrow().as_slice(), &["set-viewport"]);
    assert_eq!(observed_sessions.borrow().as_slice(), &[session_id]);
    assert_eq!(observed_incarnations.borrow().as_slice(), &[incarnation]);

    fail_operation.set(true);
    assert_eq!(
        bound.execute_authorized_context_operation(&authority, "remote-failure"),
        Err(AuthorizedContextOperationError::Adapter(()))
    );
    assert_eq!(operation_calls.get(), 2);
    fail_operation.set(false);

    let current = bound
        .advance_context_epoch(context)
        .expect("advance authority epoch");
    assert_eq!(
        bound.execute_authorized_context_operation(&authority, "stale-operation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch
        ))
    );
    assert_eq!(
        operation_calls.get(),
        2,
        "stale authority must fail before the bound adapter observes an operation"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&current, "reconcile-liveness"),
        Ok(context)
    );
    assert_eq!(operation_calls.get(), 3);
    assert_eq!(
        observed_operations.borrow().as_slice(),
        &["set-viewport", "remote-failure", "reconcile-liveness"]
    );
    assert_eq!(
        observed_sessions.borrow().as_slice(),
        &[session_id, session_id, session_id],
        "the purpose-bounded adapter must observe only the bound Browser Session identity"
    );
    assert_eq!(
        observed_incarnations.borrow().as_slice(),
        &[incarnation, incarnation, incarnation],
        "the purpose-bounded adapter must observe only the bound Browser Session incarnation"
    );
}
