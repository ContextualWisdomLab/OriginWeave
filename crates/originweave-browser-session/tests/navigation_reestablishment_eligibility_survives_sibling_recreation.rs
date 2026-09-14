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

struct SiblingRecreationEligibilityProbePort {
    handles: VecDeque<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
}

impl DisposableContextPort for SiblingRecreationEligibilityProbePort {
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

impl AuthorizedContextOperationPort for SiblingRecreationEligibilityProbePort {
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

#[derive(Clone, Copy)]
enum NavigationClosure {
    Settled,
    Aborted,
    Failed,
    DownloadStarted,
}

/// Build one adapter-issued disposable handle with an explicit isolation identity.
fn handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

/// Exercise one closure family through sibling destroy/recreate ABA and stale replay.
fn assert_eligibility_survives_sibling_recreation(closure: NavigationClosure, session_id: u64) {
    let first_context =
        BrowsingContextId::new(session_id * 10 + 1).expect("valid first browsing context");
    let second_context =
        BrowsingContextId::new(session_id * 10 + 2).expect("valid second browsing context");
    let adapter_calls = Rc::new(Cell::new(0));
    let port = SiblingRecreationEligibilityProbePort {
        handles: VecDeque::from([
            handle(first_context, "eligibility-survivor-across-sibling-recreation"),
            handle(second_context, "sibling-old-ownership-before-recreation"),
            handle(second_context, "sibling-new-ownership-after-recreation"),
        ]),
        adapter_calls: Rc::clone(&adapter_calls),
    };
    let mut bound = BrowserSession::start(
        BrowserSessionId::new(session_id).expect("valid browser session id"),
    )
    .expect("incarnation capacity")
    .bind_lifecycle_port(port);

    let first_authority = bound
        .create_disposable_context()
        .expect("first disposable context accepted");
    let second_old_authority = bound
        .create_disposable_context()
        .expect("old sibling ownership accepted");
    let first_pending = bound
        .record_observed_navigation(
            first_authority.incarnation(),
            first_context,
            first_authority.context_epoch(),
        )
        .expect("first context enters navigation-pending state");

    match closure {
        NavigationClosure::Settled => bound
            .record_observed_navigation_settled(&first_pending)
            .expect("positive closure creates one first-context re-establishment opportunity"),
        NavigationClosure::Aborted => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Aborted,
            )
            .expect("aborted closure creates one first-context re-establishment opportunity"),
        NavigationClosure::Failed => bound
            .record_observed_navigation_terminated(
                &first_pending,
                NavigationTerminationOutcome::Failed,
            )
            .expect("failed closure creates one first-context re-establishment opportunity"),
        NavigationClosure::DownloadStarted => bound
            .record_observed_navigation_download_started(&first_pending)
            .expect("download start creates one first-context re-establishment opportunity"),
    }

    let second_old_pending = bound
        .record_observed_navigation(
            second_old_authority.incarnation(),
            second_context,
            second_old_authority.context_epoch(),
        )
        .expect("old sibling ownership independently enters navigation-pending state");
    bound
        .record_observed_navigation_committed(&second_old_pending)
        .expect("old sibling ownership records non-terminal commit progress");

    let state_before_recreation = bound.browser_session().state();
    let recovery_before_recreation = bound.browser_session().recovery_evidence().to_vec();
    let calls_before_destroy = adapter_calls.get();
    bound
        .destroy_owned_disposable_context(second_context)
        .expect("proven sibling destruction consumes only its old ownership generation");
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "proven sibling destruction performs exactly one lifecycle adapter call",
    );
    assert_eq!(bound.browser_session().state(), state_before_recreation);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_recreation.as_slice(),
        "proven sibling destruction must not rewrite lifecycle recovery evidence",
    );

    let second_new_authority = bound
        .create_disposable_context()
        .expect("the same raw sibling context id may be accepted as a fresh ownership generation");
    assert_eq!(
        second_new_authority.context(),
        second_old_authority.context(),
        "the hostile case intentionally reuses the same raw sibling context id",
    );
    assert!(
        second_new_authority.context_epoch().value() > second_old_authority.context_epoch().value(),
        "recreated sibling ownership must carry a newer aggregate-issued epoch",
    );
    assert_eq!(bound.browser_session().state(), state_before_recreation);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_recreation.as_slice(),
        "sibling recreation must not rewrite lifecycle recovery evidence",
    );
    let calls_after_recreate = adapter_calls.get();

    let first_reestablished = bound
        .reestablish_presentation_authority(first_context)
        .expect("A eligibility must survive B recreation without a compensating later event");
    assert_eq!(
        first_reestablished.context_epoch().value(),
        second_new_authority.context_epoch().value() + 1,
        "re-establishment must continue the aggregate-wide epoch sequence after sibling recreation",
    );
    assert_eq!(bound.browser_session().state(), state_before_recreation);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_recreation.as_slice(),
        "first-context re-establishment must not rewrite lifecycle recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_recreate,
        "re-establishment remains an in-memory authority transition",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(
            &first_authority,
            "first-retained-authority-after-reestablishment",
        ),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        )),
        "re-establishment must not reactivate the first context's retained pre-navigation authority",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(first_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "the preserved first-context eligibility remains single-use",
    );
    assert_eq!(
        bound.reestablish_presentation_authority(second_context),
        Err(BrowserSessionError::AuthorityMismatch),
        "a recreated sibling with no navigation closure has no re-establishment eligibility",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_recreate,
        "authority and eligibility rejection after re-establishment must remain zero-I/O",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&second_new_authority, "second-new-current"),
        Ok(second_context),
        "A re-establishment must not revoke B-new fresh authority before stale predecessor evidence is observed",
    );
    let calls_after_second_new_operation = adapter_calls.get();
    assert_eq!(calls_after_second_new_operation, calls_after_recreate + 1);

    assert_eq!(
        bound.record_observed_navigation_committed(&second_old_pending),
        Err(BrowserSessionError::AuthorityMismatch),
        "commit evidence from the destroyed sibling generation must remain stale after raw-id recreation",
    );
    assert_eq!(bound.browser_session().state(), state_before_recreation);
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_before_recreation.as_slice(),
        "stale old-generation commit replay must not rewrite recovery evidence",
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_second_new_operation,
        "stale old-generation commit replay must fail before adapter I/O",
    );
    let second_current_after_stale = bound
        .presentation_authority(second_context)
        .expect("stale B-old evidence must not revoke B-new current authority");
    assert_eq!(
        second_current_after_stale, second_new_authority,
        "stale B-old evidence must preserve the complete B-new authority identity, including isolation, incarnation, context, and epoch",
    );
    assert_eq!(
        bound.execute_authorized_context_operation(
            &second_new_authority,
            "second-new-current-after-stale-replay",
        ),
        Ok(second_context),
        "stale B-old evidence must not make B-new's unchanged authority non-executable",
    );

    assert_eq!(
        bound.execute_authorized_context_operation(&first_reestablished, "first-current"),
        Ok(first_context),
        "stale predecessor evidence must not revoke A's re-established authority",
    );
    assert_eq!(adapter_calls.get(), calls_after_second_new_operation + 2);
}

/// Complete-positive navigation eligibility survives sibling raw-id recreation.
#[test]
fn positive_eligibility_survives_sibling_raw_id_recreation() {
    assert_eligibility_survives_sibling_recreation(NavigationClosure::Settled, 1381);
}

/// Aborted-navigation eligibility survives sibling raw-id recreation.
#[test]
fn aborted_eligibility_survives_sibling_raw_id_recreation() {
    assert_eligibility_survives_sibling_recreation(NavigationClosure::Aborted, 1382);
}

/// Failed-navigation eligibility survives sibling raw-id recreation.
#[test]
fn failed_eligibility_survives_sibling_raw_id_recreation() {
    assert_eligibility_survives_sibling_recreation(NavigationClosure::Failed, 1383);
}

/// Download-start eligibility survives sibling raw-id recreation.
#[test]
fn download_eligibility_survives_sibling_raw_id_recreation() {
    assert_eligibility_survives_sibling_recreation(NavigationClosure::DownloadStarted, 1384);
}
