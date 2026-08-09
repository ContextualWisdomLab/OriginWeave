#![allow(clippy::expect_used)]

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::{ConnectionPlan, DirectTcpConnection};
use originweave_tls::{
    AlpnRequirement, LeafValidityHorizon, LeafValidityHorizonError, TlsClientPolicy, TlsError,
    TlsHandshakePlan, TrustBundleIdentifier, TrustRootBundle,
};
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
use rustls::{ServerConfig, ServerConnection};

const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;
const TEST_TIMEOUT: Duration = Duration::from_secs(3);

fn server_material() -> (
    Vec<u8>,
    Vec<CertificateDer<'static>>,
    PrivateKeyDer<'static>,
) {
    let mut root_parameters =
        CertificateParams::new(Vec::new()).expect("empty root SAN list must be valid");
    root_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_parameters.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    root_parameters
        .distinguished_name
        .push(DnType::CommonName, "OriginWeave horizon root");
    let root_key = KeyPair::generate().expect("test root key generation");
    let root_certificate = root_parameters
        .self_signed(&root_key)
        .expect("test root certificate generation");
    let issuer = Issuer::new(root_parameters, root_key);

    let mut leaf_parameters =
        CertificateParams::new(vec!["localhost".to_owned()]).expect("localhost SAN is valid");
    leaf_parameters.not_before = rcgen::date_time_ymd(2025, 12, 31);
    leaf_parameters.not_after = rcgen::date_time_ymd(2026, 1, 2);
    leaf_parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    leaf_parameters.use_authority_key_identifier_extension = true;
    let leaf_key = KeyPair::generate().expect("test leaf key generation");
    let leaf_certificate = leaf_parameters
        .signed_by(&leaf_key, &issuer)
        .expect("test leaf certificate generation");
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der()));

    (
        root_certificate.der().to_vec(),
        vec![leaf_certificate.der().clone()],
        private_key,
    )
}

fn complete_server_handshake(
    connection: &mut ServerConnection,
    stream: &mut TcpStream,
) -> Result<(), String> {
    match connection.complete_io(stream) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn spawn_server() -> (SocketAddr, Vec<u8>, thread::JoinHandle<Result<(), String>>) {
    let (root_der, certificate_chain, private_key) = server_material();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 must be supported")
        .with_no_client_auth()
        .with_single_cert(certificate_chain, private_key)
        .expect("test certificate and key must match");
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .expect("loopback TLS listener must bind");
    let socket_address = listener.local_addr().expect("listener address");
    let handle = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let mut connection =
            ServerConnection::new(Arc::new(config)).map_err(|error| error.to_string())?;
        complete_server_handshake(&mut connection, &mut stream)
    });
    (socket_address, root_der, handle)
}

fn direct_connection(origin: &Origin, socket_address: SocketAddr) -> DirectTcpConnection {
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [socket_address.ip()],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution must be approved");
    ConnectionPlan::new(&snapshot, socket_address, Duration::from_secs(2), 1)
        .expect("direct connection plan")
        .connect()
        .expect("loopback TCP connection")
}

fn test_policy() -> TlsClientPolicy {
    TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        Vec::new(),
        AlpnRequirement::Optional,
    )
    .expect("test TLS policy")
}

#[test]
fn authenticated_point_in_time_certificate_can_fail_a_longer_task_horizon() {
    let (socket_address, root_der, server) = spawn_server();
    let origin = Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .expect("test origin must be canonical");
    let trust_bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("horizon_test_roots:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("test trust bundle");
    let authenticated = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle,
        test_policy(),
    )
    .expect("valid TLS plan")
    .authenticate()
    .expect("certificate is valid at the explicit trusted time");

    let evidence = authenticated.evidence();
    let remaining = u64::try_from(evidence.leaf_not_after_unix_seconds())
        .expect("test notAfter is positive")
        .checked_sub(evidence.trusted_time_unix_seconds())
        .expect("test certificate remains valid after trusted time");
    assert_eq!(remaining, 86_400);
    assert_eq!(
        LeafValidityHorizon::new(Duration::from_secs(86_400)).evaluate(
            evidence.trusted_time_unix_seconds(),
            evidence.leaf_not_after_unix_seconds(),
        ),
        Ok(())
    );
    assert_eq!(
        LeafValidityHorizon::new(Duration::from_secs(86_401)).evaluate(
            evidence.trusted_time_unix_seconds(),
            evidence.leaf_not_after_unix_seconds(),
        ),
        Err(LeafValidityHorizonError::InsufficientRemainingValidity {
            remaining_seconds: 86_400,
            minimum_seconds: 86_401,
        })
    );

    assert_eq!(server.join().expect("server thread"), Ok(()));
    drop(authenticated);
}

#[test]
fn tls_policy_enforces_the_configured_task_horizon_before_stream_exposure() {
    let (socket_address, root_der, server) = spawn_server();
    let origin = Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .expect("test origin must be canonical");
    let trust_bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("policy_horizon_roots:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("test trust bundle");
    let policy = test_policy()
        .with_minimum_leaf_validity(Duration::from_secs(86_401))
        .expect("one-day delegated horizon must be within the product policy maximum");

    let result = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle,
        policy,
    )
    .expect("valid TLS plan")
    .authenticate();

    let error = result.err();
    assert!(
        error.is_some(),
        "the authenticated stream must not escape when the leaf expires before the delegated horizon"
    );
    let Some(error) = error else {
        return;
    };
    match error {
        TlsError::InsufficientLeafValidity {
            remaining_seconds,
            minimum_seconds,
        } => {
            assert_eq!(remaining_seconds, 86_400);
            assert_eq!(minimum_seconds, 86_401);
        }
        other => assert_eq!(
            other.to_string(),
            "TLS leaf certificate has insufficient delegated-task validity"
        ),
    }

    assert_eq!(server.join().expect("server thread"), Ok(()));
}
