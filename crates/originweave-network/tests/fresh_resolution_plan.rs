use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn fresh_loopback_snapshot() -> Result<FreshResolutionSnapshot, Box<dyn Error>> {
    Ok(FreshResolutionSnapshot::approve(
        Origin::parse("http://localhost")?,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )?)
}

#[test]
fn connection_plan_requires_a_current_fresh_resolution_authority() -> Result<(), Box<dyn Error>> {
    let snapshot = fresh_loopback_snapshot()?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let socket = listener.local_addr()?;

    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )?;

    assert_eq!(plan.resolution_approved_at(), Duration::from_secs(10));
    assert_eq!(plan.resolution_valid_until(), Duration::from_secs(15));
    assert_eq!(plan.resolution_authorized_at(), Duration::from_secs(12));

    let connection = plan.connect()?;
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
    Ok(())
}

#[test]
fn expired_resolution_cannot_create_a_connection_plan() -> Result<(), Box<dyn Error>> {
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
fn fresh_resolution_still_requires_valid_connection_parameters() -> Result<(), Box<dyn Error>> {
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
