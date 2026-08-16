use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{FreshConnectionPlan, NetworkError};

#[test]
fn zero_socket_port_remains_invalid_input_before_origin_mismatch() -> Result<(), String> {
    let origin = Origin::parse("http://localhost")
        .map_err(|error| format!("default loopback origin is invalid: {error:?}"))?;
    let snapshot = FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh loopback snapshot is invalid: {error}"))?;
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
