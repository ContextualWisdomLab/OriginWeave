use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateDisposition, DisposableContextCreateError, DisposableContextCreateRequest,
    DisposableContextDestroyError, DisposableContextDestroyRequest, DisposableContextHandle,
    DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug, Default)]
struct CreationLedger {
    session: Option<BrowserSessionId>,
    incarnation: Option<BrowserSessionIncarnation>,
    pending: BTreeMap<u64, DisposableContextHandle>,
    accepted: Vec<u64>,
    rejected: Vec<u64>,
}

#[derive(Debug)]
struct TransactionalPort {
    handles: VecDeque<DisposableContextHandle>,
    ledger: Rc<RefCell<CreationLedger>>,
}

impl TransactionalPort {
    fn new(handles: Vec<DisposableContextHandle>, ledger: Rc<RefCell<CreationLedger>>) -> Self {
        Self {
            handles: handles.into(),
            ledger,
        }
    }
}

impl DisposableContextPort for TransactionalPort {
    fn create_disposable_context(
        &mut self,
        request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        let handle = self
            .handles
            .pop_front()
            .expect("fixture supplies one handle per create");
        let mut ledger = self.ledger.borrow_mut();
        ledger.session.get_or_insert(request.browser_session());
        ledger.incarnation.get_or_insert(request.incarnation());
        let prior = ledger
            .pending
            .insert(request.attempt_epoch().value(), handle.clone());
        assert!(prior.is_none(), "create attempts must not collide");
        Ok(handle)
    }

    fn complete_disposable_context_creation(
        &mut self,
        completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        let mut ledger = self.ledger.borrow_mut();
        if ledger.session != Some(completion.browser_session())
            || ledger.incarnation != Some(completion.incarnation())
        {
            return Err(DisposableContextCreateCompletionError::CompletionFailed);
        }
        let attempt = completion.attempt_epoch().value();
        if ledger.pending.remove(&attempt).is_none() {
            return Err(DisposableContextCreateCompletionError::CompletionFailed);
        }
        match completion.disposition() {
            DisposableContextCreateDisposition::Accepted => ledger.accepted.push(attempt),
            DisposableContextCreateDisposition::Rejected => ledger.rejected.push(attempt),
        }
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

#[test]
fn accepted_and_rejected_create_candidates_are_correlated_by_exact_attempt() {
    let context = BrowsingContextId::new(8010).expect("valid browsing context");
    let first = DisposableContextHandle::new(
        DisposableIsolationId::parse("transaction-user-context-a").expect("valid isolation"),
        context,
    );
    let duplicate_context = DisposableContextHandle::new(
        DisposableIsolationId::parse("transaction-user-context-b").expect("valid isolation"),
        context,
    );
    let ledger = Rc::new(RefCell::new(CreationLedger::default()));
    let port = TransactionalPort::new(vec![first, duplicate_context], Rc::clone(&ledger));
    let session = BrowserSession::start(BrowserSessionId::new(801).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port);

    let accepted = bound
        .create_disposable_context()
        .expect("first candidate accepted");
    assert_eq!(accepted.context_epoch().value(), 1);
    assert_eq!(
        bound.create_disposable_context(),
        Err(BrowserSessionError::DuplicateBrowsingContext)
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );

    let ledger = ledger.borrow();
    assert!(ledger.pending.is_empty());
    assert_eq!(ledger.accepted, vec![1]);
    assert_eq!(ledger.rejected, vec![2]);
}
