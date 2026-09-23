use std::error::Error;

use originweave_core::OriginError;

fn assert_standard_error<T: Error>() {}

#[test]
fn origin_error_exposes_a_stable_standard_error_contract() {
    assert_standard_error::<OriginError>();

    let cases = [
        (
            OriginError::MissingScheme,
            "origin must include a scheme followed by ://",
        ),
        (
            OriginError::UnsupportedScheme,
            "origin scheme must be http or https",
        ),
        (
            OriginError::InsecureRemoteOrigin,
            "remote HTTP origins are forbidden; use HTTPS or a loopback HTTP origin",
        ),
        (
            OriginError::MissingAuthority,
            "origin must include a non-empty authority after the scheme",
        ),
        (
            OriginError::UserInfoNotAllowed,
            "origin authority must not contain user information",
        ),
        (
            OriginError::PathNotAllowed,
            "origin must not contain a path, query, or fragment",
        ),
        (
            OriginError::InvalidAuthority,
            "origin authority is malformed or ambiguous",
        ),
        (
            OriginError::AmbiguousNumericHost,
            "origin host uses an ambiguous browser-style numeric address spelling",
        ),
        (
            OriginError::InvalidPort,
            "origin port must be a nonzero numeric value within 1..=65535",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
