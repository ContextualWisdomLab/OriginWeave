#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::FreshConnectionPlan;
use originweave_tls::{
    AlpnRequirement, TlsClientPolicy, TlsError, TlsHandshakePlan, TrustBundleIdentifier,
    TrustRootBundle,
};
use rustls::pki_types::UnixTime;

const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;
const HARD_DEADLINE: Duration = Duration::from_nanos(1);
const RESOLUTION_APPROVED_AT: Duration = Duration::from_secs(10);
const RESOLUTION_VALIDITY: Duration = Duration::from_secs(5);
const RESOLUTION_AUTHORIZED_AT: Duration = Duration::from_secs(12);

fn direct_connection(
    origin: &Origin,
    socket_address: SocketAddr,
) -> originweave_network::DirectTcpConnection {
    let snapshot = FreshResolutionSnapshot::approve(
        origin.clone(),
        [socket_address.ip()],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        RESOLUTION_APPROVED_AT,
        RESOLUTION_VALIDITY,
    )
    .expect("managed loopback resolution must be approved");
    FreshConnectionPlan::new(
        &snapshot,
        RESOLUTION_AUTHORIZED_AT,
        socket_address,
        Duration::from_secs(1),
        1,
    )
    .expect("fresh direct connection plan")
    .connect_at(RESOLUTION_AUTHORIZED_AT)
    .expect("loopback TCP connection")
}

#[test]
fn an_elapsed_total_deadline_rejects_tls_before_further_network_io() {
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .expect("loopback listener must bind");
    let socket_address = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (_stream, _peer) = listener.accept().expect("client connection");
        thread::sleep(Duration::from_millis(20));
    });

    let origin = Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .expect("canonical HTTPS origin");
    let root_der = rcgen::generate_simple_self_signed(vec!["root.example".to_owned()])
        .expect("test trust root")
        .cert
        .der()
        .to_vec();
    let trust_roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("deadline_test_roots:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("trust root bundle");
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        HARD_DEADLINE,
        Vec::new(),
        AlpnRequirement::Optional,
    )
    .expect("one-nanosecond policy remains syntactically valid");

    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_roots,
        policy,
    )
    .expect("deadline test plan")
    .authenticate()
    .expect_err("elapsed total deadline must fail closed");

    assert!(matches!(
        error,
        TlsError::HandshakeTimedOut { timeout } if timeout == HARD_DEADLINE
    ));
    server.join().expect("server thread");
}
