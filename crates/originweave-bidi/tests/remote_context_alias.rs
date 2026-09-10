#![allow(clippy::expect_used)]

use std::collections::VecDeque;

use originweave_bidi::{
    WebDriverBidiBrowsingContext, WebDriverBidiCreatedContext, WebDriverBidiLifecycleAdapter,
    WebDriverBidiLifecycleBackend,
};
use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, BrowserSessionState,
    DisposableContextCreateError, DisposableContextDestroyError, DisposableIsolationId,
};
use originweave_core::{BrowserSessionId, BrowsingContextId};

#[derive(Debug)]
struct AliasingBackend {
    creates: VecDeque<WebDriverBidiCreatedContext>,
}

impl WebDriverBidiLifecycleBackend for AliasingBackend {
    fn create_disposable_context(
        &mut self,
        _session: BrowserSessionId,
        _incarnation: BrowserSessionIncarnation,
    ) -> Result<WebDriverBidiCreatedContext, DisposableContextCreateError> {
        self.creates
            .pop_front()
            .ok_or(DisposableContextCreateError::CreateFailedClean)
    }

    fn destroy_disposable_context(
        &mut self,
        _session: BrowserSessionId,
        _incarnation: BrowserSessionIncarnation,
        _isolation: &DisposableIsolationId,
        _remote_context: &WebDriverBidiBrowsingContext,
    ) -> Result<(), DisposableContextDestroyError> {
        Ok(())
    }
}

#[test]
fn duplicate_remote_context_is_rejected_before_second_authority_is_minted() {
    let shared_remote = "remote-context-shared";
    let backend = AliasingBackend {
        creates: VecDeque::from([
            WebDriverBidiCreatedContext::new(
                DisposableIsolationId::parse("user-context-a").expect("valid isolation"),
                BrowsingContextId::new(701).expect("valid domain context"),
                WebDriverBidiBrowsingContext::new(shared_remote).expect("valid remote context"),
            ),
            WebDriverBidiCreatedContext::new(
                DisposableIsolationId::parse("user-context-b").expect("valid isolation"),
                BrowsingContextId::new(702).expect("valid domain context"),
                WebDriverBidiBrowsingContext::new(shared_remote).expect("valid remote context"),
            ),
        ]),
    };
    let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
    let mut session = BrowserSession::start(BrowserSessionId::new(70).expect("valid session"))
        .expect("fresh incarnation");

    session
        .create_disposable_context(&mut adapter)
        .expect("first context owns the remote target");
    assert_eq!(
        session.create_disposable_context(&mut adapter),
        Err(BrowserSessionError::ContextCreationUncertain)
    );
    assert_eq!(session.state(), BrowserSessionState::RecoveryRequired);
}
