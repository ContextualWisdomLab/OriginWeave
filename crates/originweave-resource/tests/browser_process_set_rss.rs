#![allow(clippy::expect_used)]

use originweave_resource::{
    BrowserRssSampleError, MAX_BROWSER_PROCESS_SET_SIZE, aggregate_browser_process_rss_samples,
    sample_linux_process_rss_bytes, sample_linux_process_set_rss_bytes,
};

#[test]
fn process_set_aggregation_counts_each_process_once() {
    assert_eq!(
        aggregate_browser_process_rss_samples(&[(11, 1_024), (12, 2_048), (13, 4_096)]),
        Ok(7_168)
    );
}

#[test]
fn process_set_aggregation_rejects_ambiguous_or_unbounded_membership() {
    assert_eq!(
        aggregate_browser_process_rss_samples(&[]),
        Err(BrowserRssSampleError::EmptyProcessSet)
    );
    assert_eq!(
        aggregate_browser_process_rss_samples(&[(0, 1)]),
        Err(BrowserRssSampleError::InvalidProcessId)
    );
    assert_eq!(
        aggregate_browser_process_rss_samples(&[(42, 1), (42, 2)]),
        Err(BrowserRssSampleError::DuplicateProcessId)
    );

    let oversized: Vec<(u32, u64)> = (1..=(MAX_BROWSER_PROCESS_SET_SIZE as u32 + 1))
        .map(|process_id| (process_id, 1))
        .collect();
    assert_eq!(
        aggregate_browser_process_rss_samples(&oversized),
        Err(BrowserRssSampleError::ProcessSetTooLarge)
    );
}

#[test]
fn process_set_aggregation_rejects_overflow_instead_of_undercounting() {
    assert_eq!(
        aggregate_browser_process_rss_samples(&[(1, u64::MAX), (2, 1)]),
        Err(BrowserRssSampleError::ProcessSetRssOverflow)
    );
}

#[test]
fn linux_process_set_sampler_matches_the_single_process_sampler() {
    #[cfg(target_os = "linux")]
    {
        let process_id = std::process::id();
        let single = sample_linux_process_rss_bytes(process_id)
            .expect("the current Linux test process has a readable /proc status");
        assert_eq!(
            sample_linux_process_set_rss_bytes(&[process_id]),
            Ok(single)
        );
    }

    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        sample_linux_process_set_rss_bytes(&[std::process::id()]),
        Err(BrowserRssSampleError::UnsupportedPlatform)
    );
}

#[test]
fn linux_process_set_sampler_fails_closed_for_invalid_or_partial_sets() {
    assert_eq!(
        sample_linux_process_set_rss_bytes(&[]),
        Err(BrowserRssSampleError::EmptyProcessSet)
    );
    assert_eq!(
        sample_linux_process_set_rss_bytes(&[1, 1]),
        Err(BrowserRssSampleError::DuplicateProcessId)
    );
    assert_eq!(
        sample_linux_process_set_rss_bytes(&[0]),
        Err(BrowserRssSampleError::InvalidProcessId)
    );

    #[cfg(target_os = "linux")]
    assert_eq!(
        sample_linux_process_set_rss_bytes(&[std::process::id(), u32::MAX]),
        Err(BrowserRssSampleError::ProcessStatusUnavailable)
    );
}
