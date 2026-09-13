use std::cell::Cell;
use std::collections::VecDeque;
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

struct CrossContextDownloadProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for CrossContextDownloadProbePort {
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

impl AuthorizedContextOperationPort for CrossContextDownloadProbePort {
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
fn sibling_navigation_preserves_download_reestablishment_eligibility() {
    let first_context = BrowsingContextId::new(1233).expect("valid first context");
    let second_context = BrowsingContextId::new(1234).expect("valid second context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = CrossContextDownloadProbePort {
        handles: VecDeque::from([
            handle(first_context, "cross-context-download-first"),
            handle(second_context, "cross-context-download-second"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(BrowserSessionId::new(1233).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_authority = bound
        .create_disposable_context()
        .expect("second disposable context accepted");
    let calls_after_create = adapter_calls.get();

    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first context enters navigation-pending state");
    bound
        .record_observed_navigation_download_started(&first_pending)
        .expect("first context download start creates one re-establishment opportunity");
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "navigation and download-start observations must remain zero-I/O"
    );

    let state_after_first_download = bound.browser_session().state();
    let recovery_after_first_download = bound.browser_session().recovery_evidence().to_vec();
    let second_pending = bound
        .record_observed_navigation(
            second_authority.incarnation(),
            second_context,
            second_authority.context_epoch(),
        )
        .expect("sibling navigation may start while first download eligibility remains unused");
    assert_eq!(bound.browser_session().state(), state_after_first_download);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_first_download.as_slice(),
        "sibling navigation start must not manufacture recovery evidence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "sibling navigation admission must remain zero-I/O"
    );
    let state_after_second_start = bound.browser_session().state();
    let recovery_after_second_start = bound.browser_session().recovery_evidence().to_vec();

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("sibling navigation must not erase first context download-derived eligibility");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_authority.context_epoch().value() + 1,
        "sibling navigation must not spend an aggregate presentation epoch"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the sibling remains pending until its own closing evidence arrives"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-stale-while-own-navigation-pending",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "first-context re-establishment must not revive the sibling retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "re-establishment gating and stale sibling authority rejection must fail before adapter I/O"
    );

    assert_eq!(
        bound.record_observed_navigation_download_started(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "the first download witness remains consumed after sibling navigation starts"
    );
    assert_eq!(
        bound.record_observed_navigation_settled(&first_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "late complete-positive evidence for consumed A must not settle sibling B"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Aborted,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "late aborted evidence for consumed A must not terminate sibling B"
    );
    assert_eq!(
        bound.record_observed_navigation_terminated(
            &first_pending,
            NavigationTerminationOutcome::Failed,
        ),
        Err(BrowserSessionError::AuthorityMismatch),
        "late failed evidence for consumed A must not terminate sibling B"
    );
    assert_eq!(
        bound.browser_session().state(),
        state_after_second_start,
        "consumed A replay must not change aggregate lifecycle state while B is pending"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_after_second_start.as_slice(),
        "consumed A replay must not mutate recovery evidence"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "consumed A replay must not create re-establishment eligibility for B"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_authority,
            "second-still-stale-after-first-replay",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "consumed A replay must not reactivate B retained authority"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "every consumed A replay and B authority check must fail before adapter I/O"
    );

    bound
        .record_observed_navigation_download_started(&second_pending)
        .expect("only the sibling's own current witness may close its navigation liveness boundary");
    let second_reestablished = bound
        .reestablish_presentation_authority(second_context)
        .expect("sibling may re-establish only after its own qualified download start");
    assert_eq!(
        second_reestablished.context_epoch().value(),
        first_reestablished.context_epoch().value() + 1,
        "independent sibling re-establishment must continue the aggregate-wide epoch sequence"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_create,
        "both download liveness closures and re-establishments must remain zero-I/O"
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "first-still-usable"),
        Ok(first_context),
        "sibling navigation closure must not revoke the first context's current authority"
    );
    assert_eq!(
        bound.execute_authorized_context_operation(&second_reestablished, "second-now-usable"),
        Ok(second_context),
        "the sibling authority becomes usable only after its own explicit re-establishment"
    );
}
