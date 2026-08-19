#![allow(clippy::expect_used)]

use originweave_core::{ActionIntentDigest, BrowserSessionId, BrowsingContextId, DocumentEpoch};
use originweave_evidence::{
    BrowserTaskInterruptionEvidence, BrowserTaskInterruptionKind, ExternalEffectDisposition,
    RetryDisposition,
};

const ACTION_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const OTHER_ACTION_INTENT: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn browser_authority() -> Result<(BrowserSessionId, BrowsingContextId, DocumentEpoch), String> {
    Ok((
        BrowserSessionId::new(7).map_err(|error| error.to_string())?,
        BrowsingContextId::new(11).map_err(|error| error.to_string())?,
        DocumentEpoch::new(3).map_err(|error| error.to_string())?,
    ))
}

fn action_intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(ACTION_INTENT).expect("valid action-intent digest")
}

fn other_action_intent() -> ActionIntentDigest {
    ActionIntentDigest::parse(OTHER_ACTION_INTENT).expect("valid alternate action-intent digest")
}

#[test]
fn interruption_before_external_effect_is_retryable_after_complete_cleanup() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let (session_id, context_id, document_epoch) = browser_authority;
    let intent = action_intent();
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        intent.clone(),
        BrowserTaskInterruptionKind::RendererCrash,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );

    assert_eq!(evidence.browser_session_id(), session_id);
    assert_eq!(evidence.browsing_context_id(), context_id);
    assert_eq!(evidence.document_epoch(), document_epoch);
    assert_eq!(evidence.action_intent_digest(), &intent);
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
    assert_eq!(
        evidence.retry_disposition(browser_authority, &intent),
        RetryDisposition::SafeToRetry
    );
    Ok(())
}

#[test]
fn recovery_evidence_for_another_action_intent_cannot_authorize_retry() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let intent = action_intent();
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        intent,
        BrowserTaskInterruptionKind::RendererCrash,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );

    assert_eq!(
        evidence.retry_disposition(browser_authority, &other_action_intent()),
        RetryDisposition::QuarantineRequired
    );
    Ok(())
}

#[test]
fn recovery_evidence_for_another_browser_authority_cannot_authorize_retry() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let (session_id, context_id, document_epoch) = browser_authority;
    let intent = action_intent();
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        intent.clone(),
        BrowserTaskInterruptionKind::RendererCrash,
        ExternalEffectDisposition::InterruptedBeforeExternalEffect,
        true,
        true,
        true,
    );
    let other_session = BrowserSessionId::new(session_id.value() + 1).map_err(|error| error.to_string())?;
    let other_context =
        BrowsingContextId::new(context_id.value() + 1).map_err(|error| error.to_string())?;
    let other_epoch =
        DocumentEpoch::new(document_epoch.value() + 1).map_err(|error| error.to_string())?;

    for current_authority in [
        (other_session, context_id, document_epoch),
        (session_id, other_context, document_epoch),
        (session_id, context_id, other_epoch),
    ] {
        assert_eq!(
            evidence.retry_disposition(current_authority, &intent),
            RetryDisposition::QuarantineRequired
        );
    }
    Ok(())
}

#[test]
fn ambiguous_external_effect_requires_quarantine_even_after_cleanup() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let intent = action_intent();
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        intent.clone(),
        BrowserTaskInterruptionKind::BrowserProcessExit,
        ExternalEffectDisposition::MayHaveCommitted,
        true,
        true,
        true,
    );

    assert_eq!(
        evidence.retry_disposition(browser_authority, &intent),
        RetryDisposition::QuarantineRequired
    );
    assert!(evidence.recovery_complete());
    Ok(())
}

#[test]
fn forced_context_close_is_recorded_without_inventing_external_effect() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let intent = action_intent();
    let evidence = BrowserTaskInterruptionEvidence::new(
        browser_authority,
        intent.clone(),
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
    assert_eq!(
        evidence.retry_disposition(browser_authority, &intent),
        RetryDisposition::SafeToRetry
    );
    Ok(())
}

#[test]
fn incomplete_cleanup_requires_quarantine_even_before_an_external_effect() -> Result<(), String> {
    let browser_authority = browser_authority()?;
    let intent = action_intent();
    for evidence in [
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
            intent.clone(),
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            false,
            true,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
            intent.clone(),
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            true,
            false,
            true,
        ),
        BrowserTaskInterruptionEvidence::new(
            browser_authority,
            intent.clone(),
            BrowserTaskInterruptionKind::RendererCrash,
            ExternalEffectDisposition::InterruptedBeforeExternalEffect,
            true,
            true,
            false,
        ),
    ] {
        assert!(!evidence.recovery_complete());
        assert_eq!(
            evidence.retry_disposition(browser_authority, &intent),
            RetryDisposition::QuarantineRequired
        );
    }
    Ok(())
}
