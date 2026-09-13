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

struct DownloadSupersessionProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DownloadSupersessionProbePort {
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

impl AuthorizedContextOperationPort for DownloadSupersessionProbePort {
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
fn newer_navigation_supersedes_unspent_download_reestablishment_eligibility() {
    let context = BrowsingContextId::new(1231).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DownloadSupersessionProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("download-supersession-context")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1231).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let first_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation start issues a pending witness");

    bound
        .record_observed_navigation_download_started(&first_pending)
        .expect("matching download start creates one explicit re-establishment opportunity");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "download-start liveness closure must remain zero-I/O"
    );

    let recovery_before_second_start = bound.browser_session().recovery_evidence().to_vec();
    let second_pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("a later qualified navigation supersedes the unspent download eligibility");
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_second_start.as_slice(),
        "superseding a download-derived opportunity must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "starting the newer navigation must not perform browser I/O"
    );

    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the older download-start opportunity must not mint authority while the newer navigation is pending"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &initial,
            "stale-while-newer-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "the retained pre-navigation authority must remain revoked"
    );
    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the consumed first witness must remain stale after the newer navigation starts"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "premature re-establishment and stale download replay must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the current pending witness may create a fresh re-establishment opportunity");
    let current = bound
        .reestablish_presentation_authority(context)
        .expect("current download-start outcome permits exactly one explicit fresh authority");
    assert_eq!(
        current.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "superseding the older opportunity must not spend a presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the current download-derived opportunity must remain single-use"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&current, "usable-after-current-download"),
        Ok(context),
        "only the authority minted from the current navigation may be used"
    );
}