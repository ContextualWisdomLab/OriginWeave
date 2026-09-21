use originweave_core::controlled_benchmark::{
    CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE, CONTROLLED_DETERMINISTIC_REGISTRY_VERSION,
    ControlledBenchmarkRunContext, ControlledBenchmarkSuiteError,
    ControlledBenchmarkSupportProfile, evaluate_controlled_benchmark_suite_for_run,
};

const UNICODE_18_DEFAULT_IGNORABLE_SOURCE_RANGES: &[(u32, u32)] = &[
    (0x00ad, 0x00ad),
    (0x034f, 0x034f),
    (0x061c, 0x061c),
    (0x115f, 0x1160),
    (0x17b4, 0x17b5),
    (0x180b, 0x180d),
    (0x180e, 0x180e),
    (0x180f, 0x180f),
    (0x200b, 0x200f),
    (0x202a, 0x202e),
    (0x2060, 0x2064),
    (0x2065, 0x2065),
    (0x2066, 0x206f),
    (0x3164, 0x3164),
    (0xfe00, 0xfe0f),
    (0xfeff, 0xfeff),
    (0xffa0, 0xffa0),
    (0xfff0, 0xfff8),
    (0x1bca0, 0x1bca3),
    (0x1d173, 0x1d17a),
    (0xe0000, 0xe0000),
    (0xe0001, 0xe0001),
    (0xe0002, 0xe001f),
    (0xe0020, 0xe007f),
    (0xe0080, 0xe00ff),
    (0xe0100, 0xe01ef),
    (0xe01f0, 0xe0fff),
];

fn run_context(reasoning_configuration: &str) -> ControlledBenchmarkRunContext<'_> {
    ControlledBenchmarkRunContext {
        originweave_revision: "originweave@542ca1e9c0a863595b8b6697790005d2471f5413",
        chromium_revision: "chromium@140.0.7339.82",
        os_image: "ubuntu-24.04@sha256:0123456789abcdef",
        hardware_profile: "x86_64-4cpu-16gb",
        protocol_adapters: "webdriver-bidi=2025-08;cdp=140",
        model_provider: "none",
        reasoning_configuration,
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
fn every_unicode_18_default_ignorable_source_scalar_fails_closed() {
    assert_eq!(
        CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE,
        "unicode-18.0.0-default-ignorable-exclusion"
    );

    let mut tested_scalar_count = 0_u32;

    for &(start, end) in UNICODE_18_DEFAULT_IGNORABLE_SOURCE_RANGES {
        for code_point in start..=end {
            let hostile_scalar = char::from_u32(code_point).unwrap_or_else(|| {
                panic!("Unicode 18 DICP fixture contains a non-scalar U+{code_point:04X}")
            });
            let hostile = format!("runner{hostile_scalar}suffix");
            let context = run_context(&hostile);

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
                "Unicode 18.0.0 Default_Ignorable_Code_Point escaped the benchmark identity profile: U+{code_point:04X}"
            );

            tested_scalar_count += 1;
        }
    }

    assert_eq!(tested_scalar_count, 4_174);
}
