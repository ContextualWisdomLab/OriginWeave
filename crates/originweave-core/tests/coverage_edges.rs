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
    let host = [
        label.as_str(),
        label.as_str(),
        label.as_str(),
        label.as_str(),
    ]
    .join(".");
    assert_eq!(host.len(), 255);
    assert_eq!(
        Origin::parse(&format!("https://{host}")),
        Err(OriginError::InvalidAuthority)
    );
}

#[test]
fn origin_accepts_exact_dns_label_and_authority_limits() {
    let max_label = "a".repeat(63);
    let with_max_label = format!("https://{max_label}.example");
    let origin = Origin::parse(&with_max_label).expect("63-byte DNS label is the accepted maximum");
    assert_eq!(origin.host(), format!("{max_label}.example"));

    let labels = [63_usize, 63, 63, 61].map(|length| "a".repeat(length));
    let max_host = labels.join(".");
    assert_eq!(max_host.len(), 253);
    Origin::parse(&format!("https://{max_host}")).expect("253-byte authority is accepted");

    let oversized_label = "a".repeat(64);
    assert_eq!(
        Origin::parse(&format!("https://{oversized_label}.example")),
        Err(OriginError::InvalidAuthority)
    );

    let overlong_labels = [63_usize, 63, 63, 62].map(|length| "a".repeat(length));
    let overlong_host = overlong_labels.join(".");
    assert_eq!(overlong_host.len(), 254);
    assert_eq!(
        Origin::parse(&format!("https://{overlong_host}")),
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

#[test]
fn origin_rejects_an_invalid_port_after_ipv6() {
    assert_eq!(
        Origin::parse("https://[::1]:0"),
        Err(OriginError::InvalidPort)
    );
}

#[test]
fn origin_accepts_the_maximum_authority_port() {
    let origin = Origin::parse("https://example.com:65535").expect("65535 is a valid port");
    assert_eq!(origin.as_str(), "https://example.com:65535");

    let ipv6 = Origin::parse("https://[2001:db8::1]:65535")
        .expect("65535 is valid for an IPv6 authority too");
    assert_eq!(ipv6.as_str(), "https://[2001:db8::1]:65535");
}
