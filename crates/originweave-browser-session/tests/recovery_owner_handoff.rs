use std::cell::Cell;
use std::rc::Rc;

use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionRecoveryEvidence, BrowserSessionState,
    DisposableContextCreateCompletion, DisposableContextCreateCompletionError,
    DisposableContextCreateError, DisposableContextCreateRequest, DisposableContextDestroyError,
    DisposableContextDestroyRequest, DisposableContextHandle, DisposableContextPort,
    DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

struct RecoveryTrackedPort {
    handle: DisposableContextHandle,
    fail_destroy: bool,
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
    drop_calls: Rc<Cell<usize>>,
}

impl Drop for RecoveryTrackedPort {
    fn drop(&mut self) {
        self.drop_calls.set(self.drop_calls.get() + 1);
    }
}

impl DisposableContextPort for RecoveryTrackedPort {
    fn create_disposable_context(
        &mut self,
        _request: &DisposableContextCreateRequest,
    ) -> Result<DisposableContextHandle, DisposableContextCreateError> {
        self.create_calls.set(self.create_calls.get() + 1);
        Ok(self.handle.clone())
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
        if self.fail_destroy {
            Err(DisposableContextDestroyError::DestroyFailed)
        } else {
            Ok(())
        }
    }
}

fn recovery_port(
    context: u64,
    isolation: &str,
    fail_destroy: bool,
    create_calls: Rc<Cell<usize>>,
    destroy_calls: Rc<Cell<usize>>,
    drop_calls: Rc<Cell<usize>>,
) -> Result<RecoveryTrackedPort, &'static str> {
    let isolation = DisposableIsolationId::parse(isolation)
        .map_err(|_| "static fixture isolation id must be valid")?;
    let browsing_context = BrowsingContextId::new(context)
        .map_err(|_| "static fixture browsing context id must be valid")?;
    Ok(RecoveryTrackedPort {
        handle: DisposableContextHandle::new(isolation, browsing_context),
        fail_destroy,
        create_calls,
        destroy_calls,
        drop_calls,
    })
}

#[test]
fn unproven_destroy_hands_exact_bound_adapter_and_evidence_to_recovery_owner(
) -> Result<(), &'static str> {
    let create_calls = Rc::new(Cell::new(0));
    let destroy_calls = Rc::new(Cell::new(0));
    let drop_calls = Rc::new(Cell::new(0));
    let session = BrowserSession::start(
        BrowserSessionId::new(710).map_err(|_| "static session id must be valid")?,
    )
    .map_err(|_| "browser session incarnation must be available")?;
    let port = recovery_port(
        7_100,
        "recovery-handoff-destroy",
        true,
        Rc::clone(&create_calls),
        Rc::clone(&destroy_calls),
        Rc::clone(&drop_calls),
    )?;
    let mut bound = session.bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture context creation must succeed")?;
    assert_eq!(
        bound.destroy_disposable_context(&authority),
        Err(BrowserSessionError::ContextDestructionFailed)
    );
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::RecoveryRequired
    );
    let expected_evidence = bound.browser_session().recovery_evidence().to_vec();
    assert!(matches!(
        expected_evidence.as_slice(),
        [BrowserSessionRecoveryEvidence::UnprovenDestruction { .. }]
    ));
    assert_eq!(create_calls.get(), 1);
    assert_eq!(destroy_calls.get(), 1);
    assert_eq!(drop_calls.get(), 0);

    let recovery = bound
        .into_recovery()
        .map_err(|_| "RecoveryRequired must permit consuming recovery handoff")?;

    assert_eq!(
        drop_calls.get(),
        0,
        "handoff must move, not replace, the bound adapter"
    );
    assert_eq!(recovery.state(), BrowserSessionState::RecoveryRequired);
    assert_eq!(recovery.recovery_evidence(), expected_evidence);
    assert!(recovery.create_attempt_recovery_evidence().is_empty());
    assert_eq!(create_calls.get(), 1, "handoff must not create browser state");
    assert_eq!(destroy_calls.get(), 1, "handoff must not imply cleanup I/O");

    drop(recovery);
    assert_eq!(
        drop_calls.get(),
        1,
        "the exact non-Clone adapter must stay alive until the recovery owner is dropped"
    );
    Ok(())
}

#[test]
fn transport_loss_hands_exact_bound_adapter_and_evidence_to_recovery_owner(
) -> Result<(), &'static str> {
    let create_calls = Rc::new(Cell::new(0));
    let destroy_calls = Rc::new(Cell::new(0));
    let drop_calls = Rc::new(Cell::new(0));
    let session = BrowserSession::start(
        BrowserSessionId::new(711).map_err(|_| "static session id must be valid")?,
    )
    .map_err(|_| "browser session incarnation must be available")?;
    let port = recovery_port(
        7_110,
        "recovery-handoff-transport",
        false,
        Rc::clone(&create_calls),
        Rc::clone(&destroy_calls),
        Rc::clone(&drop_calls),
    )?;
    let mut bound = session.bind_lifecycle_port(port);

    let authority = bound
        .create_disposable_context()
        .map_err(|_| "fixture context creation must succeed")?;
    assert_eq!(authority.browsing_context().value(), 7_110);
    assert!(bound.record_transport_loss());
    assert_eq!(
        bound.browser_session().state(),
        BrowserSessionState::TransportLost
    );
    let expected_evidence = bound.browser_session().recovery_evidence().to_vec();
    assert!(matches!(
        expected_evidence.as_slice(),
        [BrowserSessionRecoveryEvidence::TransportLossOwnedHandle(_)]
    ));
    assert_eq!(create_calls.get(), 1);
    assert_eq!(destroy_calls.get(), 0);
    assert_eq!(drop_calls.get(), 0);

    let recovery = bound
        .into_recovery()
        .map_err(|_| "TransportLost must permit consuming recovery handoff")?;

    assert_eq!(
        drop_calls.get(),
        0,
        "handoff must preserve the same bound adapter instance"
    );
    assert_eq!(recovery.state(), BrowserSessionState::TransportLost);
    assert_eq!(recovery.recovery_evidence(), expected_evidence);
    assert!(recovery.create_attempt_recovery_evidence().is_empty());
    assert_eq!(create_calls.get(), 1, "handoff must not create browser state");
    assert_eq!(destroy_calls.get(), 0, "transport loss is not destruction proof");

    drop(recovery);
    assert_eq!(
        drop_calls.get(),
        1,
        "the exact non-Clone adapter must remain owned by the recovery wrapper until drop"
    );
    Ok(())
}
