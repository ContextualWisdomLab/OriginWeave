use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn fresh_loopback_snapshot() -> FreshResolutionSnapshot {
    FreshResolutionSnapshot::approve(
        Origin::parse("http://localhost")
            .unwrap_or_else(|error| panic!("loopback origin: {error:?}")),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .unwrap_or_else(|error| panic!("fresh loopback snapshot: {error}"))
}

#[test]
fn connection_plan_requires_a_current_fresh_resolution_authority() {
    let snapshot = fresh_loopback_snapshot();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap_or_else(|error| panic!("loopback listener: {error}"));
    let socket = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("listener address: {error}"));

    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .unwrap_or_else(|error| panic!("fresh plan must be authorized: {error}"));

    assert_eq!(plan.resolution_approved_at(), Duration::from_secs(10));
    assert_eq!(plan.resolution_valid_until(), Duration::from_secs(15));
    assert_eq!(plan.resolution_authorized_at(), Duration::from_secs(12));

    let connection = plan
        .connect()
        .unwrap_or_else(|error| panic!("fresh plan must connect: {error}"));
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
}

#[test]
fn expired_resolution_cannot_create_a_connection_plan() {
    let snapshot = fresh_loopback_snapshot();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let error = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(15),
        socket,
        Duration::from_secs(1),
        1,
    )
    .expect_err("exclusive freshness deadline must fail closed");

    match error {
        NetworkError::DestinationNotApproved { source, .. } => {
            assert!(source.to_string().contains("expired"));
        }
        other => panic!("unexpected network error: {other}"),
    }
}

#[test]
fn fresh_resolution_still_requires_valid_connection_parameters() {
    let snapshot = fresh_loopback_snapshot();
    let invalid_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let error = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        invalid_socket,
        Duration::from_secs(1),
        1,
    )
    .expect_err("freshness must not bypass connection-plan validation");

    assert!(matches!(error, NetworkError::InvalidPort));
}
