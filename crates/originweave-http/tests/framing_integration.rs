#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    AlpnHttp11Policy, BodyFraming, HttpClientPolicy, HttpError, HttpExchangePlan, HttpMethod,
    HttpRequestTarget, IntegrityRequirement,
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
        .push(DnType::CommonName, "OriginWeave HTTP framing test root");
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
        let _ = tls.flush();
        Ok(request)
    });
    (socket_address, handle)
}

fn spawn_segmented_http_server(
    config: Arc<ServerConfig>,
    response_head: &'static [u8],
    response_body: &'static [u8],
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
        tls.write_all(response_head)
            .map_err(|error| error.to_string())?;
        tls.flush().map_err(|error| error.to_string())?;
        thread::sleep(Duration::from_millis(25));
        tls.write_all(response_body)
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
        TrustBundleIdentifier::parse("http_framing_loopback:v1").expect("trust identifier"),
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

fn policy_with_max_encoded(max_encoded_content_bytes: usize) -> HttpClientPolicy {
    let defaults = HttpClientPolicy::strict_defaults();
    HttpClientPolicy::new(
        defaults.exchange_timeout(),
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
        max_encoded_content_bytes,
        defaults.max_decoded_content_bytes(),
        defaults.max_content_expansion_ratio(),
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("bounded HTTP policy")
}

fn execute(
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

fn execute_segmented(
    response_head: &'static [u8],
    response_body: &'static [u8],
    policy: HttpClientPolicy,
) -> (
    Result<originweave_http::AuthenticatedHttpResponse, HttpError>,
    JoinHandle<ServerResult>,
) {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) =
        spawn_segmented_http_server(config, response_head, response_body);
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der);
    let target = HttpRequestTarget::parse(origin, "/segmented").expect("target");
    let result = HttpExchangePlan::new(connection, HttpMethod::Get, target, &[], policy)
        .expect("HTTP exchange plan")
        .execute();
    (result, server)
}

#[test]
fn head_response_with_declared_length_exposes_no_content() {
    let (result, server) = execute(
        HttpMethod::Head,
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n",
        HttpClientPolicy::strict_defaults(),
    );
    let response = result.expect("complete HEAD response");
    assert!(response.content().is_empty());
    assert_eq!(response.evidence().body_framing(), BodyFraming::NoContent);
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn segmented_content_length_body_is_read_after_the_response_head() {
    let (result, server) = execute_segmented(
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n",
        b"hello",
        HttpClientPolicy::strict_defaults(),
    );
    let response = result.expect("segmented content-length response");
    assert_eq!(response.content(), b"hello");
    assert_eq!(
        response.evidence().body_framing(),
        BodyFraming::ContentLength(5)
    );
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn incomplete_content_length_fails_closed() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nhello",
        HttpClientPolicy::strict_defaults(),
    );
    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn chunked_response_with_lowercase_size_and_trailer_is_complete() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\na\r\n0123456789\r\n0\r\nx-trace: ok\r\n\r\n",
        HttpClientPolicy::strict_defaults(),
    );
    let response = result.expect("complete chunked response");
    assert_eq!(response.content(), b"0123456789");
    assert_eq!(response.evidence().body_framing(), BodyFraming::Chunked);
    assert_eq!(response.evidence().chunk_count(), 2);
    assert_eq!(response.evidence().trailer_fields().len(), 1);
    assert_eq!(response.evidence().trailer_fields()[0].name(), "x-trace");
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn close_delimited_response_completes_only_at_clean_tls_eof() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nhello",
        HttpClientPolicy::strict_defaults(),
    );
    let response = result.expect("clean close-delimited response");
    assert_eq!(response.content(), b"hello");
    assert_eq!(
        response.evidence().body_framing(),
        BodyFraming::CloseDelimited
    );
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn close_delimited_response_exceeding_encoded_budget_fails_closed() {
    let (result, server) = execute(
        HttpMethod::Get,
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nhello",
        policy_with_max_encoded(4),
    );
    assert!(matches!(
        result,
        Err(HttpError::EncodedContentTooLarge {
            byte_count: 5,
            maximum_bytes: 4,
        })
    ));
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn streamed_close_delimited_overflow_is_detected_after_filling_the_exact_budget() {
    let (result, server) = execute_segmented(
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n",
        b"hello",
        policy_with_max_encoded(4),
    );
    assert!(matches!(
        result,
        Err(HttpError::EncodedContentTooLarge {
            byte_count: 5,
            maximum_bytes: 4,
        })
    ));
    server
        .join()
        .expect("server thread")
        .expect("server exchange");
}

#[test]
fn mismatched_http_origin_is_rejected_before_request_bytes() {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_http_server(config, b"");
    let tls_origin = origin_for(socket_address);
    let connection = authenticated_connection(&tls_origin, socket_address, root_der);
    let http_origin = Origin::parse("https://example.com").expect("different origin");
    let target = HttpRequestTarget::parse(http_origin.clone(), "/").expect("target");

    let error = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        HttpClientPolicy::strict_defaults(),
    )
    .expect_err("origin mismatch must fail before request serialization");
    assert!(matches!(
        error,
        HttpError::OriginAuthorityMismatch {
            http_origin,
            tls_origin: observed_tls_origin,
        } if http_origin == Origin::parse("https://example.com").expect("expected origin")
            && observed_tls_origin == tls_origin
    ));

    let request = server
        .join()
        .expect("server thread")
        .expect("server exchange");
    assert!(request.is_empty());
}
