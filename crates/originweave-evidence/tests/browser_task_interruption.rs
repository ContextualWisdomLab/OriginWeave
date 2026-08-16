use originweave_core::{BrowserSessionId, BrowsingContextId, DocumentEpoch};
use originweave_evidence::{
    BrowserTaskInterruptionEvidence, BrowserTaskInterruptionKind, ExternalEffectDisposition,
    RetryDisposition,
};

fn browser_authority() -> Result<(BrowserSessionId, BrowsingContextId, DocumentEpoch), String> {
    Ok((
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
    ))
}

#[test]
fn interruption_before_external_effect_is_retryable_after_complete_cleanup() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let (session_id, context_id, document_epoch) = browser_authority;
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        BrowserTaskInterruptionKind::RendererCrash,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );

    assert_eq!(evidence.browser_session_id(), session_id);
    assert_eq!(evidence.browsing_context_id(), context_id);
    assert_eq!(evidence.document_epoch(), document_epoch);
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
    Ok(())
}

#[test]
fn ambiguous_external_effect_requires_quarantine_even_after_cleanup() -> Result<(), String> {
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority()?,
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
    Ok(())
}

#[test]
fn forced_context_close_is_recorded_without_inventing_external_effect() -> Result<(), String> {
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority()?,
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
    Ok(())
}

#[test]
fn incomplete_cleanup_requires_quarantine_even_before_an_external_effect() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    for evidence in [
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            false,
            true,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            true,
            false,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
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
    Ok(())
}
