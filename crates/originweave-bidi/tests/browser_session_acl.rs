#![allow(clippy::expect_used)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use originweave_bidi::{
    WebDriverBidiAclError, WebDriverBidiBrowsingContext, WebDriverBidiCreatedContext,
    WebDriverBidiLifecycleAdapter, WebDriverBidiLifecycleBackend,
    WebDriverBidiPresentationOperation,
};
use originweave_browser_session::{
    BrowserSession, BrowserSessionError, BrowserSessionIncarnation, DisposableContextCreateError,
    DisposableContextDestroyError, DisposableIsolationId, PresentationMutationAuthority,
};
use originweave_core::BrowserSessionId;
use originweave_fingerprint::{DevicePixelRatio, PresentationTimeZone, ViewportBounds};

#[derive(Debug, Clone, PartialEq, Eq)]
struct DestroyTrace {
    session: BrowserSessionId,
    incarnation: BrowserSessionIncarnation,
    isolation: DisposableIsolationId,
    remote_context: String,
}

#[derive(Debug)]
struct FakeBackend {
    creates: VecDeque<WebDriverBidiCreatedContext>,
    destroys: Arc<Mutex<Vec<DestroyTrace>>>,
}

impl FakeBackend {
    fn new(
        contexts: impl IntoIterator<Item = (&'static str, &'static str)>,
        destroys: Arc<Mutex<Vec<DestroyTrace>>>,
    ) -> Self {
        let creates = contexts
            .into_iter()
            .map(|(isolation, remote_context)| {
                WebDriverBidiCreatedContext::new(
                    DisposableIsolationId::parse(isolation).expect("valid user context"),
                    WebDriverBidiBrowsingContext::new(remote_context)
                        .expect("valid remote browsing context"),
                )
            })
            .collect();
        Self { creates, destroys }
    }
}

impl WebDriverBidiLifecycleBackend for FakeBackend {
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
        session: BrowserSessionId,
        incarnation: BrowserSessionIncarnation,
        isolation: &DisposableIsolationId,
        remote_context: &WebDriverBidiBrowsingContext,
    ) -> Result<(), DisposableContextDestroyError> {
        self.destroys
            .lock()
            .expect("trace lock")
            .push(DestroyTrace {
                session,
                incarnation,
                isolation: isolation.clone(),
                remote_context: remote_context.as_str().to_owned(),
            });
        Ok(())
    }
}

fn authorize<'a>(
    adapter: &'a WebDriverBidiLifecycleAdapter<FakeBackend>,
    session: &'a BrowserSession,
    authority: &PresentationMutationAuthority,
) -> Result<originweave_bidi::AuthorizedWebDriverBidiPresentationPlan<'a>, WebDriverBidiAclError> {
    adapter.authorize_standard_presentation(
        session,
        authority,
        ViewportBounds::new(1440, 900).expect("valid viewport"),
        DevicePixelRatio::Quantized2,
        PresentationTimeZone::Utc,
    )
}

#[test]
fn exact_lifecycle_mapping_is_the_only_remote_context_source() {
    let destroys = Arc::new(Mutex::new(Vec::new()));
    let backend = FakeBackend::new(
        [
            ("user-context-a", "remote-context-a"),
            ("user-context-b", "remote-context-b"),
        ],
        destroys,
    );
    let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
    let mut session = BrowserSession::start(BrowserSessionId::new(41).expect("valid session"))
        .expect("fresh incarnation");

    let authority_a = session
        .create_disposable_context(&mut adapter)
        .expect("first owned context");
    let authority_b = session
        .create_disposable_context(&mut adapter)
        .expect("second owned context");

    let plan_a = authorize(&adapter, &session, &authority_a).expect("current authority A");
    let plan_b = authorize(&adapter, &session, &authority_b).expect("current authority B");
    assert_eq!(plan_a.context().as_str(), "remote-context-a");
    assert_eq!(plan_b.context().as_str(), "remote-context-b");

    let [viewport, timezone] = plan_a.apply_actions();
    assert_eq!(viewport.operation(), WebDriverBidiPresentationOperation::SetViewport);
    assert_eq!(viewport.context().as_str(), "remote-context-a");
    assert_eq!(viewport.viewport(), Some(ViewportBounds::new(1440, 900).expect("viewport")));
    assert_eq!(viewport.device_pixel_ratio(), Some(DevicePixelRatio::Quantized2));
    assert_eq!(viewport.timezone(), None);
    assert_eq!(timezone.operation(), WebDriverBidiPresentationOperation::SetTimezone);
    assert_eq!(timezone.context().as_str(), "remote-context-a");
    assert_eq!(timezone.viewport(), None);
    assert_eq!(timezone.device_pixel_ratio(), None);
    assert_eq!(timezone.timezone(), Some(PresentationTimeZone::Utc));

    let [reset_viewport, reset_timezone] = plan_a.cleanup_actions();
    assert_eq!(
        reset_viewport.operation(),
        WebDriverBidiPresentationOperation::ResetViewport
    );
    assert_eq!(reset_viewport.context().as_str(), "remote-context-a");
    assert_eq!(
        reset_timezone.operation(),
        WebDriverBidiPresentationOperation::ResetTimezone
    );
    assert_eq!(reset_timezone.context().as_str(), "remote-context-a");
}

#[test]
fn stale_epoch_is_rejected_before_any_remote_target_can_be_projected() {
    let destroys = Arc::new(Mutex::new(Vec::new()));
    let backend = FakeBackend::new([("user-context-a", "remote-context-a")], destroys);
    let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
    let mut session = BrowserSession::start(BrowserSessionId::new(42).expect("valid session"))
        .expect("fresh incarnation");

    let stale = session
        .create_disposable_context(&mut adapter)
        .expect("owned context");
    let current = session
        .advance_context_epoch(stale.browsing_context())
        .expect("advance authority epoch");

    assert_eq!(
        authorize(&adapter, &session, &stale),
        Err(WebDriverBidiAclError::BrowserSession(
            BrowserSessionError::AuthorityMismatch,
        ))
    );
    assert_eq!(
        authorize(&adapter, &session, &current)
            .expect("current authority")
            .context()
            .as_str(),
        "remote-context-a"
    );
}

#[test]
fn destroyed_or_transport_lost_context_cannot_project_bidi_authority() {
    let destroys = Arc::new(Mutex::new(Vec::new()));
    let backend = FakeBackend::new([("user-context-a", "remote-context-a")], destroys.clone());
    let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
    let mut session = BrowserSession::start(BrowserSessionId::new(43).expect("valid session"))
        .expect("fresh incarnation");

    let authority = session
        .create_disposable_context(&mut adapter)
        .expect("owned context");
    session
        .destroy_disposable_context(&authority, &mut adapter)
        .expect("proven destroy");
    assert_eq!(
        authorize(&adapter, &session, &authority),
        Err(WebDriverBidiAclError::BrowserSession(
            BrowserSessionError::ContextNotOwned,
        ))
    );
    let traces = destroys.lock().expect("trace lock");
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].remote_context, "remote-context-a");
    assert_eq!(traces[0].isolation.as_str(), "user-context-a");
    drop(traces);

    session.end().expect("all contexts destroyed");
    assert_eq!(
        authorize(&adapter, &session, &authority),
        Err(WebDriverBidiAclError::BrowserSession(
            BrowserSessionError::SessionNotActive,
        ))
    );

    let destroys = Arc::new(Mutex::new(Vec::new()));
    let backend = FakeBackend::new([("user-context-b", "remote-context-b")], destroys);
    let mut adapter = WebDriverBidiLifecycleAdapter::new(backend);
    let mut lost_session =
        BrowserSession::start(BrowserSessionId::new(44).expect("valid session"))
            .expect("fresh incarnation");
    let lost_authority = lost_session
        .create_disposable_context(&mut adapter)
        .expect("owned context");
    assert!(lost_session.record_transport_loss());
    assert_eq!(
        authorize(&adapter, &lost_session, &lost_authority),
        Err(WebDriverBidiAclError::BrowserSession(
            BrowserSessionError::SessionNotActive,
        ))
    );
}
