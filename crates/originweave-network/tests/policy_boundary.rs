#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::{
    ConnectionPlan, MAX_CONNECTION_ATTEMPTS, MAX_CONNECT_TIMEOUT, NetworkError,
};

fn loopback_snapshot() -> ResolutionSnapshot {
    ResolutionSnapshot::approve(
        Origin::parse("http://localhost").expect("loopback origin"),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution")
}

#[test]
fn plan_rejects_port_zero_before_network_io() {
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Duration::from_secs(1),
        1,
    )
    .expect_err("port zero must fail");
    assert!(matches!(error, NetworkError::InvalidPort));
}

#[test]
fn plan_rejects_zero_and_excessive_timeouts() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
    for timeout in [
        Duration::ZERO,
        MAX_CONNECT_TIMEOUT + Duration::from_nanos(1),
    ] {
        assert!(matches!(
            ConnectionPlan::new(&loopback_snapshot(), socket, timeout, 1),
            Err(NetworkError::InvalidConnectTimeout { .. })
        ));
    }
}

#[test]
fn plan_rejects_zero_and_excessive_attempt_counts() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
    for attempts in [0, MAX_CONNECTION_ATTEMPTS + 1] {
        assert!(matches!(
            ConnectionPlan::new(
                &loopback_snapshot(),
                socket,
                Duration::from_secs(1),
                attempts,
            ),
            Err(NetworkError::InvalidAttemptCount { .. })
        ));
    }
}

#[test]
fn public_policy_rejects_loopback_before_plan_creation() {
    let origin = Origin::parse("https://example.com").expect("public origin");
    let resolution = ResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::public_web(),
    );
    assert!(resolution.is_err());
}

#[test]
fn mapped_loopback_is_not_accepted_as_a_canonical_socket() {
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1);
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        SocketAddr::new(IpAddr::V6(mapped), 80),
        Duration::from_secs(1),
        1,
    )
    .expect_err("mapped form must be rejected");
    assert!(matches!(
        error,
        NetworkError::NonCanonicalSocketAddress { .. }
    ));
}
