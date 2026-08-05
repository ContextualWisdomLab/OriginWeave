#![allow(clippy::expect_used)]

use originweave_core::{Origin, OriginError};

/// Browser-special IPv4 spellings must never enter the policy boundary as DNS hosts.
#[test]
fn origin_rejects_browser_special_ipv4_spellings() {
    for input in [
        "https://127.1",
        "https://2130706433",
        "https://0x7f000001",
        "https://0177.0.0.1",
        "https://example.1",
    ] {
        assert_eq!(
            Origin::parse(input),
            Err(OriginError::InvalidAuthority),
            "input={input}"
        );
    }
}

/// Canonical dotted-decimal IPv4 remains a stable, browser-equivalent origin.
#[test]
fn origin_keeps_canonical_dotted_decimal_ipv4() {
    let origin = Origin::parse("https://127.0.0.1:443").expect("canonical IPv4 origin");
    assert_eq!(origin.as_str(), "https://127.0.0.1");
}
