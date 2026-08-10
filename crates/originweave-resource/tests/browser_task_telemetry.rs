#![allow(clippy::expect_used)]

use originweave_resource::{
    BrowserTaskTelemetry, BrowserTaskTelemetryError, ResourceBudget, ResourceGovernor,
    ResourceSnapshot,
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
