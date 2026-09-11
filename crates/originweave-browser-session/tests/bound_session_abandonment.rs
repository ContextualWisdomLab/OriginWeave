use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;

use originweave_browser_session::{
    abandoned_bound_session_count, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

static ABANDONMENT_COUNTER_LOCK: Mutex<()> = Mutex::new(());

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
    let _guard = ABANDONMENT_COUNTER_LOCK
        .lock()
        .expect("abandonment counter test lock");
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
fn failed_finish_retains_same_bound_owner_for_cleanup_and_retry() {
    let _guard = ABANDONMENT_COUNTER_LOCK
        .lock()
        .expect("abandonment counter test lock");
    let destroy_calls = Rc::new(Cell::new(0));
    let before = abandoned_bound_session_count();
    let session = BrowserSession::start(BrowserSessionId::new(506).expect("valid session id"))
        .expect("incarnation capacity");
    let mut bound = session.bind_lifecycle_port(port_for(506, &destroy_calls));
    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");

    assert_eq!(bound.finish(), Err(BrowserSessionError::ActiveContextRemains));
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

    bound
        .destroy_disposable_context(&authority)
        .expect("the same bound lifecycle owner must remain available for cleanup");
    assert_eq!(destroy_calls.get(), 1);
    bound.finish().expect("retry succeeds after proven destruction");
    drop(bound);
    assert_eq!(
        abandoned_bound_session_count(),
        before,
        "successful retry must leave no abandonment signal"
    );
}

#[test]
fn proven_destruction_can_finish_without_abandonment_path() {
    let _guard = ABANDONMENT_COUNTER_LOCK
        .lock()
        .expect("abandonment counter test lock");
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
    bound.finish().expect("end normally after proven destruction");
    assert_eq!(destroy_calls.get(), 1);
}
