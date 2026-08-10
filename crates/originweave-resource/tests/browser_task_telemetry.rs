#![allow(clippy::expect_used)]

use originweave_resource::{BrowserTaskTelemetry, BrowserTaskTelemetryError};

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
