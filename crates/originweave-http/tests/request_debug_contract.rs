#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_http::HttpRequestTarget;

#[test]
fn request_target_debug_never_exposes_query_values() {
    let target = HttpRequestTarget::parse(
        Origin::parse("https://example.com").expect("origin"),
        "/account/profile?access_token=do-not-log-this&next=%2Fbilling",
    )
    .expect("target");

    let debug = format!("{target:?}");

    assert!(debug.contains("HttpRequestTarget"));
    assert!(debug.contains("query_present"));
    assert!(debug.contains("target_hash"));
    assert!(debug.contains("path_prefix"));
    assert!(!debug.contains("encoded_path_and_query"));
    assert!(!debug.contains("access_token"));
    assert!(!debug.contains("do-not-log-this"));
    assert!(!debug.contains("next=%2Fbilling"));
}
