use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId, NavigationTerminationOutcome,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct DownloadWitnessConsumptionProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for DownloadWitnessConsumptionProbePort {
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

impl AuthorizedContextOperationPort for DownloadWitnessConsumptionProbePort {
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
fn download_start_consumes_navigation_witness_against_late_terminal_replay() {
    let context = BrowsingContextId::new(1221).expect("valid browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = DownloadWitnessConsumptionProbePort {
        handle: Some(DisposableContextHandle::new(
            DisposableIsolationId::parse("download-witness-consumption-context")
                .expect("valid isolation id"),
            context,
        )),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1221).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_after_create = adapter_calls.get();
    let pending = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("navigation start issues a pending witness");
    let recovery_before_download = bound.browser_session().recovery_evidence().to_vec();

    bound
        .record_observed_navigation_download_started(&pending)
        .expect("matching download start closes this exact navigation liveness boundary");
    assert_eq!(
        bound.execute_authorized_context_operation(&initial, "stale-after-download-start"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "download start must keep the retained pre-navigation authority revoked"
    );
    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "late positive terminal evidence must not reuse a witness already consumed by download start"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Aborted),
        Err(BrowserSessionError::AuthorityMismatch),
        "late abort evidence must not reuse a witness already consumed by download start"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed),
        Err(BrowserSessionError::AuthorityMismatch),
        "late failure evidence must not reuse a witness already consumed by download start"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_download.as_slice(),
        "late terminal replay before re-establishment must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "download closure, stale authority rejection, and terminal replay rejection must be zero-I/O"
    );

    let current = bound
        .reestablish_presentation_authority(context)
        .expect("download-start liveness closure preserves exactly one re-establishment opportunity");
    assert_eq!(
        current.context_epoch().value(),
        initial.context_epoch().value() + 1,
        "rejected terminal replay must not spend the presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "download-derived re-establishment opportunity remains single-use"
    );

    let calls_before_post_reestablishment_replay = adapter_calls.get();
    let recovery_before_post_reestablishment_replay =
        bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.record_observed_navigation_settled(&pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "consumed download witness must remain stale after fresh authority is minted"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Aborted),
        Err(BrowserSessionError::AuthorityMismatch),
        "late abort must not invalidate authority minted after download liveness closure"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(&pending, NavigationTerminationOutcome::Failed),
        Err(BrowserSessionError::AuthorityMismatch),
        "late failure must not invalidate authority minted after download liveness closure"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_post_reestablishment_replay.as_slice(),
        "post-re-establishment terminal replay must not mutate recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_post_reestablishment_replay,
        "post-re-establishment terminal replay must fail before adapter I/O"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&current, "usable-after-late-terminal-replay"),
        Ok(context),
        "late terminal replay for the consumed witness must not revoke the fresh authority"
    );
}
