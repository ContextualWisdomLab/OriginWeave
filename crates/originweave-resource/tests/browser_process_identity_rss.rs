use originweave_resource::{
    BrowserRssSampleError, LinuxProcessIdentity, parse_linux_proc_stat_start_time_ticks,
    read_linux_process_identity, sample_linux_process_identity_rss_bytes,
    sample_linux_process_identity_set_rss_bytes, verify_linux_process_identity,
};

fn proc_stat_with_start_time(start_time_ticks: u64) -> String {
    format!(
        "42 (chrome worker)) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 {start_time_ticks} 999"
    )
}

#[test]
fn proc_stat_parser_extracts_start_time_after_complex_comm() {
    assert_eq!(
        parse_linux_proc_stat_start_time_ticks(&proc_stat_with_start_time(987_654)),
        Ok(987_654)
    );
}

#[test]
fn proc_stat_parser_rejects_missing_or_malformed_start_time() {
    assert_eq!(
        parse_linux_proc_stat_start_time_ticks("42 chrome S 1 2 3"),
        Err(BrowserRssSampleError::InvalidProcessStat)
    );
    assert_eq!(
        parse_linux_proc_stat_start_time_ticks(
            &proc_stat_with_start_time(0).replace(" 0 999", " nope 999")
        ),
        Err(BrowserRssSampleError::InvalidProcessStat)
    );
}

#[test]
fn process_identity_binds_pid_to_kernel_start_time() -> Result<(), BrowserRssSampleError> {
    let identity = LinuxProcessIdentity::new(42, 987_654)?;
    assert_eq!(identity.process_id(), 42);
    assert_eq!(identity.start_time_ticks(), 987_654);
    assert_eq!(
        verify_linux_process_identity(identity, &proc_stat_with_start_time(987_654)),
        Ok(())
    );
    assert_eq!(
        verify_linux_process_identity(identity, &proc_stat_with_start_time(987_655)),
        Err(BrowserRssSampleError::ProcessIdentityChanged)
    );
    Ok(())
}

#[test]
fn process_identity_rejects_zero_pid() {
    assert_eq!(
        LinuxProcessIdentity::new(0, 1),
        Err(BrowserRssSampleError::InvalidProcessId)
    );
}

#[test]
fn linux_identity_sampler_rejects_pid_reuse_and_samples_current_process() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let identity = read_linux_process_identity(std::process::id())
            .map_err(|error| format!("read current process identity: {error:?}"))?;
        let rss_bytes = sample_linux_process_identity_rss_bytes(identity)
            .map_err(|error| format!("sample current process identity: {error:?}"))?;
        assert!(rss_bytes > 0);
        assert_eq!(rss_bytes % 1_024, 0);

        let stale = LinuxProcessIdentity::new(
            identity.process_id(),
            identity.start_time_ticks().wrapping_add(1),
        )
        .map_err(|error| format!("construct stale identity: {error:?}"))?;
        assert_eq!(
            sample_linux_process_identity_rss_bytes(stale),
            Err(BrowserRssSampleError::ProcessIdentityChanged)
        );
        assert_eq!(
            sample_linux_process_identity_set_rss_bytes(&[stale]),
            Err(BrowserRssSampleError::ProcessIdentityChanged)
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        let identity = LinuxProcessIdentity::new(std::process::id(), 1)
            .map_err(|error| format!("construct process identity: {error:?}"))?;
        assert_eq!(
            read_linux_process_identity(std::process::id()),
            Err(BrowserRssSampleError::UnsupportedPlatform)
        );
        assert_eq!(
            sample_linux_process_identity_rss_bytes(identity),
            Err(BrowserRssSampleError::UnsupportedPlatform)
        );
        assert_eq!(
            sample_linux_process_identity_set_rss_bytes(&[identity]),
            Err(BrowserRssSampleError::UnsupportedPlatform)
        );
    }
    Ok(())
}
