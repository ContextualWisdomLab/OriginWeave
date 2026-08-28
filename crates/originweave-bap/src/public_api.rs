//! Stable internal Browser Agent Protocol lifecycle and crash-recovery contracts.
//!
//! The public recovery types deliberately separate caller-supplied external
//! side-effect classification from task success or authority. Durable runtimes
//! remain responsible for authenticating recovery evidence and for revalidating
//! tenant, policy, destination, secret, and browser authority before any retry.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[path = "lib.rs"]
mod lifecycle;

pub use lifecycle::*;

/// Caller-supplied classification of an external side effect during crash recovery.
///
/// This value is not proof that the classified outcome occurred. A durable
/// runtime or reconciler must authenticate and persist the evidence that
/// supports the classification. Unknown or explicitly unreconciled outcomes
/// fail closed and cannot authorize redispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapExternalSideEffectOutcome {
    /// The recovery authority confirmed that the interrupted command caused no external side effect.
    ConfirmedNoSideEffect,
    /// The recovery authority confirmed that the interrupted command caused its external side effect.
    ConfirmedSideEffect,
    /// The recovery authority cannot determine whether the external side effect occurred.
    UnknownOutcome,
    /// Recovery evidence explicitly requires reconciliation before further action.
    ReconciliationRequired,
}

/// Required fail-closed handling for one classified external recovery outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapRecoveryAction {
    /// Revalidate normal authority and policy before considering command redispatch.
    RevalidateBeforeRedispatch,
    /// Verify the confirmed external side effect and its post-condition without redispatching it.
    VerifyConfirmedSideEffect,
    /// Reconcile external state before any retry, success, or terminal decision.
    ReconcileBeforeFurtherAction,
}

/// Lifecycle-aware fail-closed recovery disposition for one exact command receipt.
///
/// Unlike [`BapRecoveryAction`], this value incorporates the task lifecycle that owns the
/// retained receipt. In particular, a confirmed absence of an external side effect cannot
/// produce a redispatch disposition while the lifecycle is suspended, reconciliation-held,
/// or terminal. The disposition is still not execution authority: durable callers must
/// authenticate the referenced recovery evidence and revalidate all current authority before
/// acting on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapRecoveryDisposition {
    /// Current lifecycle state permits normal authority revalidation before redispatch is considered.
    RevalidateBeforeRedispatch,
    /// Lifecycle state forbids redispatch even though the external outcome would otherwise permit it.
    RedispatchBlockedByLifecycle {
        /// Exact lifecycle state that prevents command redispatch.
        state: BapTaskState,
    },
    /// Verify the confirmed external side effect and its post-condition without redispatching it.
    VerifyConfirmedSideEffect,
    /// Reconcile external state before any retry, success, or terminal decision.
    ReconcileBeforeFurtherAction,
}

impl BapExternalSideEffectOutcome {
    /// Map the classification to the minimum required recovery action.
    ///
    /// This mapping considers only the external side-effect classification. Use
    /// [`BapCommandRecovery::disposition`] when lifecycle state must also be included in the
    /// final fail-closed recovery decision.
    #[must_use]
    pub const fn required_action(self) -> BapRecoveryAction {
        match self {
            Self::ConfirmedNoSideEffect => BapRecoveryAction::RevalidateBeforeRedispatch,
            Self::ConfirmedSideEffect => BapRecoveryAction::VerifyConfirmedSideEffect,
            Self::UnknownOutcome | Self::ReconciliationRequired => {
                BapRecoveryAction::ReconcileBeforeFurtherAction
            }
        }
    }
}

/// Validation failure for one crash-recovery evidence digest identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapRecoveryEvidenceDigestError {
    /// The digest was not canonical lowercase SHA-256 identity evidence.
    InvalidFormat,
}

impl std::fmt::Display for BapRecoveryEvidenceDigestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => formatter.write_str(
                "recovery evidence digest must be sha256: followed by 64 lowercase hexadecimal digits",
            ),
        }
    }
}

impl std::error::Error for BapRecoveryEvidenceDigestError {}

/// Canonical SHA-256 identity for durable crash-recovery evidence.
///
/// The digest identifies the exact evidence object a durable recovery boundary must authenticate
/// before relying on an external side-effect classification. Possession of this identity does not
/// authenticate the evidence, prove the classified outcome, or grant retry, browser, network,
/// secret, approval, or storage authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BapRecoveryEvidenceDigest(String);

impl BapRecoveryEvidenceDigest {
    /// Parse one exact `sha256:` identity with 64 lowercase hexadecimal digits.
    pub fn parse(value: &str) -> Result<Self, BapRecoveryEvidenceDigestError> {
        let Some(hex_digest) = value.strip_prefix("sha256:") else {
            return Err(BapRecoveryEvidenceDigestError::InvalidFormat);
        };
        if hex_digest.len() != 64 {
            return Err(BapRecoveryEvidenceDigestError::InvalidFormat);
        }
        if hex_digest
            .bytes()
            .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(BapRecoveryEvidenceDigestError::InvalidFormat);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the canonical lowercase SHA-256 identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Receipt- and evidence-bound crash-recovery classification for one accepted BAP command.
///
/// Binding the external outcome to both the immutable command receipt and exact recovery-evidence
/// digest prevents a recovery classification from floating free of the retry namespace, task,
/// lifecycle event, accepted transition, or the durable evidence object that supports the outcome.
/// Construction does not authenticate the classification or evidence and grants no authority;
/// callers must validate the evidence at their durable trust boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BapCommandRecovery {
    receipt: BapCommandReceipt,
    external_outcome: BapExternalSideEffectOutcome,
    evidence_digest: BapRecoveryEvidenceDigest,
}

impl BapCommandRecovery {
    /// Bind one external outcome classification and evidence identity to an accepted command receipt.
    #[must_use]
    pub const fn new(
        receipt: BapCommandReceipt,
        external_outcome: BapExternalSideEffectOutcome,
        evidence_digest: BapRecoveryEvidenceDigest,
    ) -> Self {
        Self {
            receipt,
            external_outcome,
            evidence_digest,
        }
    }

    /// Return the immutable command receipt whose interrupted side effect is being classified.
    #[must_use]
    pub const fn receipt(&self) -> &BapCommandReceipt {
        &self.receipt
    }

    /// Return the caller-supplied external side-effect classification.
    #[must_use]
    pub const fn external_outcome(&self) -> BapExternalSideEffectOutcome {
        self.external_outcome
    }

    /// Return the exact recovery-evidence digest bound to this classification.
    #[must_use]
    pub const fn evidence_digest(&self) -> &BapRecoveryEvidenceDigest {
        &self.evidence_digest
    }

    /// Return the minimum fail-closed handling required by the external outcome alone.
    ///
    /// This method deliberately excludes lifecycle state. Use [`Self::disposition`] for the
    /// lifecycle-aware decision that prevents a caller from mistaking a suspended or terminal
    /// task for a redispatch candidate.
    #[must_use]
    pub const fn required_action(&self) -> BapRecoveryAction {
        self.external_outcome.required_action()
    }

    /// Return the lifecycle-aware fail-closed disposition for this exact recovery receipt.
    ///
    /// The retained receipt must first match the lifecycle's exact most recently accepted
    /// transition. Stale, foreign, state-only restored, or divergent lifecycle history therefore
    /// fails closed with the underlying typed receipt error before the external outcome is
    /// interpreted. A confirmed absence of side effect is converted to an explicit lifecycle
    /// block for terminal states, approval/input/checkpoint suspension, and reconciliation holds.
    /// Confirmed side effects still require post-condition verification, while unknown or explicitly
    /// unreconciled outcomes still require reconciliation even when the lifecycle is terminal.
    ///
    /// A returned disposition grants no authority. Durable callers must authenticate the exact
    /// recovery evidence identified by [`Self::evidence_digest`] and independently revalidate
    /// tenant, policy, destination, secret, browser, approval, and other current authority before
    /// any external action.
    pub fn disposition(
        &self,
        lifecycle: &BapTaskLifecycle,
    ) -> Result<BapRecoveryDisposition, BapCommandReceiptError> {
        lifecycle.validate_replay(
            &self.receipt,
            self.receipt.idempotency_key(),
            self.receipt.tenant_id(),
            self.receipt.task_id(),
            self.receipt.event(),
        )?;

        match self.required_action() {
            BapRecoveryAction::RevalidateBeforeRedispatch => {
                let state = lifecycle.state();
                if state.is_terminal()
                    || matches!(
                        state,
                        BapTaskState::WaitingForApproval
                            | BapTaskState::WaitingForExternalInput
                            | BapTaskState::Checkpointed
                            | BapTaskState::ReconciliationRequired
                    )
                {
                    Ok(BapRecoveryDisposition::RedispatchBlockedByLifecycle { state })
                } else {
                    Ok(BapRecoveryDisposition::RevalidateBeforeRedispatch)
                }
            }
            BapRecoveryAction::VerifyConfirmedSideEffect => {
                Ok(BapRecoveryDisposition::VerifyConfirmedSideEffect)
            }
            BapRecoveryAction::ReconcileBeforeFurtherAction => {
                Ok(BapRecoveryDisposition::ReconcileBeforeFurtherAction)
            }
        }
    }

    /// Return whether redispatch may be considered for the current exact lifecycle state.
    ///
    /// This is a convenience projection of [`Self::disposition`], so receipt validation,
    /// lifecycle suspension/terminal handling, and external-outcome handling have one canonical
    /// decision path. Only [`BapRecoveryDisposition::RevalidateBeforeRedispatch`] produces
    /// `Ok(true)`; every other disposition produces `Ok(false)`.
    ///
    /// `Ok(true)` is still not authorization to redispatch. The caller must separately
    /// authenticate the exact recovery evidence identified by [`Self::evidence_digest`] and
    /// revalidate tenant, policy, destination, secret, browser, and any other current authority
    /// before dispatching the command again.
    pub fn permits_redispatch(
        &self,
        lifecycle: &BapTaskLifecycle,
    ) -> Result<bool, BapCommandReceiptError> {
        Ok(matches!(
            self.disposition(lifecycle)?,
            BapRecoveryDisposition::RevalidateBeforeRedispatch
        ))
    }
}
