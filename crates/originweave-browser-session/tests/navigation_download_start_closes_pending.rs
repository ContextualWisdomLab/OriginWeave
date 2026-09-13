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

struct DownloadNavigationProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DownloadNavigationProbePort {
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

impl AuthorizedContextOperationPort for DownloadNavigationProbePort {
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
fn matching_download_start_closes_pending_navigation_without_reviving_old_authority() {
    let context = BrowsingContextId::new(1201).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DownloadNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("download-navigation-user-context")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1201).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");

    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-while-navigation-pending"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "navigation start must revoke retained presentation authority before any download outcome"
    );
    assert_eq!(adapter_calls.get(), calls_after_create);

    bound
        .record_observed_navigation_download_started(&pending)
        .expect("matching download start closes the pending navigation liveness boundary");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "download-start observation is evidence only and must not perform browser I/O"
    );

    let after_download_start = bound
        .reestablish_presentation_authority(context)
        .expect("qualified download start permits one explicit fresh presentation authority");
    assert_eq!(
        after_download_start.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "download start itself must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "download-start-derived re-establishment opportunity must be single-use"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "duplicate re-establishment rejection must remain zero-I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &after_download_start,
            "usable-after-qualified-download-start",
        ),
        Ok(context),
        "only the explicitly re-established authority is usable after download start"
    );
}

#[test]
fn superseded_download_start_cannot_close_the_current_pending_navigation() {
    let context = BrowsingContextId::new(1202).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DownloadNavigationProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("superseded-download-navigation-user-context")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1202).expect("valid session id"))
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
        .expect("later qualified navigation supersedes the earlier pending witness");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation state transitions must remain zero-I/O"
    );

    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "a delayed download start for the superseded navigation must not close the current pending navigation"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "rejecting stale download evidence must leave the newer navigation pending"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "stale download evidence and premature re-establishment rejection must perform no browser I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the current pending witness may close on download start");
    let current = bound
        .reestablish_presentation_authority(context)
        .expect("current download start permits one explicit fresh authority");
    assert_eq!(
        current.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "rejected stale download evidence must not spend a presentation epoch"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&current, "usable-after-current-download-start"),
        Ok(context)
    );
}
