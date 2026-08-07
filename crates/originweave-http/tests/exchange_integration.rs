#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{HttpClientPolicy, HttpError, HttpExchangePlan, HttpMethod, HttpRequestTarget};
use originweave_network::{ConnectionPlan, DirectTcpConnection};
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

type ServerResult = Result<Vec<u8>, String>;

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
    parameters
        .distinguished_name
        .push(DnType::CommonName, "OriginWeave HTTP test root");
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

fn server_config(material: CertificateMaterial, alpn: &[u8]) -> (Vec<u8>, Arc<ServerConfig>) {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .expect("test protocol versions");
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(material.certificate_chain, material.private_key)
        .expect("test certificate and key must match");
    config.alpn_protocols = vec![alpn.to_vec()];
    (material.root_der, Arc::new(config))
}

fn spawn_http_server(
    config: Arc<ServerConfig>,
    response: &'static [u8],
) -> (SocketAddr, JoinHandle<ServerResult>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback listener");
    let socket_address = listener.local_addr().expect("listener address");
    let handle = thread::spawn(move || {
        let (stream, _peer) = listener.accept().map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let connection = ServerConnection::new(config).map_err(|error| error.to_string())?;
        let mut tls = StreamOwned::new(connection, stream);
        let mut request = Vec::new();
        let mut scratch = [0_u8; 512];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = tls.read(&mut scratch).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            request.extend_from_slice(&scratch[..count]);
        }
        if !response.is_empty() {
            tls.write_all(response).map_err(|error| error.to_string())?;
            tls.flush().map_err(|error| error.to_string())?;
        }
        tls.conn.send_close_notify();
        let _ = tls.flush();
        Ok(request)
    });
    (socket_address, handle)
}

fn origin_for(socket_address: SocketAddr) -> Origin {
    Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .expect("test origin")
}

fn direct_connection(origin: &Origin, socket_address: SocketAddr) -> DirectTcpConnection {
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution");
    ConnectionPlan::new(&snapshot, socket_address, Duration::from_secs(2), 1)
        .expect("direct connection plan")
        .connect()
        .expect("loopback TCP connection")
}

fn authenticated_connection(
    origin: &Origin,
    socket_address: SocketAddr,
    root_der: Vec<u8>,
    alpn: &[u8],
) -> originweave_tls::AuthenticatedTlsConnection {
    let roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("http_loopback:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("test root bundle");
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        vec![alpn.to_vec()],
        AlpnRequirement::Required,
    )
    .expect("TLS client policy");
    TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(origin, socket_address),
        roots,
        policy,
    )
    .expect("TLS handshake plan")
    .authenticate()
    .expect("authenticated loopback TLS")
}

#[test]
fn authenticated_http11_get_uses_the_exact_tls_stream_and_returns_complete_evidence() {
    let material = certificate_material();
    let (root_der, config) = server_config(material, b"http/1.1");
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nhello";
    let (socket_address, server) = spawn_http_server(config, response);
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der, b"http/1.1");
    let target = HttpRequestTarget::parse(origin.clone(), "/hello?q=secret").expect("target");

    let response = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        HttpClientPolicy::strict_defaults(),
    )
    .expect("HTTP plan")
    .execute()
    .expect("bounded HTTP exchange");

    assert_eq!(response.content(), b"hello");
    assert_eq!(response.evidence().origin(), &origin);
    assert_eq!(response.evidence().requested_peer(), socket_address);
    assert_eq!(response.evidence().observed_peer(), socket_address);
    assert_eq!(response.evidence().method(), HttpMethod::Get);
    assert_eq!(response.evidence().status_code(), 200);
    assert!(response.evidence().query_present());
    assert_eq!(response.evidence().path_prefix(), "/hello");
    assert!(response.evidence().response_complete());
    assert!(response.redirect().is_none());

    let request = server
        .join()
        .expect("HTTP server thread")
        .expect("HTTP server result");
    let request = String::from_utf8(request).expect("ASCII request");
    assert!(request.starts_with("GET /hello?q=secret HTTP/1.1\r\n"));
    assert!(request.contains(&format!("Host: localhost:{}\r\n", socket_address.port())));
    assert!(request.contains("Connection: close\r\n"));
}

#[test]
fn non_http11_alpn_is_rejected_before_any_http_request_bytes() {
    let material = certificate_material();
    let (root_der, config) = server_config(material, b"h2");
    let (socket_address, server) = spawn_http_server(config, b"");
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der, b"h2");
    let target = HttpRequestTarget::parse(origin, "/").expect("target");

    let error = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        HttpClientPolicy::strict_defaults(),
    )
    .expect_err("h2 cannot authorize the HTTP/1.1 parser");
    assert!(matches!(error, HttpError::UnexpectedAlpn));

    let request = server
        .join()
        .expect("HTTP server thread")
        .expect("HTTP server result");
    assert!(request.is_empty());
}
