#![allow(clippy::expect_used)]

use originweave_resource::{
    BrowserRssSampleError, BrowserTaskTelemetry, BrowserTaskTelemetryError, ResourceBudget,
    ResourceGovernor, ResourceSnapshot, parse_linux_proc_status_rss_bytes,
    sample_linux_process_rss_bytes,
};

const MEBIBYTE_BYTES: u64 = 1_048_576;

#[test]
fn browser_task_telemetry_records_real_runtime_measurements() {
    let telemetry = BrowserTaskTelemetry::new(268_435_456, 16_384, 125, 2_000)
        .expect("bounded browser telemetry");

    assert_eq!(telemetry.browser_rss_bytes(), 268_435_456);
    assert_eq!(telemetry.observation_bytes(), 16_384);
    assert_eq!(telemetry.action_latency_milliseconds(), 125);
    assert_eq!(telemetry.task_duration_milliseconds(), 2_000);
}

#[test]
fn browser_task_telemetry_allows_an_empty_observation() {
    let telemetry = BrowserTaskTelemetry::new(134_217_728, 0, 40, 500)
        .expect("a task may produce a bounded empty observation");

    assert_eq!(telemetry.observation_bytes(), 0);
}

#[test]
fn browser_task_telemetry_rejects_impossible_measurements() {
    assert_eq!(
        BrowserTaskTelemetry::new(0, 1, 1, 1),
        Err(BrowserTaskTelemetryError::ZeroBrowserRss)
    );
    assert_eq!(
        BrowserTaskTelemetry::new(1, 1, 0, 0),
        Err(BrowserTaskTelemetryError::ZeroTaskDuration)
    );
    assert_eq!(
        BrowserTaskTelemetry::new(1, 1, 11, 10),
        Err(
            BrowserTaskTelemetryError::ActionLatencyExceedsTaskDuration {
                action_latency_milliseconds: 11,
                task_duration_milliseconds: 10,
            }
        )
    );
}

#[test]
fn measured_browser_rss_drives_governor_with_ceiling_mebibytes() {
    let telemetry = BrowserTaskTelemetry::new(256 * MEBIBYTE_BYTES + 1, 8_192, 20, 200)
        .expect("measured browser telemetry");
    let snapshot = ResourceSnapshot::from_browser_task_telemetry(telemetry, 0, 1, false, 10, 1);
    let budget = ResourceBudget::new(257, 512, 1, 2, 4, 20).expect("resource budget");

    let plan = ResourceGovernor::new(budget).decide(snapshot);

    assert!(plan.spill_observation_cache());
    assert!(!plan.pause_current_agent());
    assert!(!plan.reject_new_agent_work());
}

#[test]
fn exact_mebibyte_browser_rss_is_not_overstated() {
    let telemetry = BrowserTaskTelemetry::new(256 * MEBIBYTE_BYTES, 8_192, 20, 200)
        .expect("measured browser telemetry");
    let snapshot = ResourceSnapshot::from_browser_task_telemetry(telemetry, 0, 1, false, 10, 1);
    let budget = ResourceBudget::new(257, 512, 1, 2, 4, 20).expect("resource budget");

    let plan = ResourceGovernor::new(budget).decide(snapshot);

    assert!(plan.is_noop());
}

#[test]
fn browser_rss_conversion_is_overflow_safe_at_u64_max() {
    let telemetry =
        BrowserTaskTelemetry::new(u64::MAX, 0, 0, 1).expect("maximum representable measured RSS");
    let snapshot = ResourceSnapshot::from_browser_task_telemetry(telemetry, 0, 1, false, 10, 1);
    let ceiling_mebibytes = u64::MAX / MEBIBYTE_BYTES + 1;
    let budget = ResourceBudget::new(ceiling_mebibytes, u64::MAX, 1, 2, 4, 20)
        .expect("large resource budget");

    let plan = ResourceGovernor::new(budget).decide(snapshot);

    assert!(plan.spill_observation_cache());
    assert!(!plan.pause_current_agent());
}

#[test]
fn linux_proc_status_parser_returns_rss_bytes_from_kernel_kibibytes() {
    let status = "Name:\tchrome\nState:\tS (sleeping)\nVmRSS:\t  262145 kB\nThreads:\t42\n";

    assert_eq!(
        parse_linux_proc_status_rss_bytes(status),
        Ok(262_145 * 1_024)
    );
}

#[test]
fn linux_proc_status_parser_rejects_missing_duplicate_malformed_and_overflow_values() {
    assert_eq!(
        parse_linux_proc_status_rss_bytes("Name:\tchrome\n"),
        Err(BrowserRssSampleError::MissingVmRss)
    );
    assert_eq!(
        parse_linux_proc_status_rss_bytes("VmRSS: 1 kB\nVmRSS: 2 kB\n"),
        Err(BrowserRssSampleError::DuplicateVmRss)
    );
    for malformed in [
        "VmRSS:\n",
        "VmRSS: many kB\n",
        "VmRSS: +1 kB\n",
        "VmRSS: 10\n",
        "VmRSS: 10 kB extra\n",
    ] {
        assert_eq!(
            parse_linux_proc_status_rss_bytes(malformed),
            Err(BrowserRssSampleError::InvalidVmRss)
        );
    }
    assert_eq!(
        parse_linux_proc_status_rss_bytes("VmRSS: 10 MB\n"),
        Err(BrowserRssSampleError::UnsupportedVmRssUnit)
    );
    assert_eq!(
        parse_linux_proc_status_rss_bytes(&format!("VmRSS: {} kB\n", u64::MAX)),
        Err(BrowserRssSampleError::VmRssOverflow)
    );
}

#[test]
fn linux_process_rss_sampler_reads_the_current_process_and_rejects_invalid_or_missing_processes() {
    assert_eq!(
        sample_linux_process_rss_bytes(0),
        Err(BrowserRssSampleError::InvalidProcessId)
    );

    #[cfg(target_os = "linux")]
    {
        let current = sample_linux_process_rss_bytes(std::process::id())
            .expect("the current Linux test process has a readable /proc status");
        assert!(current > 0);
        assert_eq!(
            sample_linux_process_rss_bytes(u32::MAX),
            Err(BrowserRssSampleError::ProcessStatusUnavailable)
        );
    }

    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        sample_linux_process_rss_bytes(std::process::id()),
        Err(BrowserRssSampleError::UnsupportedPlatform)
    );
}
