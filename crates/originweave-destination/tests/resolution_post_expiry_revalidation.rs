#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{DestinationError, DestinationPolicy, FreshResolutionSnapshot};

fn origin() -> Origin {
    Origin::parse("https://example.com").expect("test origin must parse")
}

fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

#[test]
fn post_expiry_revalidation_establishes_new_authority_without_reviving_the_old_snapshot() {
    let first = ipv4(8, 8, 8, 8);
    let second = ipv4(1, 1, 1, 1);
    let policy = DestinationPolicy::public_web();
    let snapshot = FreshResolutionSnapshot::approve(
        origin(),
        [first, second],
        &policy,
        Duration::from_secs(10),
        Duration::from_secs(4),
    )
    .expect("initial bounded freshness authority");

    let expiry = Duration::from_secs(14);
    assert_eq!(
        snapshot.authorize_connection(first, expiry),
        Err(DestinationError::ResolutionApprovalExpired {
            valid_until: expiry,
            current_time: expiry,
        })
    );

    let refreshed = snapshot
        .revalidate([second], &policy, expiry)
        .expect("fresh non-expanding validation may establish a new bounded snapshot");
    assert_eq!(refreshed.approved_at(), expiry);
    assert_eq!(refreshed.valid_until(), Duration::from_secs(18));
    refreshed
        .authorize_connection(second, expiry)
        .expect("the newly validated snapshot has independent current authority");

    assert_eq!(
        snapshot.authorize_connection(second, expiry),
        Err(DestinationError::ResolutionApprovalExpired {
            valid_until: expiry,
            current_time: expiry,
        })
    );
}

#[test]
fn post_expiry_revalidation_still_rejects_address_set_expansion() {
    let approved = ipv4(8, 8, 8, 8);
    let unexpected = ipv4(9, 9, 9, 9);
    let policy = DestinationPolicy::public_web();
    let snapshot = FreshResolutionSnapshot::approve(
        origin(),
        [approved],
        &policy,
        Duration::from_secs(10),
        Duration::from_secs(4),
    )
    .expect("initial bounded freshness authority");

    assert_eq!(
        snapshot.revalidate([approved, unexpected], &policy, Duration::from_secs(14)),
        Err(DestinationError::ResolutionSetExpanded {
            address: unexpected,
        })
    );
}
