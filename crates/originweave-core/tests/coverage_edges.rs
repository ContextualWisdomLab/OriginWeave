#![allow(clippy::expect_used)]

use originweave_core::{Origin, OriginError};

#[test]
fn origin_rejects_embedded_control_characters() {
    let input = format!("https://exam{}ple.com", char::from(0));
    assert_eq!(Origin::parse(&input), Err(OriginError::InvalidAuthority));
}

#[test]
fn origin_rejects_authorities_longer_than_the_dns_limit() {
    let label = "a".repeat(63);
    let host = [label.as_str(), label.as_str(), label.as_str(), label.as_str()].join(".");
    assert_eq!(host.len(), 255);
    assert_eq!(
        Origin::parse(&format!("https://{host}")),
        Err(OriginError::InvalidAuthority)
    );
}

#[test]
fn origin_rejects_a_leading_empty_dns_label() {
    assert_eq!(
        Origin::parse("https://.example.com"),
        Err(OriginError::InvalidAuthority)
    );
}
