use originweave_core::{ActionIntentDigest, ActionKind, Origin};
use originweave_evidence::{
    EvidenceSourceKind, PostConditionKind, PostConditionObservation, ProvenanceRecord,
    VerificationResult, VerifiedActionOutcomeError, VerifiedActionOutcomeEvidence,
};

const VALID_INTENT: &str =
    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALID_SOURCE_HASH: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const DISPATCHED_AT_MILLISECONDS: u64 = 1_000;
const OBSERVED_AT_MILLISECONDS: u64 = 1_025;
const MAXIMUM_OBSERVATION_DELAY_MILLISECONDS: u64 = 250;

fn intent() -> Result<ActionIntentDigest, String> {
    ActionIntentDigest::parse(VALID_INTENT).map_err(|error| format!("{error:?}"))
}

fn origin() -> Result<Origin, String> {
    Origin::parse("https://app.example").map_err(|error| format!("{error:?}"))
}

fn provenance(result: VerificationResult) -> Result<ProvenanceRecord, String> {
    ProvenanceRecord::new(
        "https://app.example/receipt",
        "dom:#receipt-status",
        VALID_SOURCE_HASH,
        EvidenceSourceKind::DomTree,
        result,
    )
    .map_err(|error| format!("{error:?}"))
}

fn observation() -> Result<PostConditionObservation, String> {
    PostConditionObservation::new(
        DISPATCHED_AT_MILLISECONDS,
        OBSERVED_AT_MILLISECONDS,
        MAXIMUM_OBSERVATION_DELAY_MILLISECONDS,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn verified_post_condition_can_create_action_success_evidence() -> Result<(), String> {
    let target = origin()?;
    let expected_intent = intent()?;
    let observation = observation()?;
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        target.clone(),
        expected_intent.clone(),
        PostConditionKind::NodeStateChanged,
        observation,
        provenance(VerificationResult::Verified)?,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(evidence.action(), ActionKind::Submit);
    assert_eq!(evidence.target_origin(), &target);
    assert_eq!(evidence.intent_digest(), &expected_intent);
    assert_eq!(
        evidence.post_condition(),
        PostConditionKind::NodeStateChanged
    );
    assert_eq!(evidence.observation(), observation);
    assert_eq!(
        evidence.dispatched_at_milliseconds(),
        DISPATCHED_AT_MILLISECONDS
    );
    assert_eq!(
        evidence.observed_at_milliseconds(),
        OBSERVED_AT_MILLISECONDS
    );
    assert_eq!(
        evidence.maximum_observation_delay_milliseconds(),
        MAXIMUM_OBSERVATION_DELAY_MILLISECONDS
    );
    assert_eq!(
        evidence.provenance().verification_result(),
        VerificationResult::Verified
    );
    Ok(())
}

#[test]
fn unverified_or_rejected_post_condition_cannot_be_recorded_as_success() -> Result<(), String> {
    let observation = observation()?;
    for result in [VerificationResult::Unverified, VerificationResult::Rejected] {
        let outcome = VerifiedActionOutcomeEvidence::new(
            ActionKind::Submit,
            origin()?,
            intent()?,
            PostConditionKind::NodeStateChanged,
            observation,
            provenance(result)?,
        );
        let Err(error) = outcome else {
            return Err("non-verified post-condition admitted success evidence".to_owned());
        };

        assert_eq!(error, VerifiedActionOutcomeError::PostConditionNotVerified);
        assert_eq!(
            error.to_string(),
            "action success requires an independently verified post-condition"
        );
        assert!(std::error::Error::source(&error).is_none());
    }
    Ok(())
}

#[test]
fn post_condition_observation_cannot_predate_action_dispatch() -> Result<(), String> {
    let outcome = PostConditionObservation::new(2_000, 1_999, MAXIMUM_OBSERVATION_DELAY_MILLISECONDS);
    let Err(error) = outcome else {
        return Err("pre-dispatch observation admitted action-success evidence".to_owned());
    };

    assert_eq!(
        error,
        VerifiedActionOutcomeError::PostConditionPredatesDispatch {
            dispatched_at_milliseconds: 2_000,
            observed_at_milliseconds: 1_999,
        }
    );
    assert_eq!(
        error.to_string(),
        "post-condition observation at 1999 ms predates action dispatch at 2000 ms"
    );
    Ok(())
}

#[test]
fn zero_observation_delay_budget_is_rejected() -> Result<(), String> {
    let outcome =
        PostConditionObservation::new(DISPATCHED_AT_MILLISECONDS, OBSERVED_AT_MILLISECONDS, 0);
    let Err(error) = outcome else {
        return Err("zero observation delay budget admitted action-success evidence".to_owned());
    };

    assert_eq!(
        error,
        VerifiedActionOutcomeError::ZeroObservationDelayBudget
    );
    assert_eq!(
        error.to_string(),
        "post-condition observation delay budget must be greater than zero"
    );
    Ok(())
}

#[test]
fn post_condition_observation_cannot_outlive_delay_budget() -> Result<(), String> {
    let outcome = PostConditionObservation::new(5_000, 5_251, MAXIMUM_OBSERVATION_DELAY_MILLISECONDS);
    let Err(error) = outcome else {
        return Err("stale observation admitted action-success evidence".to_owned());
    };

    assert_eq!(
        error,
        VerifiedActionOutcomeError::PostConditionObservationExpired {
            elapsed_milliseconds: 251,
            maximum_observation_delay_milliseconds: MAXIMUM_OBSERVATION_DELAY_MILLISECONDS,
        }
    );
    assert_eq!(
        error.to_string(),
        "post-condition observation delay 251 ms exceeds the configured maximum of 250 ms"
    );
    Ok(())
}

#[test]
fn same_monotonic_tick_is_allowed_for_coarse_clock_sources() -> Result<(), String> {
    let observation = PostConditionObservation::new(
        4_000,
        4_000,
        MAXIMUM_OBSERVATION_DELAY_MILLISECONDS,
    )
    .map_err(|error| error.to_string())?;
    let evidence = VerifiedActionOutcomeEvidence::new(
        ActionKind::Submit,
        origin()?,
        intent()?,
        PostConditionKind::NetworkMutationObserved,
        observation,
        provenance(VerificationResult::Verified)?,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(evidence.dispatched_at_milliseconds(), 4_000);
    assert_eq!(evidence.observed_at_milliseconds(), 4_000);
    Ok(())
}

#[test]
fn post_condition_kinds_cover_first_browser_vertical_slice_evidence() -> Result<(), String> {
    let observation = observation()?;
    for kind in [
        PostConditionKind::UrlChanged,
        PostConditionKind::NodeStateChanged,
        PostConditionKind::DialogStateChanged,
        PostConditionKind::NetworkMutationObserved,
    ] {
        let evidence = VerifiedActionOutcomeEvidence::new(
            ActionKind::Submit,
            origin()?,
            intent()?,
            kind,
            observation,
            provenance(VerificationResult::Verified)?,
        )
        .map_err(|error| error.to_string())?;

        assert_eq!(evidence.post_condition(), kind);
    }
    Ok(())
}
