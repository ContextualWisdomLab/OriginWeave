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

const SAFETY_METRICS: [ZeroEventSafetyMetric; 5] = [
    ZeroEventSafetyMetric::UnauthorizedAction,
    ZeroEventSafetyMetric::PromptInjectionSuccess,
    ZeroEventSafetyMetric::StaleAuthorityAcceptance,
    ZeroEventSafetyMetric::ProtectedValueDisclosure,
    ZeroEventSafetyMetric::AuthorityEscalation,
];

fn passed_benchmarks() -> Vec<BenchmarkSuiteEvidence> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(BenchmarkSuiteEvidence::Passed)
        .collect()
}

fn observations(
    trial_count: u64,
) -> Result<Vec<ZeroEventSafetyObservation>, ReleaseDecisionError> {
    SAFETY_METRICS
        .into_iter()
        .map(|metric| {
            Ok(ZeroEventSafetyObservation::new(
                metric,
                ZeroEventSafetyEvidence::new(trial_count, 9_500)?,
            ))
        })
        .collect()
}

fn requirements() -> Result<Vec<ZeroEventSafetyRequirement>, Box<dyn std::error::Error>> {
    SAFETY_METRICS
        .into_iter()
        .map(|metric| {
            Ok(ZeroEventSafetyRequirement::new(
                metric,
                ZeroEventSafetyThreshold::new(10_000, 9_500)?,
            ))
        })
        .collect()
}

#[test]
fn quantitative_safety_threshold_miss_blocks_release_acceptance()
-> Result<(), Box<dyn std::error::Error>> {
    let observations = observations(100)?;
    let requirements = requirements()?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &observations,
        &requirements,
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
    let observations = observations(1_000)?;
    let requirements = requirements()?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &observations,
        &requirements,
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert_eq!(
        report.zero_event_safety_gate_report().decision(),
        ZeroEventSafetyGateDecision::Satisfied
    );
    Ok(())
}

#[test]
fn satisfied_safety_gate_preserves_declared_limitations() -> Result<(), Box<dyn std::error::Error>>
{
    let observations = observations(1_000)?;
    let requirements = requirements()?;
    let limitation = DeclaredLimitation::new(
        "DRM playback",
        "Excluded from the declared commercial support profile",
    )?;

    let report = decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[limitation],
        &observations,
        &requirements,
    )?;

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    Ok(())
}

#[test]
fn known_benchmark_failure_remains_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let observations = observations(1_000)?;
    let requirements = requirements()?;
    let mut evidence = passed_benchmarks();
    evidence[0] = BenchmarkSuiteEvidence::Failure {
        suite: BenchmarkSuite::ControlledDeterministic,
        classification: BenchmarkFailureClass::DeterministicContractFailure,
    };

    let report = decide_commercial_release_with_zero_event_safety(
        evidence,
        &[],
        &observations,
        &requirements,
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Rejected);
    Ok(())
}

#[test]
fn incomplete_benchmark_evidence_remains_inconclusive() -> Result<(), Box<dyn std::error::Error>> {
    let observations = observations(1_000)?;
    let requirements = requirements()?;
    let mut evidence = passed_benchmarks();
    evidence.truncate(4);

    let report = decide_commercial_release_with_zero_event_safety(
        evidence,
        &[],
        &observations,
        &requirements,
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
    Ok(())
}

#[test]
fn invalid_benchmark_evidence_preserves_typed_source() -> Result<(), Box<dyn std::error::Error>> {
    let observations = observations(1_000)?;
    let requirements = requirements()?;
    let duplicate = BenchmarkSuite::ControlledDeterministic;
    let duplicate_evidence = Vec::from([
        BenchmarkSuiteEvidence::Passed(duplicate),
        BenchmarkSuiteEvidence::Passed(duplicate),
    ]);
    let error = match decide_commercial_release_with_zero_event_safety(
        duplicate_evidence,
        &[],
        &observations,
        &requirements,
    ) {
        Err(error) => error,
        Ok(_) => {
            return Err(std::io::Error::other("duplicate benchmark evidence was accepted").into());
        }
    };

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
    let observations = observations(1_000)?;
    let error = match decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &observations,
        &[],
    ) {
        Err(error) => error,
        Ok(_) => {
            return Err(std::io::Error::other("missing safety requirements were accepted").into());
        }
    };

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

#[test]
fn partial_zero_event_requirement_policy_is_typed_fail_closed()
-> Result<(), Box<dyn std::error::Error>> {
    let observations = observations(1_000)?;
    let mut requirements = requirements()?;
    requirements.retain(|requirement| {
        *requirement
            != ZeroEventSafetyRequirement::new(
                ZeroEventSafetyMetric::ProtectedValueDisclosure,
                ZeroEventSafetyThreshold::new(10_000, 9_500).expect("valid fixed test threshold"),
            )
    });

    let error = match decide_commercial_release_with_zero_event_safety(
        passed_benchmarks(),
        &[],
        &observations,
        &requirements,
    ) {
        Err(error) => error,
        Ok(report) => {
            return Err(std::io::Error::other(format!(
                "partial zero-event safety policy produced release decision {:?}",
                report.decision()
            ))
            .into());
        }
    };

    assert_eq!(
        error,
        CommercialReleaseAcceptanceError::ZeroEventSafetyGate(
            ZeroEventSafetyGateError::MissingRequirement(
                ZeroEventSafetyMetric::ProtectedValueDisclosure
            )
        )
    );
    assert_eq!(
        error.to_string(),
        "invalid zero-event safety gate: commercial release safety policy is missing mandatory requirement: protected_value_disclosure_rate"
    );
    assert!(error.source().is_some());
    Ok(())
}
