#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_http::{
    AlpnHttp11Policy, DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_TRAILER_FIELD_COUNT, HttpClientPolicy, HttpPolicyLimits, IntegrityRequirement,
};

#[test]
fn named_limits_bind_partial_changes_to_the_intended_budget() {
    let mut limits = HttpPolicyLimits::strict_defaults();
    limits.max_header_name_bytes = 64;

    let policy = HttpClientPolicy::from_limits(
        Duration::from_secs(7),
        limits,
        AlpnHttp11Policy::PermitAbsentForManagedLoopback,
        IntegrityRequirement::Optional,
    )
    .expect("named HTTP policy limits");

    assert_eq!(policy.max_header_name_bytes(), 64);
    assert_eq!(
        policy.max_header_field_count(),
        DEFAULT_MAX_HEADER_FIELD_COUNT
    );
    assert_eq!(
        policy.max_trailer_field_count(),
        DEFAULT_MAX_TRAILER_FIELD_COUNT
    );
    assert_ne!(
        policy.max_header_name_bytes(),
        DEFAULT_MAX_HEADER_NAME_BYTES
    );
}
