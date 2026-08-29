use std::error::Error as _;

use originweave_core::{
    benchmark_failure::BenchmarkFailureClass,
    release_acceptance::{
        BenchmarkSuite, BenchmarkSuiteEvidence, DeclaredLimitation, ReleaseDecision,
        ReleaseDecisionError, ZeroEventSafetyEvidence, ZeroEventSafetyMetric,
        ZeroEventSafetyObservation,
    },
    zero_event_safety_gate::{
        CommercialReleaseAcceptanceError, ZeroEventSafetyGateDecision, ZeroEventSafetyGateError,
        ZeroEventSafetyRequirement, decide_commercial_release_with_zero_event_safety,
    },
    zero_event_threshold::ZeroEventSafetyThreshold,
};

fn passed_benchmarks() -> Vec<BenchmarkSuiteEvidence> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(BenchmarkSuiteEvidence::Passed)
        .collect()
}

fn observation(trial_count: u64) -> Result<ZeroEventSafetyObservation, ReleaseDecisionError> {
    Ok(ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::UnauthorizedAction,
        ZeroEventSafetyEvidence::new(trial_count, 9_500)?,
    ))
}

fn requirement() -> Result<ZeroEventSafetyRequirement, Box<dyn std::error::Error>> {
    Ok(ZeroEventSafetyRequirement::new(
        ZeroEventSafetyMetric::UnauthorizedAction,
        ZeroEventSafetyThreshold::new(10_000, 9_500)?,
    ))
}

#[test]
fn quantitative_safety_threshold_miss_blocks_release_acceptance()
-> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(100)?;
    let requirement = requirement()?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(
        report.benchmark_report().decision(),
        ReleaseDecision::Accepted
    );
    assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
    assert_eq!(
        report.zero_event_safety_gate_report().decision(),
        ZeroEventSafetyGateDecision::Inconclusive
    );
    Ok(())
}

#[test]
fn satisfied_safety_gate_preserves_full_acceptance() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let requirement = requirement()?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert_eq!(
        report.zero_event_safety_gate_report().decision(),
        ZeroEventSafetyGateDecision::Satisfied
    );
    Ok(())
}

#[test]
fn satisfied_safety_gate_preserves_declared_limitations()
-> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let requirement = requirement()?;
    let limitation = DeclaredLimitation::new(
        "DRM playback",
        "Excluded from the declared commercial support profile",
    )?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[limitation],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    Ok(())
}

#[test]
fn known_benchmark_failure_remains_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let requirement = requirement()?;
    let mut evidence = passed_benchmarks();
    evidence[0] = BenchmarkSuiteEvidence::Failure {
        suite: BenchmarkSuite::ControlledDeterministic,
        classification: BenchmarkFailureClass::DeterministicContractFailure,
    };

    let report = decide_commercial_release_with_zero_event_safety(
        evidence,
        &[],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Rejected);
    Ok(())
}

#[test]
fn incomplete_benchmark_evidence_remains_inconclusive()
-> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let requirement = requirement()?;
    let evidence = passed_benchmarks().into_iter().take(4);

    let report = decide_commercial_release_with_zero_event_safety(
        evidence,
        &[],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
    Ok(())
}

#[test]
fn invalid_benchmark_evidence_preserves_typed_source() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let requirement = requirement()?;
    let duplicate = BenchmarkSuite::ControlledDeterministic;
    let error = decide_commercial_release_with_zero_event_safety(
        [
            BenchmarkSuiteEvidence::Passed(duplicate),
            BenchmarkSuiteEvidence::Passed(duplicate),
        ],
        &[],
        &[observation],
        &[requirement],
    )
    .expect_err("duplicate benchmark evidence must fail closed");

    assert_eq!(
        error,
        CommercialReleaseAcceptanceError::ReleaseEvidence(ReleaseDecisionError::DuplicateSuite(
            duplicate
        ))
    );
    assert!(error.to_string().starts_with("invalid release evidence:"));
    assert!(error.source().is_some());
    Ok(())
}

#[test]
fn invalid_safety_policy_preserves_typed_source() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observation(1_000)?;
    let error = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &[observation],
        &[],
    )
    .expect_err("missing safety requirements must fail closed");

    assert_eq!(
        error,
        CommercialReleaseAcceptanceError::ZeroEventSafetyGate(
            ZeroEventSafetyGateError::MissingRequirements
        )
    );
    assert!(
        error
            .to_string()
            .starts_with("invalid zero-event safety gate:")
    );
    assert!(error.source().is_some());
    Ok(())
}
