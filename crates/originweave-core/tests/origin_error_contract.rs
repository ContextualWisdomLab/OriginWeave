use std::error::Error as _;

use originweave_core::OriginError;

#[test]
fn origin_errors_have_a_stable_standard_error_contract() {
    let cases = [
        (OriginError::MissingScheme, "origin must include a scheme"),
        (
            OriginError::UnsupportedScheme,
            "origin scheme must be HTTPS or loopback HTTP",
        ),
        (
            OriginError::InsecureRemoteOrigin,
            "HTTP origins are restricted to loopback hosts",
        ),
        (
            OriginError::MissingAuthority,
            "origin authority is required",
        ),
        (
            OriginError::UserInfoNotAllowed,
            "origin user information is not allowed",
        ),
        (
            OriginError::PathNotAllowed,
            "origin must not include a path, query, or fragment",
        ),
        (OriginError::InvalidAuthority, "origin authority is invalid"),
        (
            OriginError::AmbiguousNumericHost,
            "origin host uses an ambiguous numeric spelling",
        ),
        (
            OriginError::InvalidPort,
            "origin port must be within 1..=65535",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}

fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn origin_error_is_send_sync_and_standard_error_compatible() {
    assert_standard_error_contract::<OriginError>();
}
