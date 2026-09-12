use std::cell::{Cell, RefCell};
use std::rc::Rc;

use originweave_browser_session::{
    AuthorizedContextOperationError, AuthorizedContextOperationPort,
    AuthorizedContextOperationRequest, BrowserSession, BrowserSessionError, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct CleanupProbePort {
    handle: Option<DisposableContextHandle>,
    adapter_calls: Rc<Cell<usize>>,
    destroyed_handles: Rc<RefCell<Vec<DisposableContextHandle>>>,
    fail_destroy: bool,
}

impl DisposableContextPort for CleanupProbePort {
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
        request: &DisposableContextDestroyRequest,
    ) -> Result<(), DisposableContextDestroyError> {
        self.adapter_calls.set(self.adapter_calls.get() + 1);
        self.destroyed_handles
            .borrow_mut()
            .push(request.context().clone());
        if self.fail_destroy {
            Err(DisposableContextDestroyError::DestroyFailed)
        } else {
            Ok(())
        }
    }
}

impl AuthorizedContextOperationPort for CleanupProbePort {
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

fn bound_session(
    session: u64,
    context: BrowsingContextId,
    isolation: &str,
    adapter_calls: &Rc<Cell<usize>>,
    destroyed_handles: &Rc<RefCell<Vec<DisposableContextHandle>>>,
    fail_destroy: bool,
) -> originweave_browser_session::BoundBrowserSession<CleanupProbePort> {
    let handle = DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    );
    let port = CleanupProbePort {
        handle: Some(handle),
        adapter_calls: Rc::clone(adapter_calls),
        destroyed_handles: Rc::clone(destroyed_handles),
        fail_destroy,
    };
    BrowserSession::start(BrowserSessionId::new(session).expect("valid session id"))
        .expect("incarnation capacity")
        .bind_lifecycle_port(port)
}

fn expected_handle(context: BrowsingContextId, isolation: &str) -> DisposableContextHandle {
    DisposableContextHandle::new(
        DisposableIsolationId::parse(isolation).expect("valid isolation id"),
        context,
    )
}

#[test]
fn navigation_invalidation_does_not_strand_owned_disposable_cleanup() {
    let context = BrowsingContextId::new(921).expect("valid browsing context");
    let isolation = "navigation-cleanup-user-context-921";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        921,
        context,
        isolation,
        &adapter_calls,
        &destroyed_handles,
        false,
    );

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "pre-navigation"),
        Ok(context)
    );

    bound
        .record_observed_navigation(
            pre_navigation.incarnation(),
            context,
            pre_navigation.context_epoch(),
        )
        .expect("observed navigation invalidates presentation authority");
    let calls_after_navigation = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-presentation"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        ))
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_navigation,
        "stale presentation authority must fail before adapter I/O"
    );

    bound
        .destroy_owned_disposable_context(context)
        .expect(
            "the exact bound lifecycle owner must destroy its isolation without presentation re-establishment",
        );
    assert_eq!(
        adapter_calls.get(),
        calls_after_navigation + 1,
        "lifecycle destruction must call the bound port exactly once"
    );
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(context, isolation)],
        "destruction must use the exact browser-issued handle retained at creation"
    );

    let calls_after_destroy = adapter_calls.get();
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::ContextNotOwned),
        "proven destruction must consume lifecycle ownership and cannot reopen presentation authority"
    );
    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextNotOwned),
        "proven destruction must not permit a second remote cleanup attempt from the same raw selector"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_destroy,
        "post-destroy re-establishment and duplicate cleanup must fail before adapter I/O"
    );

    bound
        .end()
        .expect("proven destruction leaves no owned disposable context behind");
}

#[test]
fn later_navigation_after_reestablishment_still_allows_lifecycle_cleanup_without_reissuing_presentation_authority() {
    let context = BrowsingContextId::new(931).expect("valid browsing context");
    let isolation = "navigation-cleanup-user-context-931";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        931,
        context,
        isolation,
        &adapter_calls,
        &destroyed_handles,
        false,
    );

    let initial = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let first_settlement_authority = bound
        .record_observed_navigation(initial.incarnation(), context, initial.context_epoch())
        .expect("first navigation invalidates initial presentation authority");
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::AuthorityMismatch),
        "navigation start alone must not reissue presentation authority"
    );
    bound
        .record_observed_navigation_settled(&first_settlement_authority)
        .expect("matching browser settlement presents the aggregate-issued witness");
    let reestablished = bound
        .reestablish_presentation_authority(context)
        .expect("owner explicitly establishes authority for the settled next document");
    assert_eq!(
        reestablished.context_epoch().value(),
        initial.context_epoch().value() + 1
    );
    bound
        .record_observed_navigation(
            reestablished.incarnation(),
            context,
            reestablished.context_epoch(),
        )
        .expect("later navigation invalidates the re-established presentation authority");

    let calls_before_destroy = adapter_calls.get();
    assert_eq!(
        bound.execute_authorized_context_operation(&reestablished, "stale-second-document"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        ))
    );
    assert_eq!(adapter_calls.get(), calls_before_destroy);

    bound
        .destroy_owned_disposable_context(context)
        .expect("lifecycle ownership survives presentation invalidation across later navigation");
    assert_eq!(adapter_calls.get(), calls_before_destroy + 1);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(context, isolation)]
    );
    bound.end().expect("destroyed session can end normally");
}

#[test]
fn raw_foreign_context_cannot_select_cleanup_outside_bound_lifecycle_ownership_after_navigation_invalidation() {
    let owned = BrowsingContextId::new(941).expect("valid owned context");
    let foreign = BrowsingContextId::new(942).expect("valid foreign context");
    let isolation = "navigation-cleanup-user-context-941";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        941,
        owned,
        isolation,
        &adapter_calls,
        &destroyed_handles,
        false,
    );

    let authority = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    let calls_before_navigation = adapter_calls.get();
    bound
        .record_observed_navigation(authority.incarnation(), owned, authority.context_epoch())
        .expect("owned navigation invalidates presentation authority before cleanup selection");
    assert_eq!(
        adapter_calls.get(),
        calls_before_navigation,
        "navigation invalidation must not perform adapter I/O"
    );

    let calls_before_foreign_destroy = adapter_calls.get();
    assert_eq!(
        bound.destroy_owned_disposable_context(foreign),
        Err(BrowserSessionError::ContextNotOwned),
        "navigation invalidation must not let a raw foreign browser identifier manufacture cleanup authority"
    );
    assert_eq!(adapter_calls.get(), calls_before_foreign_destroy);
    assert!(destroyed_handles.borrow().is_empty());

    bound
        .destroy_owned_disposable_context(owned)
        .expect(
            "the same bound owner can destroy its exact retained handle while presentation remains invalidated",
        );
    assert_eq!(adapter_calls.get(), calls_before_foreign_destroy + 1);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[expected_handle(owned, isolation)]
    );
}

#[test]
fn failed_cleanup_after_navigation_enters_recovery_without_reopening_presentation() {
    let context = BrowsingContextId::new(951).expect("valid browsing context");
    let isolation = "navigation-cleanup-user-context-951";
    let adapter_calls = Rc::new(Cell::new(0));
    let destroyed_handles = Rc::new(RefCell::new(Vec::new()));
    let mut bound = bound_session(
        951,
        context,
        isolation,
        &adapter_calls,
        &destroyed_handles,
        true,
    );

    let pre_navigation = bound
        .create_disposable_context()
        .expect("accepted disposable context");
    bound
        .record_observed_navigation(
            pre_navigation.incarnation(),
            context,
            pre_navigation.context_epoch(),
        )
        .expect(
            "navigation invalidates presentation authority without changing lifecycle ownership",
        );

    let calls_before_destroy = adapter_calls.get();
    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::ContextDestructionFailed),
        "unproven remote destruction must fail closed"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_before_destroy + 1,
        "one cleanup attempt may reach the exact bound lifecycle port"
    );
    let exact_handle = expected_handle(context, isolation);
    assert_eq!(
        destroyed_handles.borrow().as_slice(),
        &[exact_handle],
        "failed cleanup must still target the exact retained browser-issued handle"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "unproven destruction must enter recovery instead of consuming ownership"
    );
    assert!(
        !bound.browser_session().recovery_evidence().is_empty(),
        "unproven destruction must leave recovery evidence; exact epoch-bearing evidence shape remains the upstream #317 acceptance"
    );

    let calls_after_failure = adapter_calls.get();
    let recovery_evidence_after_failure = bound.browser_session().recovery_evidence().to_vec();
    assert_eq!(
        bound.execute_authorized_context_operation(&pre_navigation, "stale-after-failed-cleanup"),
        Err(AuthorizedContextOperationError::BrowserSession(
            BrowserSessionError::SessionNotActive,
        )),
        "inactive aggregate trust must take precedence over stale presentation-generation inspection"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "stale authority rejection must not rewrite the recovery-required aggregate state"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "stale authority rejection must leave the original unproven-destruction evidence unchanged"
    );
    assert_eq!(
        bound.reestablish_presentation_authority(context),
        Err(BrowserSessionError::SessionNotActive),
        "cleanup uncertainty must preserve lifecycle ownership while closing the normal presentation-authority surface"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "presentation re-establishment must not consume or rewrite unresolved lifecycle ownership"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "presentation re-establishment must leave the original unproven-destruction evidence unchanged"
    );
    assert_eq!(
        bound.destroy_owned_disposable_context(context),
        Err(BrowserSessionError::SessionNotActive),
        "ordinary cleanup retry must remain closed while exact lifecycle ownership is preserved for reviewed recovery"
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired,
        "ordinary cleanup retry must not consume unresolved lifecycle ownership"
    );
    assert_eq!(
        bound.browser_session().recovery_evidence(),
        recovery_evidence_after_failure.as_slice(),
        "ordinary cleanup retry must leave the original recovery evidence unchanged"
    );
    assert_eq!(
        adapter_calls.get(),
        calls_after_failure,
        "retained stale authority, re-authorize, and normal retry must all fail before any additional adapter I/O"
    );
}
