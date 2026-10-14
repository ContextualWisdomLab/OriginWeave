use originweave_core::{ActionIntentDigest, BrowserSessionId, BrowsingContextId, DocumentEpoch};

/// Browser/runtime interruption category recorded for one Agent Task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrowserTaskInterruptionKind {
    /// The renderer process serving the task crashed or became unavailable.
    RendererCrash,
    /// The browser process exited while the task was active.
    BrowserProcessExit,
    /// The task's browser context was forcibly closed by a trusted runtime boundary.
    ForcedContextClose,
}

/// What is known about externally visible effects when the interruption occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalEffectDisposition {
    /// The trusted runtime established that interruption occurred before any external effect.
    InterruptedBeforeExternalEffect,
    /// An externally visible effect may already have committed and must be reconciled.
    MayHaveCommitted,
}

/// Whether the interrupted task may be retried without first reconciling ambiguous state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetryDisposition {
    /// Cleanup is complete and no external effect could have committed.
    SafeToRetry,
    /// Retry is unsafe until recovery or external-state reconciliation completes.
    QuarantineRequired,
}

/// Credential-free recovery evidence for an interrupted browser task.
///
/// The value binds caller-supplied interruption and cleanup facts to one exact OriginWeave
/// browser session, browsing context, document epoch, and immutable canonical action-intent digest.
/// It does not authenticate the browser adapter, prove that the identified browser authority or
/// action experienced the interruption, detect crashes, prove cleanup, reconcile external effects,
/// or dispatch a retry by itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTaskInterruptionEvidence {
    browser_session_id: BrowserSessionId,
    browsing_context_id: BrowsingContextId,
    document_epoch: DocumentEpoch,
    action_intent_digest: ActionIntentDigest,
    interruption_kind: BrowserTaskInterruptionKind,
    external_effect_disposition: ExternalEffectDisposition,
    browser_context_closed: bool,
    resources_reclaimed: bool,
    evidence_finalized: bool,
}

impl BrowserTaskInterruptionEvidence {
    /// Create one interruption evidence value bound to exact browser and action authority.
    ///
    /// `browser_authority` is the exact `(browser session, browsing context, document epoch)`
    /// tuple observed by the trusted runtime. `action_intent_digest` must be the complete canonical
    /// intent digest of the interrupted governed action. Callers must not infer either input from
    /// ambient browser state.
    #[must_use]
    pub fn new(
        browser_authority: (BrowserSessionId, BrowsingContextId, DocumentEpoch),
        action_intent_digest: ActionIntentDigest,
        interruption_kind: BrowserTaskInterruptionKind,
        external_effect_disposition: ExternalEffectDisposition,
        browser_context_closed: bool,
        resources_reclaimed: bool,
        evidence_finalized: bool,
    ) -> Self {
        let (browser_session_id, browsing_context_id, document_epoch) = browser_authority;
        Self {
            browser_session_id,
            browsing_context_id,
            document_epoch,
            action_intent_digest,
            interruption_kind,
            external_effect_disposition,
            browser_context_closed,
            resources_reclaimed,
            evidence_finalized,
        }
    }

    /// Return the exact OriginWeave browser session bound to the interruption.
    #[must_use]
    pub const fn browser_session_id(&self) -> BrowserSessionId {
        self.browser_session_id
    }

    /// Return the exact OriginWeave browsing context bound to the interruption.
    #[must_use]
    pub const fn browsing_context_id(&self) -> BrowsingContextId {
        self.browsing_context_id
    }

    /// Return the exact OriginWeave document epoch bound to the interruption.
    #[must_use]
    pub const fn document_epoch(&self) -> DocumentEpoch {
        self.document_epoch
    }

    /// Return the immutable canonical action-intent digest bound to this recovery evidence.
    #[must_use]
    pub const fn action_intent_digest(&self) -> &ActionIntentDigest {
        &self.action_intent_digest
    }

    /// Return the recorded interruption category.
    #[must_use]
    pub const fn interruption_kind(&self) -> BrowserTaskInterruptionKind {
        self.interruption_kind
    }

    /// Return the recorded external-effect disposition.
    #[must_use]
    pub const fn external_effect_disposition(&self) -> ExternalEffectDisposition {
        self.external_effect_disposition
    }

    /// Return whether the task browser context is confirmed closed.
    #[must_use]
    pub const fn browser_context_closed(&self) -> bool {
        self.browser_context_closed
    }

    /// Return whether task-owned runtime resources are confirmed reclaimed.
    #[must_use]
    pub const fn resources_reclaimed(&self) -> bool {
        self.resources_reclaimed
    }

    /// Return whether interruption evidence is confirmed finalized.
    #[must_use]
    pub const fn evidence_finalized(&self) -> bool {
        self.evidence_finalized
    }

    /// Return true only when context closure, resource reclamation, and evidence finalization all completed.
    #[must_use]
    pub const fn recovery_complete(&self) -> bool {
        self.browser_context_closed && self.resources_reclaimed && self.evidence_finalized
    }

    /// Derive whether the exact expected browser authority and action may be safely retried.
    ///
    /// `expected_browser_authority` must be the trusted runtime's current exact `(browser session,
    /// browsing context, document epoch)` tuple. Recovery evidence for any other browser authority
    /// or canonical action intent always requires quarantine, even when cleanup is otherwise
    /// complete. This prevents recovery facts from one session/context/document/action from
    /// authorizing replay in another authority scope.
    #[must_use]
    pub fn retry_disposition(
        &self,
        expected_browser_authority: (BrowserSessionId, BrowsingContextId, DocumentEpoch),
        expected_action_intent: &ActionIntentDigest,
    ) -> RetryDisposition {
        let (browser_session_id, browsing_context_id, document_epoch) = expected_browser_authority;
        if self.browser_session_id != browser_session_id
            || self.browsing_context_id != browsing_context_id
            || self.document_epoch != document_epoch
            || &self.action_intent_digest != expected_action_intent
        {
            return RetryDisposition::QuarantineRequired;
        }
        if matches!(
            self.external_effect_disposition,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect
        ) && self.recovery_complete()
        {
            RetryDisposition::SafeToRetry
        } else {
            RetryDisposition::QuarantineRequired
        }
    }
}
