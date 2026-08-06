#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::ConnectionPlan;

#[test]
fn approved_loopback_socket_becomes_the_exact_peer() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind listener");
    let socket = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        stream.write_all(b"ok").expect("write response");
    });

    let origin = Origin::parse("http://localhost").expect("loopback origin");
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("approve loopback");
    let connection = ConnectionPlan::new(&snapshot, socket, Duration::from_secs(1), 1)
        .expect("build plan")
        .connect()
        .expect("connect exact peer");

    assert_eq!(connection.stream().peer_addr().expect("peer"), socket);
    assert_eq!(connection.evidence().origin(), &origin);
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
    assert_eq!(
        connection.evidence().address_class(),
        AddressClass::Loopback
    );
    assert_eq!(connection.evidence().attempt_number(), 1);
    assert_eq!(
        connection.evidence().connect_timeout(),
        Duration::from_secs(1)
    );

    let (mut stream, evidence) = connection.into_parts();
    let mut body = [0_u8; 2];
    stream.read_exact(&mut body).expect("read response");
    assert_eq!(&body, b"ok");
    assert_eq!(evidence.observed_peer(), socket);
    server.join().expect("server thread");
}

#[test]
fn refused_loopback_connection_stops_at_the_attempt_bound() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("reserve port");
    let socket = listener.local_addr().expect("reserved address");
    drop(listener);

    let snapshot = ResolutionSnapshot::approve(
        Origin::parse("http://localhost").expect("loopback origin"),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("approve loopback");
    let error = ConnectionPlan::new(&snapshot, socket, Duration::from_secs(1), 3)
        .expect("build plan")
        .connect()
        .expect_err("closed port must fail");

    assert_eq!(error.attempt_count(), Some(3));
}
