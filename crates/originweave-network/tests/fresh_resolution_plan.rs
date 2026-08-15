use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationError, DestinationPolicy, FreshResolutionSnapshot,
};
use originweave_network::{FreshConnectionPlan, NetworkError};

fn fresh_loopback_snapshot_for_port(port: u16) -> Result<FreshResolutionSnapshot, String> {
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

fn fresh_default_loopback_snapshot() -> Result<FreshResolutionSnapshot, String> {
    let origin = Origin::parse("http://localhost")
        .map_err(|error| format!("default loopback origin fixture is invalid: {error:?}"))?;
    FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh default loopback snapshot is invalid: {error}"))
}

#[test]
fn connection_plan_requires_a_current_fresh_resolution_authority() -> Result<(), String> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| format!("bind loopback listener: {error}"))?;
    let socket = listener
        .local_addr()
        .map_err(|error| format!("read loopback listener address: {error}"))?;
    let snapshot = fresh_loopback_snapshot_for_port(socket.port())?;

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
        .connect_at(Duration::from_secs(12))
        .map_err(|error| format!("connect fresh loopback plan: {error}"))?;
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
    Ok(())
}

#[test]
fn expired_resolution_cannot_create_a_connection_plan() -> Result<(), String> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    let snapshot = fresh_loopback_snapshot_for_port(socket.port())?;

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
    let snapshot = fresh_loopback_snapshot_for_port(socket.port())?;
    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize fresh connection plan: {error}"))?;

    let result = plan.connect_at(Duration::from_secs(15));
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
    let snapshot = fresh_loopback_snapshot_for_port(socket.port())?;
    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize fresh connection plan: {error}"))?;

    let result = plan.connect_at(Duration::from_secs(11));
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
fn compatibility_connect_path_expires_from_real_monotonic_elapsed_time() -> Result<(), String> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9);
    let origin = Origin::parse("http://localhost:9")
        .map_err(|error| format!("loopback origin fixture is invalid: {error:?}"))?;
    let snapshot = FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_millis(1),
    )
    .map_err(|error| format!("short-lived snapshot is invalid: {error}"))?;
    let plan = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(10),
        socket,
        Duration::from_secs(1),
        1,
    )
    .map_err(|error| format!("authorize short-lived connection plan: {error}"))?;

    thread::sleep(Duration::from_millis(5));
    let result = plan.connect();
    assert!(matches!(
        result,
        Err(NetworkError::DestinationNotApproved {
            source: DestinationError::ResolutionApprovalExpired { .. },
            ..
        })
    ));
    Ok(())
}

#[test]
fn fresh_resolution_still_requires_valid_connection_parameters() -> Result<(), String> {
    let snapshot = fresh_default_loopback_snapshot()?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::ZERO,
        1,
    );

    assert!(matches!(
        result,
        Err(NetworkError::InvalidConnectTimeout {
            connect_timeout,
            ..
        }) if connect_timeout == Duration::ZERO
    ));
    Ok(())
}

#[test]
fn connection_plan_rejects_socket_port_that_changes_default_origin() -> Result<(), String> {
    let snapshot = fresh_default_loopback_snapshot()?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    );
    let error = match result {
        Err(error) => error,
        Ok(_plan) => return Err("origin-port drift unexpectedly produced a connection plan".into()),
    };

    assert!(matches!(
        &error,
        NetworkError::OriginPortMismatch {
            requested_port,
            expected_port,
        } if *requested_port == 8080 && *expected_port == 80
    ));
    assert_eq!(
        error.to_string(),
        "socket port 8080 does not match canonical origin port 80"
    );
    assert!(error.source().is_none());
    assert_eq!(error.attempt_count(), None);
    Ok(())
}

#[test]
fn connection_plan_enforces_default_https_port_for_ipv6_origin() -> Result<(), String> {
    let origin = Origin::parse("https://[::1]")
        .map_err(|error| format!("default HTTPS IPv6 origin fixture is invalid: {error:?}"))?;
    let snapshot = FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V6(Ipv6Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh HTTPS IPv6 snapshot is invalid: {error}"))?;

    let authorized = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 443),
        Duration::from_secs(1),
        1,
    );
    assert!(authorized.is_ok());

    let wrong_port = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 444),
        Duration::from_secs(1),
        1,
    );
    assert!(matches!(
        wrong_port,
        Err(NetworkError::OriginPortMismatch {
            requested_port: 444,
            expected_port: 443,
        })
    ));
    Ok(())
}

#[test]
fn connection_plan_rejects_socket_port_that_changes_explicit_origin() -> Result<(), String> {
    let origin = Origin::parse("http://localhost:8080")
        .map_err(|error| format!("explicit-port origin fixture is invalid: {error:?}"))?;
    let snapshot = FreshResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        Duration::from_secs(10),
        Duration::from_secs(5),
    )
    .map_err(|error| format!("fresh explicit-port snapshot is invalid: {error}"))?;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);

    let result = FreshConnectionPlan::new(
        &snapshot,
        Duration::from_secs(12),
        socket,
        Duration::from_secs(1),
        1,
    );

    assert!(matches!(
        result,
        Err(NetworkError::OriginPortMismatch {
            requested_port: 8081,
            expected_port: 8080,
        })
    ));
    Ok(())
}
