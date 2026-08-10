#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationError, DestinationPolicy, FreshResolutionSnapshot,
    MAX_RESOLUTION_VALIDITY,
};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

#[test]
fn fresh_resolution_authority_is_half_open_and_bound_to_pinned_addresses() {
    let approved_at = Duration::from_secs(100);
    let validity = Duration::from_secs(5);
    let target = origin("https://example.com");
    let approved = ipv4(8, 8, 8, 8);
    let snapshot = FreshResolutionSnapshot::approve(
        target.clone(),
        [approved],
        &DestinationPolicy::public_web(),
        approved_at,
        validity,
    )
    .expect("bounded fresh resolution");

    assert_eq!(snapshot.origin(), &target);
    assert_eq!(snapshot.approved_at(), approved_at);
    assert_eq!(snapshot.validity(), validity);
    assert_eq!(snapshot.valid_until(), Duration::from_secs(105));

    let evidence = snapshot
        .authorize_connection(approved, approved_at)
        .expect("authority begins at approval time");
    let connection = evidence.connection_evidence();
    assert_eq!(connection.origin(), &target);
    assert_eq!(connection.requested_address(), approved);
    assert_eq!(connection.canonical_address(), approved);
    assert_eq!(connection.address_class(), AddressClass::Public);
    assert_eq!(evidence.resolution_approved_at(), approved_at);
    assert_eq!(evidence.resolution_valid_until(), Duration::from_secs(105));
    assert_eq!(evidence.authorized_at(), approved_at);

    snapshot
        .authorize_connection(approved, Duration::from_secs(104))
        .expect("authority remains valid before the exclusive deadline");

    assert_eq!(
        snapshot.authorize_connection(approved, Duration::from_secs(99)),
        Err(DestinationError::ResolutionUseBeforeApproval {
            approved_at,
            current_time: Duration::from_secs(99),
        })
    );
    assert_eq!(
        snapshot.authorize_connection(approved, Duration::from_secs(105)),
        Err(DestinationError::ResolutionApprovalExpired {
            valid_until: Duration::from_secs(105),
            current_time: Duration::from_secs(105),
        })
    );
    assert_eq!(
        snapshot.authorize_connection(ipv4(9, 9, 9, 9), approved_at),
        Err(DestinationError::UnapprovedConnectionAddress {
            address: ipv4(9, 9, 9, 9),
        })
    );
}

#[test]
fn fresh_resolution_rejects_invalid_or_overflowing_validity() {
    let target = origin("https://example.com");
    let address = ipv4(8, 8, 8, 8);
    let policy = DestinationPolicy::public_web();

    for validity in [
        Duration::ZERO,
        MAX_RESOLUTION_VALIDITY + Duration::from_nanos(1),
    ] {
        assert_eq!(
            FreshResolutionSnapshot::approve(
                target.clone(),
                [address],
                &policy,
                Duration::from_secs(1),
                validity,
            ),
            Err(DestinationError::InvalidResolutionValidity {
                validity,
                maximum_validity: MAX_RESOLUTION_VALIDITY,
            })
        );
    }

    assert_eq!(
        FreshResolutionSnapshot::approve(
            target,
            [address],
            &policy,
            Duration::MAX,
            Duration::from_nanos(1),
        ),
        Err(DestinationError::ResolutionValidityOverflow {
            approved_at: Duration::MAX,
            validity: Duration::from_nanos(1),
        })
    );
}

#[test]
fn fresh_resolution_rejects_denied_addresses_before_granting_time_authority() {
    let target = origin("https://example.com");
    let denied = ipv4(127, 0, 0, 1);
    let public = ipv4(8, 8, 8, 8);
    let policy = DestinationPolicy::public_web();
    let expected = Err(DestinationError::AddressClassDenied {
        address: denied,
        address_class: AddressClass::Loopback,
    });

    assert_eq!(
        FreshResolutionSnapshot::approve(
            target.clone(),
            [denied],
            &policy,
            Duration::from_secs(1),
            Duration::from_secs(1),
        ),
        expected.clone()
    );
    assert_eq!(
        FreshResolutionSnapshot::approve(
            target,
            [denied, public],
            &policy,
            Duration::from_secs(1),
            Duration::from_secs(1),
        ),
        expected
    );
}

#[test]
fn fresh_revalidation_preserves_the_budget_and_resets_approval_time() {
    let first = ipv4(8, 8, 8, 8);
    let second = ipv4(1, 1, 1, 1);
    let policy = DestinationPolicy::public_web();
    let snapshot = FreshResolutionSnapshot::approve(
        origin("https://example.com"),
        [first, second],
        &policy,
        Duration::from_secs(10),
        Duration::from_secs(4),
    )
    .expect("initial fresh resolution");

    let refreshed = snapshot
        .revalidate([second], &policy, Duration::from_secs(13))
        .expect("a fresh non-expanding answer renews the bounded window");
    assert_eq!(
        refreshed.addresses(),
        &std::collections::BTreeSet::from([second])
    );
    assert_eq!(refreshed.approved_at(), Duration::from_secs(13));
    assert_eq!(refreshed.validity(), Duration::from_secs(4));
    assert_eq!(refreshed.valid_until(), Duration::from_secs(17));
    refreshed
        .authorize_connection(second, Duration::from_secs(16))
        .expect("refreshed authority is usable before its new deadline");

    assert_eq!(
        snapshot.revalidate([second], &policy, Duration::from_secs(9)),
        Err(DestinationError::ResolutionUseBeforeApproval {
            approved_at: Duration::from_secs(10),
            current_time: Duration::from_secs(9),
        })
    );
    assert_eq!(
        snapshot.revalidate([first, ipv4(9, 9, 9, 9)], &policy, Duration::from_secs(11)),
        Err(DestinationError::ResolutionSetExpanded {
            address: ipv4(9, 9, 9, 9),
        })
    );
}

#[test]
fn freshness_errors_have_deterministic_bounded_messages() {
    let invalid = DestinationError::InvalidResolutionValidity {
        validity: Duration::ZERO,
        maximum_validity: MAX_RESOLUTION_VALIDITY,
    };
    assert_eq!(
        invalid.to_string(),
        "resolution validity 0ns is outside 1ns..=30s"
    );

    let overflow = DestinationError::ResolutionValidityOverflow {
        approved_at: Duration::MAX,
        validity: Duration::from_nanos(1),
    };
    assert!(overflow.to_string().contains("overflows approval time"));

    let before = DestinationError::ResolutionUseBeforeApproval {
        approved_at: Duration::from_secs(10),
        current_time: Duration::from_secs(9),
    };
    assert_eq!(
        before.to_string(),
        "resolution use time 9s precedes approval time 10s"
    );

    let expired = DestinationError::ResolutionApprovalExpired {
        valid_until: Duration::from_secs(15),
        current_time: Duration::from_secs(15),
    };
    assert_eq!(
        expired.to_string(),
        "resolution approval expired at 15s; current time is 15s"
    );
}
