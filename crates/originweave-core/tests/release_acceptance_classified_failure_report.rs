use originweave_core::benchmark_failure::BenchmarkFailureClass;
use originweave_core::release_acceptance::{
    BenchmarkFailureEvidence, BenchmarkSuite, BenchmarkSuiteEvidence, ReleaseDecision,
    ReleaseDecisionError, decide_release_with_classified_benchmark_evidence,
};

fn passed(suite: BenchmarkSuite) -> BenchmarkSuiteEvidence {
    BenchmarkSuiteEvidence::Passed(suite)
}

#[test]
fn classified_environment_failures_remain_inconclusive_and_are_retained(
) -> Result<(), ReleaseDecisionError> {
    let report = decide_release_with_classified_benchmark_evidence(
        [
            passed(BenchmarkSuite::ControlledDeterministic),
            BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::WebCompatibility,
                classification: BenchmarkFailureClass::ExternalOutage,
            },
            passed(BenchmarkSuite::SecurityAdversarial),
            BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::ReliabilityRecovery,
                classification: BenchmarkFailureClass::InfrastructureFailure,
            },
            passed(BenchmarkSuite::EnterpriseOperability),
        ],
        &[],
        &[],
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
    assert_eq!(
        report.inconclusive_suites(),
        &[
            BenchmarkSuite::WebCompatibility,
            BenchmarkSuite::ReliabilityRecovery,
        ]
    );
    assert_eq!(
        report.benchmark_failures(),
        &[
            BenchmarkFailureEvidence::new(
                BenchmarkSuite::WebCompatibility,
                BenchmarkFailureClass::ExternalOutage,
            ),
            BenchmarkFailureEvidence::new(
                BenchmarkSuite::ReliabilityRecovery,
                BenchmarkFailureClass::InfrastructureFailure,
            ),
        ]
    );
    Ok(())
}

#[test]
fn classified_product_failure_rejects_release_and_retains_cause(
) -> Result<(), ReleaseDecisionError> {
    let report = decide_release_with_classified_benchmark_evidence(
        [
            passed(BenchmarkSuite::ControlledDeterministic),
            passed(BenchmarkSuite::WebCompatibility),
            BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::SecurityAdversarial,
                classification: BenchmarkFailureClass::DeterministicContractFailure,
            },
            passed(BenchmarkSuite::ReliabilityRecovery),
            passed(BenchmarkSuite::EnterpriseOperability),
        ],
        &[],
        &[],
    )?;

    assert_eq!(report.decision(), ReleaseDecision::Rejected);
    assert_eq!(
        report.failed_suites(),
        &[BenchmarkSuite::SecurityAdversarial]
    );
    assert_eq!(report.benchmark_failures().len(), 1);
    assert_eq!(
        report.benchmark_failures()[0].classification(),
        BenchmarkFailureClass::DeterministicContractFailure
    );
    assert_eq!(
        report.benchmark_failures()[0].suite(),
        BenchmarkSuite::SecurityAdversarial
    );
    Ok(())
}

#[test]
fn duplicate_suite_evidence_still_fails_closed() {
    let result = decide_release_with_classified_benchmark_evidence(
        [
            passed(BenchmarkSuite::ControlledDeterministic),
            BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::ControlledDeterministic,
                classification: BenchmarkFailureClass::BenchmarkDefect,
            },
        ],
        &[],
        &[],
    );

    assert_eq!(
        result,
        Err(ReleaseDecisionError::DuplicateSuite(
            BenchmarkSuite::ControlledDeterministic
        ))
    );
}

struct DuplicateThenTracked {
    step: u8,
}

impl Iterator for DuplicateThenTracked {
    type Item = BenchmarkSuiteEvidence;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.step {
            0 => passed(BenchmarkSuite::ControlledDeterministic),
            1 => BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::ControlledDeterministic,
                classification: BenchmarkFailureClass::BenchmarkDefect,
            },
            2 => passed(BenchmarkSuite::WebCompatibility),
            _ => return None,
        };
        self.step += 1;
        Some(item)
    }
}

#[test]
fn duplicate_suite_stops_consuming_evidence_at_first_duplicate() {
    let mut evidence = DuplicateThenTracked { step: 0 };
    let result =
        decide_release_with_classified_benchmark_evidence(&mut evidence, &[], &[]);

    assert_eq!(
        result,
        Err(ReleaseDecisionError::DuplicateSuite(
            BenchmarkSuite::ControlledDeterministic
        ))
    );
    assert_eq!(evidence.step, 2);
}
