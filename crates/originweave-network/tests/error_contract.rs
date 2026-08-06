#![allow(clippy::expect_used)]

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::{ConnectionPlan, MAX_CONNECT_TIMEOUT, NetworkError};

fn loopback_snapshot() -> ResolutionSnapshot {
    ResolutionSnapshot::approve(
        Origin::parse("http://localhost").expect("loopback origin"),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution")
}

#[test]
fn validation_errors_are_deterministic_and_have_no_source() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
    let errors = [
        ConnectionPlan::new(
            &loopback_snapshot(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            Duration::from_secs(1),
            1,
        )
        .expect_err("zero port"),
        ConnectionPlan::new(&loopback_snapshot(), socket, Duration::ZERO, 1)
            .expect_err("zero timeout"),
        ConnectionPlan::new(
            &loopback_snapshot(),
            socket,
            MAX_CONNECT_TIMEOUT + Duration::from_nanos(1),
            1,
        )
        .expect_err("excessive timeout"),
        ConnectionPlan::new(&loopback_snapshot(), socket, Duration::from_secs(1), 0)
            .expect_err("zero attempts"),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
        assert!(error.attempt_count().is_none());
    }
}

#[test]
fn destination_denial_is_preserved_as_the_error_source() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 443);
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        socket,
        Duration::from_secs(1),
        1,
    )
    .expect_err("address absent from snapshot");

    assert!(matches!(
        error,
        NetworkError::DestinationNotApproved { .. }
    ));
    assert!(error.source().is_some());
    assert!(error.to_string().contains("not approved"));
}

#[test]
fn noncanonical_address_error_has_no_source() {
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1);
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        SocketAddr::new(IpAddr::V6(mapped), 443),
        Duration::from_secs(1),
        1,
    )
    .expect_err("mapped address is not canonical");

    assert!(matches!(
        error,
        NetworkError::NonCanonicalSocketAddress { .. }
    ));
    assert!(error.source().is_none());
    assert!(error.to_string().contains("canonical"));
}
