use originweave_evidence::{
    BrowserTaskInterruptionEvidence, BrowserTaskInterruptionKind, ExternalEffectDisposition,
    RetryDisposition,
};

#[test]
fn interruption_before_external_effect_is_retryable_after_complete_cleanup() {
    let evidence = BrowserTaskInterruptionEvidence::new(
        BrowserTaskInterruptionKind::RendererCrash,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );

    assert_eq!(
        evidence.interruption_kind(),
        BrowserTaskInterruptionKind::RendererCrash
    );
    assert_eq!(
        evidence.external_effect_disposition(),
        ExternalEffectDisposition::InterruptedBeforeExternalEffect
    );
    assert!(evidence.browser_context_closed());
    assert!(evidence.resources_reclaimed());
    assert!(evidence.evidence_finalized());
    assert!(evidence.recovery_complete());
    assert_eq!(evidence.retry_disposition(), RetryDisposition::SafeToRetry);
}

#[test]
fn ambiguous_external_effect_requires_quarantine_even_after_cleanup() {
    let evidence = BrowserTaskInterruptionEvidence::new(
        BrowserTaskInterruptionKind::BrowserProcessExit,
        ExternalEffectDisposition::MayHaveCommitted,
        true,
        true,
        true,
    );

    assert_eq!(
        evidence.retry_disposition(),
        RetryDisposition::QuarantineRequired
    );
    assert!(evidence.recovery_complete());
}

#[test]
fn forced_context_close_is_recorded_without_inventing_external_effect() {
    let evidence = BrowserTaskInterruptionEvidence::new(
        BrowserTaskInterruptionKind::ForcedContextClose,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );

    assert_eq!(
        evidence.interruption_kind(),
        BrowserTaskInterruptionKind::ForcedContextClose
    );
    assert_eq!(evidence.retry_disposition(), RetryDisposition::SafeToRetry);
}

#[test]
fn incomplete_cleanup_requires_quarantine_even_before_an_external_effect() {
    for evidence in [
        BrowserTaskInterruptionEvidence::new(
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            false,
            true,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            true,
            false,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            true,
            true,
            false,
        ),
    ] {
        assert!(!evidence.recovery_complete());
        assert_eq!(
            evidence.retry_disposition(),
            RetryDisposition::QuarantineRequired
        );
    }
}
