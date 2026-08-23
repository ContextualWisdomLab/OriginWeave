#![allow(clippy::expect_used)]

use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    HttpClientPolicy, HttpError, HttpExchangePlan, HttpMethod, HttpRequestTarget,
};
use originweave_network::ConnectionPlan;
use originweave_tls::{
    AlpnRequirement, TlsClientPolicy, TlsHandshakePlan, TrustBundleIdentifier, TrustRootBundle,
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;
const TEST_TIMEOUT: Duration = Duration::from_secs(3);
const SEGMENT_DELAY: Duration = Duration::from_millis(50);

struct CertificateMaterial {
    root_der: Vec<u8>,
    certificate_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

fn certificate_authority() -> (Vec<u8>, Issuer<'static, KeyPair>) {
    let mut parameters = CertificateParams::new(Vec::new()).expect("empty CA SAN list");
    parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    parameters.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    parameters.distinguished_name.push(
        DnType::CommonName,
        "OriginWeave segmented surplus test root",
    );
    let key_pair = KeyPair::generate().expect("test CA key generation");
    let certificate = parameters
        .self_signed(&key_pair)
        .expect("test CA certificate generation");
    (
        certificate.der().to_vec(),
        Issuer::new(parameters, key_pair),
    )
}

fn certificate_material() -> CertificateMaterial {
    let (root_der, issuer) = certificate_authority();
    let mut parameters =
        CertificateParams::new(vec!["localhost".to_owned()]).expect("localhost SAN");
    parameters.not_before = rcgen::date_time_ymd(2025, 1, 1);
    parameters.not_after = rcgen::date_time_ymd(2030, 1, 1);
    parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    parameters.use_authority_key_identifier_extension = true;
    let key_pair = KeyPair::generate().expect("test leaf key generation");
    let certificate: Certificate = parameters
        .signed_by(&key_pair, &issuer)
        .expect("test leaf certificate generation");
    CertificateMaterial {
        root_der,
        certificate_chain: vec![certificate.der().clone()],
        private_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der())),
    }
}

fn server_config(material: CertificateMaterial) -> (Vec<u8>, Arc<ServerConfig>) {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .expect("test protocol versions");
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(material.certificate_chain, material.private_key)
        .expect("test certificate and key must match");
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    (material.root_der, Arc::new(config))
}

fn read_request(tls: &mut StreamOwned<ServerConnection, std::net::TcpStream>) -> io::Result<()> {
    let mut request = Vec::new();
    let mut scratch = [0_u8; 512];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        match tls.read(&mut scratch) {
            Ok(0) => break,
            Ok(count) => request.extend_from_slice(&scratch[..count]),
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn spawn_segmented_surplus_server(config: Arc<ServerConfig>) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback listener");
    let socket_address = listener.local_addr().expect("listener address");
    let handle = thread::spawn(move || {
        let (stream, _peer) = listener.accept().expect("accept client");
        stream
            .set_read_timeout(Some(TEST_TIMEOUT))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(TEST_TIMEOUT))
            .expect("write timeout");
        let connection = ServerConnection::new(config).expect("server TLS connection");
        let mut tls = StreamOwned::new(connection, stream);
        read_request(&mut tls).expect("server request read");
        tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx")
            .expect("write declared response");
        tls.flush().expect("flush declared response");
        thread::sleep(SEGMENT_DELAY);
        let _ = tls.write_all(b"y");
        let _ = tls.flush();
        tls.conn.send_close_notify();
        let _ = tls.flush();
    });
    (socket_address, handle)
}

fn authenticated_connection(
    origin: &Origin,
    socket_address: SocketAddr,
    root_der: Vec<u8>,
) -> originweave_tls::AuthenticatedTlsConnection {
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution");
    let direct = ConnectionPlan::new(&snapshot, socket_address, Duration::from_secs(2), 1)
        .expect("direct connection plan")
        .connect()
        .expect("loopback TCP connection");
    let roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("http_segmented_surplus:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("test root bundle");
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        vec![b"http/1.1".to_vec()],
        AlpnRequirement::Required,
    )
    .expect("TLS client policy");
    TlsHandshakePlan::new(origin.clone(), direct, roots, policy)
        .expect("TLS handshake plan")
        .authenticate()
        .expect("authenticated loopback TLS")
}

#[test]
fn content_length_rejects_surplus_bytes_arriving_after_the_declared_body() {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_segmented_surplus_server(config);
    let origin = Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .expect("test origin");
    let connection = authenticated_connection(&origin, socket_address, root_der);
    let target = HttpRequestTarget::parse(origin, "/segmented-surplus").expect("target");
    let result = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        HttpClientPolicy::strict_defaults(),
    )
    .expect("HTTP exchange plan")
    .execute();

    assert!(matches!(
        result,
        Err(HttpError::UnexpectedResponseBytes { byte_count: 1 })
    ));
    server.join().expect("server thread");
}
