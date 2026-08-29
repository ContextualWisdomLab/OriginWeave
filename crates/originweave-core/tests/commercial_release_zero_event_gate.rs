use originweave_core::{
    release_acceptance::{
        BenchmarkSuite, BenchmarkSuiteEvidence, ReleaseDecision, ZeroEventSafetyEvidence,
        ZeroEventSafetyMetric, ZeroEventSafetyObservation,
    },
    zero_event_safety_gate::{
        decide_commercial_release_with_zero_event_safety, ZeroEventSafetyRequirement,
    },
    zero_event_threshold::ZeroEventSafetyThreshold,
};

#[test]
fn quantitative_safety_threshold_miss_blocks_release_acceptance(
) -> Result<(), Box<dyn std::error::Error>> {
    let observation = ZeroEventSafetyObservation::new(
        ZeroEventSafetyMetric::UnauthorizedAction,
        ZeroEventSafetyEvidence::new(100, 9_500)?,
    );
    let requirement = ZeroEventSafetyRequirement::new(
        ZeroEventSafetyMetric::UnauthorizedAction,
        ZeroEventSafetyThreshold::new(10_000, 9_500)?,
    );

    let report = decide_commercial_release_with_zero_event_safety(
        BenchmarkSuite::ALL.into_iter().map(BenchmarkSuiteEvidence::Passed),
        &[],
        &[observation],
        &[requirement],
    )?;

    assert_eq!(report.benchmark_report().decision(), ReleaseDecision::Accepted);
    assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
    Ok(())
}
