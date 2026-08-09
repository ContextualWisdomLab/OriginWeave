#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    AlpnHttp11Policy, HttpClientPolicy, HttpError, HttpExchangePlan, HttpMethod, HttpRequestTarget,
    IntegrityRequirement,
};
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
const SEGMENT_DELAY: Duration = Duration::from_millis(50);

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
        .push(DnType::CommonName, "OriginWeave response failure test root");
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

fn read_request(tls: &mut StreamOwned<ServerConnection, std::net::TcpStream>) -> ServerResult {
    let mut request = Vec::new();
    let mut scratch = [0_u8; 512];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        match tls.read(&mut scratch) {
            Ok(0) => break,
            Ok(count) => request.extend_from_slice(&scratch[..count]),
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                if request.is_empty() {
                    break;
                }
                return Err(error.to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(request)
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
        let request = read_request(&mut tls)?;
        if !response.is_empty() {
            tls.write_all(response).map_err(|error| error.to_string())?;
            tls.flush().map_err(|error| error.to_string())?;
        }
        tls.conn.send_close_notify();
        tls.flush().map_err(|error| error.to_string())?;
        Ok(request)
    });
    (socket_address, handle)
}

fn spawn_segmented_chunked_wire_overflow_server(
    config: Arc<ServerConfig>,
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
        let request = read_request(&mut tls)?;

        // The body prefix is exactly 37 bytes: one accepted chunk followed by an unterminated
        // 16-byte chunk-size line. Under `tiny_chunked_wire_policy` the complete wire budget is
        // 42 bytes, leaving five bytes of capacity and forcing the client to perform the six-byte
        // sentinel read that must be rejected before buffer growth.
        tls.write_all(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0000000000000001\r\na\r\n0000000000000000",
        )
        .map_err(|error| error.to_string())?;
        tls.flush().map_err(|error| error.to_string())?;
        thread::sleep(SEGMENT_DELAY);
        tls.write_all(b"000000")
            .map_err(|error| error.to_string())?;
        tls.flush().map_err(|error| error.to_string())?;
        tls.conn.send_close_notify();
        let _ = tls.flush();
        Ok(request)
    });
    (socket_address, handle)
}

fn origin_for(socket_address: SocketAddr) -> Origin {
    Origin::parse(&format!("https://localhost:{}", socket_address.port())).expect("test origin")
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
) -> originweave_tls::AuthenticatedTlsConnection {
    let roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("http_response_failure:v1").expect("trust identifier"),
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

fn execute_with_policy(
    method: HttpMethod,
    response: &'static [u8],
    policy: HttpClientPolicy,
) -> (
    Result<originweave_http::AuthenticatedHttpResponse, HttpError>,
    JoinHandle<ServerResult>,
) {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_http_server(config, response);
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der);
    let target = HttpRequestTarget::parse(origin, "/resource").expect("target");
    let result = HttpExchangePlan::new(connection, method, target, &[], policy)
        .expect("HTTP exchange plan")
        .execute();
    (result, server)
}

fn execute(
    method: HttpMethod,
    response: &'static [u8],
) -> (
    Result<originweave_http::AuthenticatedHttpResponse, HttpError>,
    JoinHandle<ServerResult>,
) {
    execute_with_policy(method, response, HttpClientPolicy::strict_defaults())
}

fn tiny_chunked_wire_policy() -> HttpClientPolicy {
    let defaults = HttpClientPolicy::strict_defaults();
    HttpClientPolicy::new(
        TEST_TIMEOUT,
        defaults.max_request_bytes(),
        defaults.max_status_line_bytes(),
        defaults.max_header_field_count(),
        defaults.max_header_name_bytes(),
        defaults.max_header_value_bytes(),
        defaults.max_header_section_bytes(),
        defaults.max_interim_response_count(),
        1,
        defaults.max_trailer_field_count(),
        1,
        1,
        1,
        1,
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("tiny chunked-wire policy")
}

fn execute_segmented_chunked_wire_overflow() -> (
    Result<originweave_http::AuthenticatedHttpResponse, HttpError>,
    JoinHandle<ServerResult>,
) {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_segmented_chunked_wire_overflow_server(config);
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der);
    let target = HttpRequestTarget::parse(origin, "/segmented-overflow").expect("target");
    let result = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        tiny_chunked_wire_policy(),
    )
    .expect("HTTP exchange plan")
    .execute();
    (result, server)
}

fn join_server(server: JoinHandle<ServerResult>) {
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn clean_eof_before_any_response_head_is_incomplete() {
    let (result, server) = execute(HttpMethod::Get, b"");
    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
    join_server(server);
}

#[test]
fn clean_tls_eof_before_first_chunk_is_incomplete() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
    );
    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
    join_server(server);
}

#[test]
fn chunked_body_prefix_over_independent_wire_budget_fails_before_parsing() {
    let (result, server) = execute_with_policy(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n1111111111111111111111111111111111111111111111111111111111111111",
        tiny_chunked_wire_policy(),
    );
    assert!(matches!(
        result,
        Err(HttpError::EncodedContentTooLarge {
            byte_count,
            maximum_bytes: 42,
        }) if byte_count > 42
    ));
    join_server(server);
}

#[test]
fn segmented_chunked_wire_growth_rejects_sentinel_read_before_buffer_expansion() {
    let (result, server) = execute_segmented_chunked_wire_overflow();
    assert!(matches!(
        result,
        Err(HttpError::EncodedContentTooLarge {
            byte_count: 43,
            maximum_bytes: 42,
        })
    ));
    join_server(server);
}

#[test]
fn no_content_semantics_reject_payload_bytes_already_received_with_the_head() {
    let (result, server) = execute(
        HttpMethod::Head,
        b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx",
    );
    assert!(matches!(
        result,
        Err(HttpError::UnexpectedResponseBytes { byte_count: 1 })
    ));
    join_server(server);
}

#[test]
fn content_length_rejects_surplus_bytes_already_received_with_the_head() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nxy",
    );
    assert!(matches!(
        result,
        Err(HttpError::UnexpectedResponseBytes { byte_count: 1 })
    ));
    join_server(server);
}

#[test]
fn clean_eof_with_a_truncated_chunk_is_incomplete() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nab",
    );
    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
    join_server(server);
}

#[test]
fn chunked_terminal_section_rejects_surplus_wire_bytes() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n1\r\na\r\n0\r\n\r\nx",
    );
    assert!(matches!(
        result,
        Err(HttpError::UnexpectedResponseBytes { byte_count: 1 })
    ));
    join_server(server);
}
