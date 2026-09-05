use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationError, DestinationPolicy, FreshResolutionSnapshot,
};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn fresh_loopback_snapshot(port: u16) -> Result<FreshResolutionSnapshot, String> {
    let origin = Origin::parse(&format!("http://localhost:{port}"))
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
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| format!("bind loopback listener: {error}"))?;
    let socket = listener
        .local_addr()
        .map_err(|error| format!("read loopback listener address: {error}"))?;
    let snapshot = fresh_loopback_snapshot(socket.port())?;

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
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    let snapshot = fresh_loopback_snapshot(socket.port())?;

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(15),
        socket,
        Duration::from_secs(1),
        1,
    );

    assert!(matches!(
        result,
        Err(NetworkError::DestinationNotApproved {
            source: DestinationError::ResolutionApprovalExpired {
                valid_until,
                current_time,
            },
            ..
        }) if valid_until == Duration::from_secs(15)
            && current_time == Duration::from_secs(15)
    ));
    Ok(())
}

#[test]
fn plan_must_still_be_fresh_at_actual_socket_use() -> Result<(), String> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9);
    let snapshot = fresh_loopback_snapshot(socket.port())?;
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
        Err(NetworkError::DestinationNotApproved {
            source: DestinationError::ResolutionApprovalExpired {
                valid_until,
                current_time,
            },
            ..
        }) if valid_until == Duration::from_secs(15)
            && current_time == Duration::from_secs(15)
    ));
    Ok(())
}

#[test]
fn socket_use_time_cannot_regress_before_plan_authorization() -> Result<(), String> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9);
    let snapshot = fresh_loopback_snapshot(socket.port())?;
    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize fresh connection plan: {error}"))?;

    let result = plan.connect(Duration::from_secs(11));
    assert!(matches!(
        result,
        Err(NetworkError::DestinationNotApproved {
            source: DestinationError::ResolutionUseBeforeApproval {
                approved_at,
                current_time,
            },
            ..
        }) if approved_at == Duration::from_secs(12)
            && current_time == Duration::from_secs(11)
    ));
    Ok(())
}

#[test]
fn fresh_resolution_still_requires_valid_connection_parameters() -> Result<(), String> {
    let invalid_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    let snapshot = fresh_loopback_snapshot(80)?;

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

#[test]
fn connection_port_must_match_the_approved_logical_origin() -> Result<(), String> {
    let origin = Origin::parse("http://localhost:8080")
        .map_err(|error| format!("loopback origin fixture is invalid: {error:?}"))?;
    let snapshot = FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh loopback snapshot is invalid: {error}"))?;
    let mismatched_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);

    assert!(matches!(
        FreshConnectionPlan::new(
            &snapshot,
            Duration::from_secs(12),
            mismatched_socket,
            Duration::from_secs(1),
            1,
        ),
        Err(NetworkError::OriginPortMismatch {
            socket_port: 8081,
            origin_port: 8080,
        })
    ));
    Ok(())
}

#[test]
fn default_origin_ports_are_enforced_for_http_and_https() -> Result<(), String> {
    let policy = DestinationPolicy::from_allowed_classes([AddressClass::Loopback]);
    let cases = [
        ("http://localhost", 80_u16, 81_u16),
        ("https://localhost", 443_u16, 444_u16),
    ];

    for (origin_text, expected_port, mismatched_port) in cases {
        let origin = Origin::parse(origin_text)
            .map_err(|error| format!("default-port origin fixture is invalid: {error:?}"))?;
        let snapshot = FreshResolutionSnapshot::approve(
            origin,
            [IpAddr::V4(Ipv4Addr::LOCALHOST)],
            &policy,
            Duration::from_secs(10),
            Duration::from_secs(5),
        )
        .map_err(|error| format!("fresh default-port snapshot is invalid: {error}"))?;
        let matching_socket =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), expected_port);
        let mismatched_socket =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), mismatched_port);

        assert!(FreshConnectionPlan::new(
            &snapshot,
            Duration::from_secs(12),
            matching_socket,
            Duration::from_secs(1),
            1,
        )
        .is_ok());
        assert!(matches!(
            FreshConnectionPlan::new(
                &snapshot,
                Duration::from_secs(12),
                mismatched_socket,
                Duration::from_secs(1),
                1,
            ),
            Err(NetworkError::OriginPortMismatch {
                socket_port,
                origin_port,
            }) if socket_port == mismatched_port && origin_port == expected_port
        ));
    }
    Ok(())
}
