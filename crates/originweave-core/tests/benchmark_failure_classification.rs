use originweave_core::benchmark_failure::BenchmarkFailureClass;
use originweave_core::release_acceptance::BenchmarkSuiteOutcome;

#[test]
fn benchmark_failure_classes_have_stable_evidence_identifiers() {
    let expected = [
        (
            BenchmarkFailureClass::DeterministicContractFailure,
            "deterministic_contract_failure",
        ),
        (
            BenchmarkFailureClass::StochasticModelFailure,
            "stochastic_model_failure",
        ),
        (
            BenchmarkFailureClass::ExternalSiteDrift,
            "external_site_drift",
        ),
        (BenchmarkFailureClass::ExternalOutage, "external_outage"),
        (
            BenchmarkFailureClass::UnsupportedCapability,
            "unsupported_capability",
        ),
        (
            BenchmarkFailureClass::InfrastructureFailure,
            "infrastructure_failure",
        ),
        (BenchmarkFailureClass::BenchmarkDefect, "benchmark_defect"),
    ];

    for (classification, identifier) in expected {
        assert_eq!(classification.as_str(), identifier);
    }
}

#[test]
fn product_threshold_failures_remain_failed_suite_evidence() {
    for classification in [
        BenchmarkFailureClass::DeterministicContractFailure,
        BenchmarkFailureClass::StochasticModelFailure,
    ] {
        assert_eq!(
            classification.suite_outcome(),
            BenchmarkSuiteOutcome::Failed
        );
    }
}

#[test]
fn environment_or_benchmark_uncertainty_remains_inconclusive() {
    for classification in [
        BenchmarkFailureClass::ExternalSiteDrift,
        BenchmarkFailureClass::ExternalOutage,
        BenchmarkFailureClass::UnsupportedCapability,
        BenchmarkFailureClass::InfrastructureFailure,
        BenchmarkFailureClass::BenchmarkDefect,
    ] {
        assert_eq!(
            classification.suite_outcome(),
            BenchmarkSuiteOutcome::Inconclusive
        );
    }
}
