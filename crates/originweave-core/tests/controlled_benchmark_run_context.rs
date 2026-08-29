use originweave_core::controlled_benchmark::{
    CONTROLLED_DETERMINISTIC_REGISTRY_VERSION, ControlledBenchmarkRunContext,
    ControlledBenchmarkSuiteError, ControlledBenchmarkSupportProfile,
    evaluate_controlled_benchmark_suite_for_run,
};
use originweave_core::release_acceptance::BenchmarkSuiteOutcome;

fn run_context() -> ControlledBenchmarkRunContext<'static> {
    ControlledBenchmarkRunContext {
        originweave_revision: "originweave@542ca1e9c0a863595b8b6697790005d2471f5413",
        chromium_revision: "chromium@140.0.7339.82",
        os_image: "ubuntu-24.04@sha256:0123456789abcdef",
        hardware_profile: "x86_64-4cpu-16gb",
        protocol_adapters: "webdriver-bidi=2025-08;cdp=140",
        model_provider: "none",
        reasoning_configuration: "deterministic-browser-oracle-v1",
        fixture_corpus_version: "controlled-deterministic-v1",
        random_seed_set: "seeds-v1",
    }
}

fn base_profile() -> ControlledBenchmarkSupportProfile {
    ControlledBenchmarkSupportProfile {
        manifest_v3: false,
        native_messaging: false,
    }
}

#[test]
fn matching_reproducibility_context_preserves_suite_inconclusive_state() {
    let context = run_context();

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            context,
            context,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Ok(BenchmarkSuiteOutcome::Inconclusive)
    );
}

#[test]
fn chromium_revision_mismatch_fails_closed_before_suite_acceptance() {
    let expected = run_context();
    let mut observed = expected;
    observed.chromium_revision = "chromium@140.0.7339.83";

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            expected,
            observed,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(ControlledBenchmarkSuiteError::RunContextMismatch {
            field: "chromium_revision",
        })
    );
}

#[test]
fn blank_reproducibility_context_field_fails_closed() {
    let expected = run_context();
    let mut observed = expected;
    observed.random_seed_set = " ";

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            expected,
            observed,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(ControlledBenchmarkSuiteError::InvalidRunContext {
            field: "random_seed_set",
        })
    );
}
