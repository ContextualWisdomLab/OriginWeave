use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn loopback_snapshot() -> Result<FreshResolutionSnapshot, String> {
    let origin = Origin::parse("http://localhost")
        .map_err(|error| format!("default loopback origin is invalid: {error:?}"))?;
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
fn zero_socket_port_remains_invalid_input_before_origin_mismatch() -> Result<(), String> {
    let snapshot = loopback_snapshot()?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    );

    assert!(matches!(result, Err(NetworkError::InvalidPort)));
    Ok(())
}

#[test]
fn malformed_connection_settings_fail_before_origin_port_authority() -> Result<(), String> {
    let snapshot = loopback_snapshot()?;
    let wrong_port_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 81);

    let invalid_timeout = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        wrong_port_socket,
        Duration::ZERO,
        1,
    );
    assert!(matches!(
        invalid_timeout,
        Err(NetworkError::InvalidConnectTimeout { .. })
    ));

    let invalid_attempt_count = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        wrong_port_socket,
        Duration::from_secs(1),
        0,
    );
    assert!(matches!(
        invalid_attempt_count,
        Err(NetworkError::InvalidAttemptCount { .. })
    ));
    Ok(())
}

#[test]
fn malformed_connection_settings_fail_before_resolution_membership() -> Result<(), String> {
    let snapshot = loopback_snapshot()?;
    let unapproved_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 80);

    let invalid_timeout = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        unapproved_socket,
        Duration::ZERO,
        1,
    );
    assert!(matches!(
        invalid_timeout,
        Err(NetworkError::InvalidConnectTimeout { .. })
    ));

    let invalid_attempt_count = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        unapproved_socket,
        Duration::from_secs(1),
        0,
    );
    assert!(matches!(
        invalid_attempt_count,
        Err(NetworkError::InvalidAttemptCount { .. })
    ));
    Ok(())
}
