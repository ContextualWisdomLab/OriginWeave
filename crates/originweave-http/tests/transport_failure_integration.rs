#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener};
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
const EXCHANGE_TIMEOUT: Duration = Duration::from_millis(75);

type ServerResult = Result<Vec<u8>, String>;

struct CertificateMaterial {
    root_der: Vec<u8>,
    certificate_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

enum ServerBehavior {
    WriteThenStall(&'static [u8], Duration),
    WriteThenUncleanClose(&'static [u8]),
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
        "OriginWeave HTTP transport-failure test root",
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

fn spawn_http_server(
    config: Arc<ServerConfig>,
    behavior: ServerBehavior,
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

        match behavior {
            ServerBehavior::WriteThenStall(response, delay) => {
                tls.write_all(response).map_err(|error| error.to_string())?;
                tls.flush().map_err(|error| error.to_string())?;
                thread::sleep(delay);
            }
            ServerBehavior::WriteThenUncleanClose(response) => {
                tls.write_all(response).map_err(|error| error.to_string())?;
                tls.flush().map_err(|error| error.to_string())?;
                tls.sock
                    .shutdown(Shutdown::Both)
                    .map_err(|error| error.to_string())?;
            }
        }
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
        TrustBundleIdentifier::parse("http_transport_failure_loopback:v1")
            .expect("trust identifier"),
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

fn policy_with_exchange_timeout(exchange_timeout: Duration) -> HttpClientPolicy {
    let defaults = HttpClientPolicy::strict_defaults();
    HttpClientPolicy::new(
        exchange_timeout,
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

fn execute(
    behavior: ServerBehavior,
    policy: HttpClientPolicy,
) -> (
    Result<originweave_http::AuthenticatedHttpResponse, HttpError>,
    JoinHandle<ServerResult>,
) {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_http_server(config, behavior);
    let origin = origin_for(socket_address);
    let connection = authenticated_connection(&origin, socket_address, root_der);
    let target = HttpRequestTarget::parse(origin, "/transport-failure").expect("target");
    let result = HttpExchangePlan::new(connection, HttpMethod::Get, target, &[], policy)
        .expect("HTTP exchange plan")
        .execute();
    (result, server)
}

#[test]
fn stalled_content_length_body_expires_the_total_exchange_deadline() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhe";
    let (result, server) = execute(
        ServerBehavior::WriteThenStall(response, Duration::from_millis(300)),
        policy_with_exchange_timeout(EXCHANGE_TIMEOUT),
    );

    assert!(matches!(
        result,
        Err(HttpError::HttpExchangeTimedOut { timeout }) if timeout == EXCHANGE_TIMEOUT
    ));
    let request = server
        .join()
        .expect("server thread")
        .expect("server exchange");
    assert!(request.starts_with(b"GET /transport-failure HTTP/1.1\r\n"));
}

#[test]
fn close_delimited_body_requires_authenticated_tls_close_notify() {
    let response = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\npartial";
    let (result, server) = execute(
        ServerBehavior::WriteThenUncleanClose(response),
        HttpClientPolicy::strict_defaults(),
    );

    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
    let request = server
        .join()
        .expect("server thread")
        .expect("server exchange");
    assert!(request.starts_with(b"GET /transport-failure HTTP/1.1\r\n"));
}

#[test]
fn local_write_half_shutdown_is_reported_as_transport_io_failure() {
    let material = certificate_material();
    let (root_der, config) = server_config(material);
    let (socket_address, server) = spawn_http_server(
        config,
        ServerBehavior::WriteThenStall(b"", Duration::from_millis(25)),
    );
    let origin = origin_for(socket_address);
    let mut connection = authenticated_connection(&origin, socket_address, root_der);
    connection
        .stream_mut()
        .sock
        .shutdown(Shutdown::Write)
        .expect("locally shut down write half");
    let target = HttpRequestTarget::parse(origin, "/write-failure").expect("target");

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
        Err(HttpError::HttpExchangeIoFailed { .. })
    ));
    let request = server
        .join()
        .expect("server thread")
        .expect("server exchange");
    assert!(request.is_empty());
}
