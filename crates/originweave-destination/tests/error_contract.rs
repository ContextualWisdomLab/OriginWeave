#![allow(clippy::expect_used)]

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationError, MAX_REDIRECT_HOPS, MAX_RESOLUTION_ADDRESS_COUNT,
    RedirectError, RedirectTargetDigest, RedirectTargetDigestError,
};

const DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn address(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

fn assert_standard_error(error: &(dyn Error + 'static), expected: &str) {
    assert_eq!(error.to_string(), expected);
    assert!(error.source().is_none());
}

#[test]
fn redirect_digest_error_implements_display_and_error() {
    assert_standard_error(
        &RedirectTargetDigestError::InvalidFormat,
        "redirect target digest must be sha256: followed by 64 lowercase hexadecimal digits",
    );
}

#[test]
fn redirect_errors_implement_display_and_error_without_sensitive_targets() {
    let source = origin("https://source.example");
    let target = origin("https://target.example");
    let other = origin("https://other.example");
    let digest = RedirectTargetDigest::parse(DIGEST).expect("valid digest");

    let cases: Vec<(RedirectError, String)> = vec![
        (
            RedirectError::InvalidMaximumHops { maximum_hops: 0 },
            format!("redirect maximum 0 is outside 1..={MAX_REDIRECT_HOPS}"),
        ),
        (
            RedirectError::RedirectLimitExceeded,
            "redirect limit exceeded".to_owned(),
        ),
        (
            RedirectError::OriginNotGranted {
                origin: target.clone(),
            },
            format!("redirect target origin is not granted: {target}"),
        ),
        (
            RedirectError::ResolutionOriginMismatch {
                target_origin: target.clone(),
                resolution_origin: other.clone(),
            },
            format!("redirect resolution origin {other} does not match target {target}"),
        ),
        (
            RedirectError::InsecureSchemeDowngrade {
                source_origin: source.clone(),
                target_origin: target.clone(),
            },
            format!("insecure redirect downgrade from {source} to {target}"),
        ),
        (
            RedirectError::RedirectCycle {
                target_digest: digest,
            },
            format!("redirect target was already visited: {DIGEST}"),
        ),
    ];

    for (error, expected) in cases {
        assert_standard_error(&error, &expected);
    }
}

#[test]
fn destination_errors_implement_display_and_error() {
    let public = address(8, 8, 8, 8);
    let private = address(10, 0, 0, 1);
    let other = address(1, 1, 1, 1);
    let cases: Vec<(DestinationError, String)> = vec![
        (
            DestinationError::EmptyResolution,
            "resolver answer is empty".to_owned(),
        ),
        (
            DestinationError::ResolutionAddressLimitExceeded {
                maximum_count: MAX_RESOLUTION_ADDRESS_COUNT,
            },
            format!(
                "resolver answer exceeds the maximum of {MAX_RESOLUTION_ADDRESS_COUNT} addresses"
            ),
        ),
        (
            DestinationError::AddressClassDenied {
                address: private,
                address_class: AddressClass::PrivateNetwork,
            },
            format!("destination address {private} is denied as PrivateNetwork"),
        ),
        (
            DestinationError::LocalhostResolutionNotLoopback {
                address: public,
                address_class: AddressClass::Public,
            },
            format!("localhost resolved to non-loopback address {public} classified as Public"),
        ),
        (
            DestinationError::LiteralOriginAddressMismatch {
                origin_address: public,
                resolved_address: other,
            },
            format!(
                "literal origin address {public} does not match resolved address {other}"
            ),
        ),
        (
            DestinationError::UnapprovedConnectionAddress { address: other },
            format!(
                "connection address {other} is not in the approved resolution snapshot"
            ),
        ),
        (
            DestinationError::ResolutionSetExpanded { address: other },
            format!("refreshed DNS answer introduced unapproved address {other}"),
        ),
    ];

    for (error, expected) in cases {
        assert_standard_error(&error, &expected);
    }
}
