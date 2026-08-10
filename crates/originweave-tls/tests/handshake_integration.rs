#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, FreshResolutionSnapshot};
use originweave_network::{DirectTcpConnection, FreshConnectionPlan};
use originweave_tls::TlsReferenceIdentity;
use originweave_tls::{
    AlpnRequirement, NegotiatedAlpn, RevocationStatus, TlsClientPolicy, TlsError, TlsHandshakePlan,
    TlsProtocolVersion, TrustBundleIdentifier, TrustRootBundle,
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
use rustls::{ServerConfig, ServerConnection, SupportedProtocolVersion};

const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;
const TEST_TIMEOUT: Duration = Duration::from_secs(3);
const RESOLUTION_APPROVED_AT: Duration = Duration::from_secs(10);
const RESOLUTION_VALIDITY: Duration = Duration::from_secs(5);
const RESOLUTION_AUTHORIZED_AT: Duration = Duration::from_secs(12);

type ServerResult = Result<Option<Vec<u8>>, String>;

struct CertificateMaterial {
    root_der: Vec<u8>,
    certificate_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

fn certificate_authority(common_name: &str) -> (Vec<u8>, Issuer<'static, KeyPair>) {
    let mut parameters =
        CertificateParams::new(Vec::new()).expect("empty CA SAN list must be valid");
    parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    parameters.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    parameters
        .distinguished_name
        .push(DnType::CommonName, common_name);
    let key_pair = KeyPair::generate().expect("test CA key generation");
    let certificate = parameters
        .self_signed(&key_pair)
        .expect("test CA certificate generation");
    (
        certificate.der().to_vec(),
        Issuer::new(parameters, key_pair),
    )
}

fn certificate_material(
    subject_alt_names: Vec<String>,
    common_name: Option<&str>,
    not_before: (i32, u8, u8),
    not_after: (i32, u8, u8),
) -> CertificateMaterial {
    let (root_der, issuer) = certificate_authority("OriginWeave test root");
    let mut parameters =
        CertificateParams::new(subject_alt_names).expect("test SAN values must be valid");
    parameters.not_before = rcgen::date_time_ymd(not_before.0, not_before.1, not_before.2);
    parameters.not_after = rcgen::date_time_ymd(not_after.0, not_after.1, not_after.2);
    parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    parameters.use_authority_key_identifier_extension = true;
    if let Some(name) = common_name {
        parameters.distinguished_name.push(DnType::CommonName, name);
    }
    let key_pair = KeyPair::generate().expect("test leaf key generation");
    let certificate: Certificate = parameters
        .signed_by(&key_pair, &issuer)
        .expect("test leaf certificate generation");
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der()));
    CertificateMaterial {
        root_der,
        certificate_chain: vec![certificate.der().clone()],
        private_key,
    }
}

fn server_config(
    material: CertificateMaterial,
    alpn_protocols: &[&[u8]],
    protocol_versions: &[&'static SupportedProtocolVersion],
) -> (Vec<u8>, Arc<ServerConfig>) {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(protocol_versions)
        .expect("test protocol versions must be supported");
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(material.certificate_chain, material.private_key)
        .expect("test certificate and key must match");
    config.alpn_protocols = alpn_protocols
        .iter()
        .map(|protocol| protocol.to_vec())
        .collect();
    config.max_early_data_size = 0;
    config.key_log = Arc::new(rustls::NoKeyLog {});
    (material.root_der, Arc::new(config))
}

fn spawn_server(
    bind_address: IpAddr,
    config: Arc<ServerConfig>,
) -> (SocketAddr, JoinHandle<ServerResult>) {
    let listener = TcpListener::bind(SocketAddr::new(bind_address, 0))
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
        let mut connection = ServerConnection::new(config).map_err(|error| error.to_string())?;
        connection
            .complete_io(&mut stream)
            .map_err(|error| error.to_string())?;
        Ok(connection.alpn_protocol().map(<[u8]>::to_vec))
    });
    (socket_address, handle)
}

fn origin_for(host: &str, socket_address: SocketAddr) -> Origin {
    Origin::parse(&format!("https://{host}:{}", socket_address.port()))
        .expect("test origin must be canonical")
}

fn direct_connection(origin: &Origin, socket_address: SocketAddr) -> DirectTcpConnection {
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
        Duration::from_secs(2),
        1,
    )
    .expect("fresh direct connection plan")
    .connect()
    .expect("loopback TCP connection")
}

fn trust_bundle(root_der: Vec<u8>, identifier: &str) -> TrustRootBundle {
    TrustRootBundle::new(
        TrustBundleIdentifier::parse(identifier).expect("test trust identifier"),
        vec![root_der],
    )
    .expect("test root bundle")
}

fn client_policy(alpn_protocols: &[&[u8]], requirement: AlpnRequirement) -> TlsClientPolicy {
    TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        alpn_protocols
            .iter()
            .map(|protocol| protocol.to_vec())
            .collect::<Vec<_>>(),
        requirement,
    )
    .expect("test client policy")
}

fn valid_material(
    subject_alt_names: Vec<String>,
    common_name: Option<&str>,
) -> CertificateMaterial {
    certificate_material(subject_alt_names, common_name, (2025, 1, 1), (2030, 1, 1))
}

#[test]
fn dns_identity_authenticates_the_exact_verified_tcp_stream() {
    let material = valid_material(vec!["localhost".to_owned()], Some("irrelevant.example"));
    let (root_der, config) = server_config(
        material,
        &[b"h2"],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let connection = direct_connection(&origin, socket_address);
    let mut authenticated = TlsHandshakePlan::new(
        origin.clone(),
        connection,
        trust_bundle(root_der, "local_test_roots:v1"),
        client_policy(&[b"h2", b"http/1.1"], AlpnRequirement::Required),
    )
    .expect("valid TLS plan")
    .authenticate()
    .expect("trusted DNS identity must authenticate");

    let evidence = authenticated.evidence().clone();
    assert_eq!(evidence.origin(), &origin);
    assert_eq!(evidence.requested_peer(), socket_address);
    assert_eq!(evidence.observed_peer(), socket_address);
    assert_eq!(
        evidence.negotiated_alpn(),
        &NegotiatedAlpn::Protocol(b"h2".to_vec())
    );
    assert!(matches!(
        evidence.protocol_version(),
        TlsProtocolVersion::Tls12 | TlsProtocolVersion::Tls13
    ));
    assert_eq!(
        evidence.revocation_status(),
        RevocationStatus::NotConfigured
    );
    assert_eq!(evidence.presented_certificate_count(), 1);
    assert!(evidence.presented_certificate_bytes() > 0);
    assert_eq!(evidence.trust_root_count(), 1);
    assert!(evidence.trust_root_bytes() > 0);
    assert_eq!(evidence.trusted_time_unix_seconds(), TRUSTED_TIME_SECONDS);
    assert!(evidence.handshake_duration() <= evidence.handshake_timeout());
    assert_eq!(
        evidence.reference_identity(),
        &TlsReferenceIdentity::Dns("localhost".to_owned())
    );
    assert_ne!(evidence.cipher_suite_identifier(), [0_u8; 2]);
    assert!(!evidence.cipher_suite_label().is_empty());
    assert_eq!(evidence.presented_certificate_hashes().len(), 1);
    assert!(
        evidence
            .presented_certificate_hashes()
            .iter()
            .all(|identifier| identifier.starts_with("sha256:"))
    );
    assert!(evidence.leaf_not_before_unix_seconds() < evidence.leaf_not_after_unix_seconds());
    assert_eq!(
        evidence.trust_bundle_identifier().as_str(),
        "local_test_roots:v1"
    );
    for identifier in [
        evidence.leaf_certificate_hash(),
        evidence.leaf_spki_hash(),
        evidence.trust_bundle_hash(),
    ] {
        assert!(identifier.starts_with("sha256:"));
        assert_eq!(identifier.len(), 71);
    }
    assert_eq!(
        authenticated
            .stream()
            .sock
            .peer_addr()
            .expect("authenticated peer"),
        socket_address
    );
    let _mutable_stream = authenticated.stream_mut();
    let (_stream, consumed_evidence) = authenticated.into_parts();
    assert_eq!(consumed_evidence, evidence);
    assert_eq!(
        server.join().expect("server thread"),
        Ok(Some(b"h2".to_vec()))
    );
}

#[test]
fn optional_alpn_records_explicit_absence() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let authenticated = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "optional_alpn:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect("optional ALPN plan")
    .authenticate()
    .expect("ALPN absence is permitted");
    assert_eq!(
        authenticated.evidence().negotiated_alpn(),
        &NegotiatedAlpn::Absent
    );
    assert_eq!(server.join().expect("server thread"), Ok(None));
}

#[test]
fn required_alpn_rejects_explicit_server_absence() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "required_absence:v1"),
        client_policy(&[b"h2"], AlpnRequirement::Required),
    )
    .expect("required ALPN plan")
    .authenticate()
    .expect_err("explicit ALPN absence must fail policy");
    assert!(matches!(error, TlsError::AlpnRequired));
    let _server_result = server.join().expect("server thread");
}

#[test]
fn required_alpn_rejects_no_common_protocol() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[b"http/1.1"],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "required_alpn:v1"),
        client_policy(&[b"h2"], AlpnRequirement::Required),
    )
    .expect("required ALPN plan")
    .authenticate()
    .expect_err("no shared ALPN must fail policy");
    assert!(matches!(error, TlsError::AlpnRequired));
    let _server_result = server.join().expect("server thread");
}

fn assert_dns_certificate_failure(
    subject_alt_names: Vec<String>,
    common_name: Option<&str>,
    expected: fn(&TlsError) -> bool,
) {
    let material = valid_material(subject_alt_names, common_name);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "identity_failure:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect("identity failure plan")
    .authenticate()
    .expect_err("certificate identity must fail");
    assert!(expected(&error));
    let _server_result = server.join().expect("server thread");
}

#[test]
fn wrong_dns_san_is_rejected() {
    assert_dns_certificate_failure(vec!["other.example".to_owned()], None, |error| {
        matches!(error, TlsError::ServiceIdentityMismatch { .. })
    });
}

#[test]
fn common_name_never_replaces_a_dns_subject_alt_name() {
    assert_dns_certificate_failure(
        vec!["other.example".to_owned()],
        Some("localhost"),
        |error| matches!(error, TlsError::ServiceIdentityMismatch { .. }),
    );
}

#[test]
fn an_untrusted_root_is_rejected() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (_server_root, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (untrusted_root, _issuer) = certificate_authority("Unrelated test root");
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(untrusted_root, "wrong_roots:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect("untrusted root plan")
    .authenticate()
    .expect_err("untrusted issuer must fail");
    assert!(matches!(error, TlsError::UnknownIssuer { .. }));
    let _server_result = server.join().expect("server thread");
}

#[test]
fn fixed_trusted_time_rejects_expired_and_future_certificates() {
    for (not_before, not_after, is_expected) in [
        ((2020, 1, 1), (2025, 1, 1), true),
        ((2027, 1, 1), (2030, 1, 1), false),
    ] {
        let material =
            certificate_material(vec!["localhost".to_owned()], None, not_before, not_after);
        let (root_der, config) = server_config(
            material,
            &[],
            &[&rustls::version::TLS13, &rustls::version::TLS12],
        );
        let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
        let origin = origin_for("localhost", socket_address);
        let error = TlsHandshakePlan::new(
            origin.clone(),
            direct_connection(&origin, socket_address),
            trust_bundle(root_der, "validity_roots:v1"),
            client_policy(&[], AlpnRequirement::Optional),
        )
        .expect("validity plan")
        .authenticate()
        .expect_err("certificate validity must be enforced");
        if is_expected {
            assert!(matches!(error, TlsError::CertificateExpired { .. }));
        } else {
            assert!(matches!(error, TlsError::CertificateNotYetValid { .. }));
        }
        let _server_result = server.join().expect("server thread");
    }
}

fn authenticate_ip_literal(bind_address: IpAddr, origin_host: &str, san: &str) {
    let material = valid_material(vec![san.to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(bind_address, config);
    let origin = origin_for(origin_host, socket_address);
    let authenticated = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "literal_ip_roots:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect("literal IP plan")
    .authenticate()
    .expect("exact IP SAN must authenticate");
    assert_eq!(authenticated.evidence().origin(), &origin);
    assert_eq!(authenticated.evidence().observed_peer(), socket_address);
    assert_eq!(server.join().expect("server thread"), Ok(None));
}

#[test]
fn literal_ipv4_origin_requires_an_exact_ip_san() {
    authenticate_ip_literal(IpAddr::V4(Ipv4Addr::LOCALHOST), "127.0.0.1", "127.0.0.1");
}

#[test]
fn literal_ipv6_origin_requires_an_exact_ip_san() {
    authenticate_ip_literal(IpAddr::V6(Ipv6Addr::LOCALHOST), "[::1]", "::1");
}

#[test]
fn tls12_and_tls13_are_independently_supported() {
    for (version, expected) in [
        (&rustls::version::TLS12, TlsProtocolVersion::Tls12),
        (&rustls::version::TLS13, TlsProtocolVersion::Tls13),
    ] {
        let material = valid_material(vec!["localhost".to_owned()], None);
        let (root_der, config) = server_config(material, &[], &[version]);
        let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
        let origin = origin_for("localhost", socket_address);
        let authenticated = TlsHandshakePlan::new(
            origin.clone(),
            direct_connection(&origin, socket_address),
            trust_bundle(root_der, "protocol_roots:v1"),
            client_policy(&[], AlpnRequirement::Optional),
        )
        .expect("protocol plan")
        .authenticate()
        .expect("supported protocol must authenticate");
        assert_eq!(authenticated.evidence().protocol_version(), expected);
        assert_eq!(server.join().expect("server thread"), Ok(None));
    }
}

#[test]
fn tls_origin_must_match_the_transport_authority_origin() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let transport_origin = origin_for("localhost", socket_address);
    let tls_origin = origin_for("127.0.0.1", socket_address);
    let error = TlsHandshakePlan::new(
        tls_origin,
        direct_connection(&transport_origin, socket_address),
        trust_bundle(root_der, "origin_binding:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect_err("TLS origin cannot replace transport authority");
    assert!(matches!(error, TlsError::TransportOriginMismatch { .. }));
    let _server_result = server.join().expect("server thread");
}

#[test]
fn a_non_https_origin_is_rejected_before_tls_bytes() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = Origin::parse(&format!("http://localhost:{}", socket_address.port()))
        .expect("managed loopback HTTP origin");
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "https_only:v1"),
        client_policy(&[], AlpnRequirement::Optional),
    )
    .expect_err("HTTP origin cannot enter TLS identity authority");
    assert!(matches!(error, TlsError::OriginRequiresHttps { .. }));
    let _server_result = server.join().expect("server thread");
}
