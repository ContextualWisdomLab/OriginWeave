use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, DisposableContextCreateCompletion,
    DisposableContextCreateCompletionError, DisposableContextCreateError,
    DisposableContextCreateRequest, DisposableContextDestroyError, DisposableContextDestroyRequest,
    DisposableContextHandle, DisposableContextPort, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct ReestablishmentProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for ReestablishmentProbePort {
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

#[test]
fn explicit_reestablishment_is_single_use_for_each_terminal_navigation_transition() {
    let context = BrowsingContextId::new(989).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = ReestablishmentProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("navigation-single-use-user-context-989")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(989).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_navigation = adapter_calls.get();

    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("current navigation start invalidates presentation authority");
    bound
        .record_observed_navigation_settled(&pending)
        .expect("matching terminal evidence settles the current navigation");
    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("one explicit re-establishment is permitted for the terminal transition");
    assert_eq!(
        reestablished.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "one settled navigation consumes exactly one fresh presentation epoch"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "navigation settlement and re-establishment are zero-I/O Browser Session transitions"
    );

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the same terminal transition must not mint presentation authority twice"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "duplicate re-establishment must fail before adapter I/O"
    );

    let still_current = bound
        .presentation_authority(context)
        .expect("rejected duplicate re-establishment must leave current authority intact");
    assert_eq!(still_current.incarnation(), reestablished.incarnation());
    assert_eq!(still_current.context_epoch(), reestablished.context_epoch());

    let later_pending = bound
        .record_observed_navigation(
            still_current.incarnation(),
            context,
            still_current.context_epoch(),
        )
        .expect("a later real navigation may invalidate the current generation");
    bound
        .record_observed_navigation_settled(&later_pending)
        .expect("later matching terminal evidence settles only the later navigation");
    let second_reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("the later terminal transition permits one new re-establishment");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        reestablished.context_epoch().value() + 1,
        "a rejected duplicate re-establishment must not consume an aggregate epoch"
    );
    assert_eq!(adapter_calls.get(), calls_before_navigation);
}
