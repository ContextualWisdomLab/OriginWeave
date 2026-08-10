use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{ConnectionPlan, NetworkError};

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
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let plan = ConnectionPlan::new(
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
}

#[test]
fn expired_resolution_cannot_create_a_connection_plan() {
    let snapshot = fresh_loopback_snapshot();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let error = ConnectionPlan::new(
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
