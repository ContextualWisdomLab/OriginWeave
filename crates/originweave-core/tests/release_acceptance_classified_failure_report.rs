use originweave_core::benchmark_failure::BenchmarkFailureClass;
use originweave_core::release_acceptance::{
    decide_release_with_classified_benchmark_evidence, BenchmarkFailureEvidence, BenchmarkSuite,
    BenchmarkSuiteEvidence, ReleaseDecision, ReleaseDecisionError,
};

fn passed(suite: BenchmarkSuite) -> BenchmarkSuiteEvidence {
    BenchmarkSuiteEvidence::Passed(suite)
}

#[test]
fn classified_environment_failures_remain_inconclusive_and_are_retained() {
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
    )
    .expect("distinct mandatory suite evidence should be accepted");

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
}

#[test]
fn classified_product_failure_rejects_release_and_retains_cause() {
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
    )
    .expect("distinct mandatory suite evidence should be accepted");

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
}

#[test]
fn duplicate_suite_evidence_still_fails_closed() {
    let error = decide_release_with_classified_benchmark_evidence(
        [
            passed(BenchmarkSuite::ControlledDeterministic),
            BenchmarkSuiteEvidence::Failure {
                suite: BenchmarkSuite::ControlledDeterministic,
                classification: BenchmarkFailureClass::BenchmarkDefect,
            },
        ],
        &[],
        &[],
    )
    .expect_err("duplicate suite evidence must fail closed");

    assert_eq!(
        error,
        ReleaseDecisionError::DuplicateSuite(BenchmarkSuite::ControlledDeterministic)
    );
}
