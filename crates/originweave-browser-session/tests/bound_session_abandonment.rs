use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    abandoned_bound_session_count, BrowserSession, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct AbandonmentPort {
    handle: Option<DisposableContextHandle>,
    destroy_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for AbandonmentPort {
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
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        Ok(())
    }
}

fn port_for(context: u64, destroy_calls: &Rc<Cell<usize>>) -> AbandonmentPort {
    AbandonmentPort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse(&format!("abandoned-user-context-{context}"))
                .expect("valid isolation id"),
            BrowsingContextId::new(context).expect("valid browsing context"),
        )),
        destroy_calls: Rc::clone(destroy_calls),
    }
}

#[test]
fn dropping_unresolved_bound_session_is_observable_without_implicit_browser_io() {
    let destroy_calls = Rc::new(Cell::new(0));
    let before = abandoned_bound_session_count();
    let session = BrowserSession::start(BrowserSessionId::new(504).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port_for(504, &destroy_calls));
    let _authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");

    drop(bound);

    assert_eq!(
        destroy_calls.get(),
        0,
        "Drop must never pretend synchronous browser cleanup succeeded"
    );
    assert!(
        abandoned_bound_session_count() > before,
        "unresolved bound-session abandonment must be observable to recovery/operability code"
    );
}

#[test]
fn failed_finish_must_not_be_reclassified_as_abandonment() {
    let destroy_calls = Rc::new(Cell::new(0));
    let before = abandoned_bound_session_count();
    let session = BrowserSession::start(BrowserSessionId::new(506).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port_for(506, &destroy_calls));
    let _authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");

    let finish = bound.finish();

    assert!(
        matches!(
            finish,
            Err(originweave_browser_session::BrowserSessionError::ActiveContextRemains)
        ),
        "finish must reject while remote ownership remains unresolved"
    );
    assert_eq!(
        abandoned_bound_session_count(),
        before,
        "a failed deliberate finish must retain the bound lifecycle owner instead of dropping it as abandonment"
    );
    assert_eq!(
        destroy_calls.get(),
        0,
        "failed finish validation must not perform implicit browser cleanup"
    );
}

#[test]
fn proven_destruction_can_finish_without_abandonment_path() {
    let destroy_calls = Rc::new(Cell::new(0));
    let session = BrowserSession::start(BrowserSessionId::new(505).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port_for(505, &destroy_calls));
    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .destroy_disposable_context(&authority)
        .expect("proven destruction");
    bound.finish().expect("consume normally ended bound session");
    assert_eq!(destroy_calls.get(), 1);
}
