use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionState, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ObservedPort {
    create_calls: Rc<Cell<usize>>,
    completion_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ObservedPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls.set(self.create_calls.get() + 1);
        Ok(DisposableContextHandle::new(
            DisposableIsolationId::parse("transport-user-context-501").expect("valid isolation id"),
            BrowsingContextId::new(501).expect("valid browsing context"),
        ))
    }

    fn complete_disposable_context_creation(
        &mut self,
        _completion: &DisposableContextCreateCompletion,
    ) -> Result<(), DisposableContextCreateCompletionError> {
        self.completion_calls.set(self.completion_calls.get() + 1);
        Ok(())
    }

    fn destroy_disposable_context(
        &mut self,
        _request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Ok(())
    }
}

#[test]
fn transport_loss_preserves_exact_owned_handle_as_non_authorizing_recovery_evidence() {
    let create_calls = Rc::new(Cell::new(0));
    let completion_calls = Rc::new(Cell::new(0));
    let destroy_calls = Rc::new(Cell::new(0));
    let port = ObservedPort {
        create_calls: Rc::clone(&create_calls),
        completion_calls: Rc::clone(&completion_calls),
        destroy_calls: Rc::clone(&destroy_calls),
    };
    let session = BrowserSession::start(BrowserSessionId::new(501).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(authority.browsing_context().value(), 501);
    assert_eq!(create_calls.get(), 1);
    assert_eq!(completion_calls.get(), 1);
    assert_eq!(destroy_calls.get(), 0);

    assert!(bound.record_transport_loss());
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost
    );
    assert_eq!(
        create_calls.get(),
        1,
        "transport loss must not create browser state"
    );
    assert_eq!(
        completion_calls.get(),
        1,
        "transport loss must not settle another create attempt"
    );
    assert_eq!(
        destroy_calls.get(),
        0,
        "transport loss is not destruction proof"
    );

    let evidence = bound.browser_session().recovery_evidence();
    assert_eq!(
        evidence.len(),
        1,
        "the exact previously owned handle must remain externally recoverable after transport loss"
    );
    let rendered = format!("{:?}", evidence[0]);
    assert!(
        rendered.contains("transport-user-context-501"),
        "recovery evidence lost the exact disposable isolation identity: {rendered}"
    );
    assert!(
        rendered.contains("501"),
        "recovery evidence lost the exact browsing-context identity: {rendered}"
    );

    assert!(!bound.record_transport_loss());
    assert_eq!(
        bound.browser_session().recovery_evidence().len(),
        1,
        "repeated transport-loss reports must not duplicate recovery evidence"
    );
    assert_eq!(
        bound.presentation_authority(authority.browsing_context()),
        Err(originweave_browser_session::BrowserSessionError::SessionNotActive),
        "transport-loss recovery evidence must never resurrect presentation authority"
    );
    assert_eq!(destroy_calls.get(), 0);
}
