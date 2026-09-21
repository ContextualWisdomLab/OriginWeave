use originweave_core::controlled_benchmark::{
    CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE, CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
    ControlledBenchmarkRunContext, ControlledBenchmarkSuiteError,
    ControlledBenchmarkSupportProfile, evaluate_controlled_benchmark_suite_for_run,
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
    let mismatch = ControlledBenchmarkSuiteError::RunContextMismatch {
        field: "chromium_revision",
    };

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            expected,
            observed,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(mismatch.clone())
    );
    assert_eq!(
        mismatch.to_string(),
        "controlled benchmark run context field chromium_revision does not match the required reproducibility context"
    );
}

#[test]
fn blank_reproducibility_context_field_fails_closed() {
    let expected = run_context();
    let mut observed = expected;
    observed.random_seed_set = " ";
    let invalid = ControlledBenchmarkSuiteError::InvalidRunContext {
        field: "random_seed_set",
    };

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            expected,
            observed,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(invalid.clone())
    );
    assert_eq!(
        invalid.to_string(),
        "controlled benchmark run context field random_seed_set is blank"
    );
}

#[test]
fn blank_required_reproducibility_context_field_fails_closed() {
    let mut expected = run_context();
    expected.hardware_profile = " ";
    let observed = run_context();

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            expected,
            observed,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(ControlledBenchmarkSuiteError::InvalidRunContext {
            field: "hardware_profile",
        })
    );
}

#[test]
fn surrounding_whitespace_reproducibility_context_field_fails_closed() {
    let mut context = run_context();
    context.chromium_revision = " chromium@140.0.7339.82 ";
    let invalid = ControlledBenchmarkSuiteError::NonCanonicalRunContext {
        field: "chromium_revision",
    };

    assert_eq!(
        evaluate_controlled_benchmark_suite_for_run(
            context,
            context,
            CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
            base_profile(),
            &[],
        ),
        Err(invalid.clone())
    );
    assert_eq!(
        invalid.to_string(),
        "controlled benchmark run context field chromium_revision contains non-canonical surrounding whitespace"
    );
}

#[test]
fn control_character_in_reproducibility_context_fails_closed() {
    for hostile in [
        "runner\nspoofed=passed",
        "runner\0suffix",
        "runner\u{0085}suffix",
    ] {
        let mut context = run_context();
        context.reasoning_configuration = hostile;

        assert_eq!(
            evaluate_controlled_benchmark_suite_for_run(
                context,
                context,
                CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
                base_profile(),
                &[],
            ),
            Err(ControlledBenchmarkSuiteError::ControlCharacterRunContext {
                field: "reasoning_configuration",
            }),
            "hostile context identity must not become benchmark evidence: {hostile:?}"
        );
    }
}

#[test]
fn unicode_line_separator_in_reproducibility_context_fails_closed() {
    for hostile in [
        "runner\u{2028}spoofed=passed",
        "runner\u{2029}spoofed=passed",
    ] {
        let mut context = run_context();
        context.reasoning_configuration = hostile;

        assert_eq!(
            evaluate_controlled_benchmark_suite_for_run(
                context,
                context,
                CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
                base_profile(),
                &[],
            ),
            Err(ControlledBenchmarkSuiteError::ControlCharacterRunContext {
                field: "reasoning_configuration",
            }),
            "Unicode line/paragraph separators must not split benchmark evidence identity rendering: {hostile:?}"
        );
    }
}

#[test]
fn bidi_control_in_reproducibility_context_fails_closed() {
    for hostile in [
        "runner\u{061c}suffix",
        "runner\u{200e}suffix",
        "runner\u{200f}suffix",
        "runner\u{202a}suffix",
        "runner\u{202b}suffix",
        "runner\u{202c}suffix",
        "runner\u{202d}suffix",
        "runner\u{202e}suffix",
        "runner\u{2066}suffix",
        "runner\u{2067}suffix",
        "runner\u{2068}suffix",
        "runner\u{2069}suffix",
    ] {
        let mut context = run_context();
        context.reasoning_configuration = hostile;

        assert_eq!(
            evaluate_controlled_benchmark_suite_for_run(
                context,
                context,
                CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
                base_profile(),
                &[],
            ),
            Err(ControlledBenchmarkSuiteError::ControlCharacterRunContext {
                field: "reasoning_configuration",
            }),
            "bidi formatting controls must not become benchmark evidence identity: {hostile:?}"
        );
    }

    let invalid = ControlledBenchmarkSuiteError::ControlCharacterRunContext {
        field: "reasoning_configuration",
    };
    assert_eq!(
        invalid.to_string(),
        "controlled benchmark run context field reasoning_configuration contains a disallowed C0/C1 control, Unicode line/paragraph separator, bidirectional formatting character, or Unicode 18.0.0 Default_Ignorable_Code_Point"
    );
}

#[test]
fn unicode_18_default_ignorable_reproducibility_context_fails_closed() {
    assert_eq!(
        CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE,
        "unicode-18.0.0-default-ignorable-exclusion"
    );

    for hostile_scalar in [
        '\u{00ad}',
        '\u{034f}',
        '\u{061c}',
        '\u{115f}',
        '\u{1160}',
        '\u{17b4}',
        '\u{17b5}',
        '\u{180b}',
        '\u{180f}',
        '\u{200b}',
        '\u{200c}',
        '\u{200d}',
        '\u{2060}',
        '\u{2065}',
        '\u{206f}',
        '\u{3164}',
        '\u{fe00}',
        '\u{fe0f}',
        '\u{feff}',
        '\u{ffa0}',
        '\u{fff0}',
        '\u{fff8}',
        '\u{1bca0}',
        '\u{1bca3}',
        '\u{1d173}',
        '\u{1d17a}',
        '\u{e0000}',
        '\u{e0fff}',
    ] {
        let hostile = format!("runner{hostile_scalar}suffix");
        let expected = run_context();
        let observed = ControlledBenchmarkRunContext {
            reasoning_configuration: &hostile,
            ..run_context()
        };

        assert_eq!(
            evaluate_controlled_benchmark_suite_for_run(
                expected,
                observed,
                CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
                base_profile(),
                &[],
            ),
            Err(ControlledBenchmarkSuiteError::ControlCharacterRunContext {
                field: "reasoning_configuration",
            }),
            "Unicode 18.0.0 Default_Ignorable_Code_Point must not become observed benchmark evidence identity: U+{:04X}",
            hostile_scalar as u32
        );
    }
}

#[test]
fn visible_unicode_reproducibility_context_remains_valid() {
    for visible in ["결정적-ブラウザ-oráculo-v1", "محرك-מבחן-v1"] {
        let mut context = run_context();
        context.reasoning_configuration = visible;

        assert_eq!(
            evaluate_controlled_benchmark_suite_for_run(
                context,
                context,
                CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
                base_profile(),
                &[],
            ),
            Ok(BenchmarkSuiteOutcome::Inconclusive),
            "visible Unicode and RTL scripts remain valid without excluded formatting scalars: {visible:?}"
        );
    }
}
