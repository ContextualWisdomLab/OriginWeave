#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    AlpnHttp11Policy, BodyFraming, HttpClientPolicy, HttpExchangePlan, HttpMethod,
    HttpRequestTarget, IntegrityRequirement,
};
use originweave_network::{ConnectionPlan, DirectTcpConnection};
use originweave_tls::{
    AlpnRequirement, TlsClientPolicy, TlsHandshakePlan, TrustBundleIdentifier, TrustRootBundle,
};
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;
const TLS_TIMEOUT: Duration = Duration::from_secs(3);
const HTTP_TIMEOUT: Duration = Duration::from_millis(100);
const KEEP_ALIVE_HOLD: Duration = Duration::from_millis(350);

fn server_material() -> (Vec<u8>, Arc<ServerConfig>) {
    let mut root_params = CertificateParams::new(Vec::new()).expect("empty CA SAN list");
    root_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    root_params
        .distinguished_name
        .push(DnType::CommonName, "OriginWeave persistent HTTP test root");
    let root_key = KeyPair::generate().expect("test CA key generation");
    let root_certificate = root_params
        .self_signed(&root_key)
        .expect("test CA certificate generation");
    let issuer = Issuer::new(root_params, root_key);

    let mut leaf_params =
        CertificateParams::new(vec!["localhost".to_owned()]).expect("localhost SAN");
    leaf_params.not_before = rcgen::date_time_ymd(2025, 1, 1);
    leaf_params.not_after = rcgen::date_time_ymd(2030, 1, 1);
    leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    leaf_params.use_authority_key_identifier_extension = true;
    let leaf_key = KeyPair::generate().expect("test leaf key generation");
    let leaf_certificate = leaf_params
        .signed_by(&leaf_key, &issuer)
        .expect("test leaf certificate generation");

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .expect("test protocol versions");
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(leaf_certificate.der().to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
        )
        .expect("test certificate and key must match");
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    (root_certificate.der().to_vec(), Arc::new(config))
}

fn spawn_persistent_server(config: Arc<ServerConfig>) -> (SocketAddr, thread::JoinHandle<()>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let handle = thread::spawn(move || {
        let (stream, _peer) = listener.accept().expect("accept loopback client");
        stream
            .set_read_timeout(Some(TLS_TIMEOUT))
            .expect("server read timeout");
        stream
            .set_write_timeout(Some(TLS_TIMEOUT))
            .expect("server write timeout");
        let connection = ServerConnection::new(config).expect("server TLS connection");
        let mut tls = StreamOwned::new(connection, stream);
        let mut request = Vec::new();
        let mut scratch = [0_u8; 512];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = tls.read(&mut scratch).expect("read request bytes");
            assert_ne!(count, 0, "request must complete before EOF");
            request.extend_from_slice(&scratch[..count]);
        }
        tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello")
            .expect("write complete framed response");
        tls.flush().expect("flush framed response");
        thread::sleep(KEEP_ALIVE_HOLD);
        tls.conn.send_close_notify();
        let _ = tls.flush();
    });
    (address, handle)
}

fn connection(
    origin: &Origin,
    address: SocketAddr,
    root_der: Vec<u8>,
) -> originweave_tls::AuthenticatedTlsConnection {
    let resolution = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution");
    let tcp = ConnectionPlan::new(&resolution, address, Duration::from_secs(2), 1)
        .expect("direct connection plan")
        .connect()
        .expect("loopback TCP connection");
    authenticate(origin, tcp, root_der)
}

fn authenticate(
    origin: &Origin,
    tcp: DirectTcpConnection,
    root_der: Vec<u8>,
) -> originweave_tls::AuthenticatedTlsConnection {
    let roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("persistent_content_length:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("test root bundle");
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TLS_TIMEOUT,
        vec![b"http/1.1".to_vec()],
        AlpnRequirement::Required,
    )
    .expect("TLS client policy");
    TlsHandshakePlan::new(origin.clone(), tcp, roots, policy)
        .expect("TLS handshake plan")
        .authenticate()
        .expect("authenticated loopback TLS")
}

fn http_policy() -> HttpClientPolicy {
    let defaults = HttpClientPolicy::strict_defaults();
    HttpClientPolicy::new(
        HTTP_TIMEOUT,
        defaults.max_request_bytes(),
        defaults.max_status_line_bytes(),
        defaults.max_header_field_count(),
        defaults.max_header_name_bytes(),
        defaults.max_header_value_bytes(),
        defaults.max_header_section_bytes(),
        defaults.max_interim_response_count(),
        defaults.max_chunk_count(),
        defaults.max_trailer_field_count(),
        defaults.max_trailer_section_bytes(),
        defaults.max_encoded_content_bytes(),
        defaults.max_decoded_content_bytes(),
        defaults.max_content_expansion_ratio(),
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("bounded HTTP policy")
}

#[test]
fn content_length_response_completes_without_transport_eof() {
    let (root_der, config) = server_material();
    let (address, server) = spawn_persistent_server(config);
    let origin = Origin::parse(&format!("https://localhost:{}", address.port())).expect("origin");
    let target = HttpRequestTarget::parse(origin.clone(), "/persistent").expect("target");
    let result = HttpExchangePlan::new(
        connection(&origin, address, root_der),
        HttpMethod::Get,
        target,
        &[],
        http_policy(),
    )
    .expect("HTTP exchange plan")
    .execute();

    let response = result.expect("Content-Length must delimit the response before transport EOF");
    assert_eq!(response.content(), b"hello");
    assert_eq!(
        response.evidence().body_framing(),
        BodyFraming::ContentLength(5)
    );
    server.join().expect("server thread");
}
