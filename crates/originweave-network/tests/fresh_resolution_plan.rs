use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn fresh_loopback_snapshot() -> Result<FreshResolutionSnapshot, String> {
    let origin = Origin::parse("http://localhost")
        .map_err(|error| format!("loopback origin fixture is invalid: {error:?}"))?;
    FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh loopback snapshot is invalid: {error}"))
}

#[test]
fn connection_plan_requires_a_current_fresh_resolution_authority() -> Result<(), String> {
    let snapshot = fresh_loopback_snapshot()?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| format!("bind loopback listener: {error}"))?;
    let socket = listener
        .local_addr()
        .map_err(|error| format!("read loopback listener address: {error}"))?;

    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize fresh connection plan: {error}"))?;

    assert_eq!(plan.resolution_approved_at(), Duration::from_secs(10));
    assert_eq!(plan.resolution_valid_until(), Duration::from_secs(15));
    assert_eq!(plan.resolution_authorized_at(), Duration::from_secs(12));

    let connection = plan
        .connect(Duration::from_secs(12))
        .map_err(|error| format!("connect fresh loopback plan: {error}"))?;
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
    Ok(())
}

#[test]
fn expired_resolution_cannot_create_a_connection_plan() -> Result<(), String> {
    let snapshot = fresh_loopback_snapshot()?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(15),
        socket,
        Duration::from_secs(1),
        1,
    );

    assert!(matches!(
        result,
        Err(NetworkError::DestinationNotApproved { ref source, .. })
            if source.to_string().contains("expired")
    ));
    Ok(())
}

#[test]
fn connection_plan_rechecks_freshness_at_socket_use_time() -> Result<(), String> {
    let snapshot = fresh_loopback_snapshot()?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize fresh connection plan: {error}"))?;

    let result = plan.connect(Duration::from_secs(15));

    assert!(matches!(
        result,
        Err(NetworkError::DestinationNotApproved { ref source, .. })
            if source.to_string().contains("expired")
    ));
    Ok(())
}

#[test]
fn fresh_resolution_still_requires_valid_connection_parameters() -> Result<(), String> {
    let snapshot = fresh_loopback_snapshot()?;
    let invalid_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        invalid_socket,
        Duration::from_secs(1),
        1,
    );

    assert!(matches!(result, Err(NetworkError::InvalidPort)));
    Ok(())
}
