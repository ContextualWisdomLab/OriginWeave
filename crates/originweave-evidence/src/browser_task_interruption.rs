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
/// The value records caller-supplied facts only. It does not detect browser crashes,
/// prove cleanup, reconcile external effects, or dispatch a retry by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserTaskInterruptionEvidence {
    interruption_kind: BrowserTaskInterruptionKind,
    external_effect_disposition: ExternalEffectDisposition,
    browser_context_closed: bool,
    resources_reclaimed: bool,
    evidence_finalized: bool,
}

impl BrowserTaskInterruptionEvidence {
    /// Create one interruption evidence value from trusted runtime observations.
    #[must_use]
    pub const fn new(
        interruption_kind: BrowserTaskInterruptionKind,
        external_effect_disposition: ExternalEffectDisposition,
        browser_context_closed: bool,
        resources_reclaimed: bool,
        evidence_finalized: bool,
    ) -> Self {
        Self {
            interruption_kind,
            external_effect_disposition,
            browser_context_closed,
            resources_reclaimed,
            evidence_finalized,
        }
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

    /// Derive whether a retry is safe from external-effect and cleanup evidence.
    #[must_use]
    pub const fn retry_disposition(&self) -> RetryDisposition {
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
