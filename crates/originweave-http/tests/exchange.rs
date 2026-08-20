#![allow(clippy::expect_used)]

use std::error::Error;
use std::io::{self, Cursor, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    HttpError, HttpExchange, HttpExchangePolicy, HttpMethod, HttpRequest, HttpRequestError,
    HttpRequestTarget, HttpResponse, MAX_CHUNK_COUNT, MAX_HEADER_FIELDS, MAX_HEADER_NAME_BYTES,
    MAX_HEADER_VALUE_BYTES, MAX_HTTP_BODY_BYTES, MAX_TRAILER_FIELDS, MAX_TRAILER_SECTION_BYTES,
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

struct ServerMaterial {
    root_der: Vec<u8>,
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
}

fn server_material() -> ServerMaterial {
    let mut root_parameters = CertificateParams::new(Vec::new()).expect("root SANs");
    root_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    root_parameters.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
    ];
    root_parameters
        .distinguished_name
        .push(DnType::CommonName, "OriginWeave HTTP test root");
    let root_key = KeyPair::generate().expect("root key");
    let root_certificate = root_parameters
        .self_signed(&root_key)
        .expect("root certificate");
    let issuer = Issuer::new(root_parameters, root_key);

    let mut leaf_parameters = CertificateParams::new(vec!["localhost".to_owned()]).expect("SANs");
    leaf_parameters.not_before = rcgen::date_time_ymd(2025, 1, 1);
    leaf_parameters.not_after = rcgen::date_time_ymd(2030, 1, 1);
    leaf_parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    leaf_parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    leaf_parameters.use_authority_key_identifier_extension = true;
    let leaf_key = KeyPair::generate().expect("server key");
    let certificate: Certificate = leaf_parameters
        .signed_by(&leaf_key, &issuer)
        .expect("server certificate");
    ServerMaterial {
        root_der: root_certificate.der().to_vec(),
        chain: vec![certificate.der().clone()],
        key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
    }
}

fn server_config(material: ServerMaterial, alpn: &[&[u8]]) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .expect("TLS versions");
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(material.chain, material.key)
        .expect("server certificate and key");
    config.alpn_protocols = alpn.iter().map(|value| value.to_vec()).collect();
    config.max_early_data_size = 0;
    config.key_log = Arc::new(rustls::NoKeyLog {});
    Arc::new(config)
}

fn spawn_server(
    config: Arc<ServerConfig>,
    response: Option<&'static [u8]>,
) -> (SocketAddr, JoinHandle<Result<Vec<u8>, String>>) {
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let thread = thread::spawn(move || {
        let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(TEST_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let mut connection = ServerConnection::new(config).map_err(|error| error.to_string())?;
        let handshake_result = connection.complete_io(&mut stream);
        if response.is_none() {
            thread::sleep(Duration::from_millis(20));
            return Ok(Vec::new());
        }
        handshake_result.map_err(|error| error.to_string())?;
        let mut stream = StreamOwned::new(connection, stream);
        let mut request = Vec::new();
        let response = response.expect("response checked above");
        let mut buffer = [0_u8; 128];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream
                .read(&mut buffer)
                .map_err(|error| error.to_string())?;
            if read == 0 || request.len() + read > 16 * 1024 {
                return Err("request did not reach its bounded header boundary".to_owned());
            }
            request.extend_from_slice(&buffer[..read]);
        }
        stream
            .write_all(response)
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
        Ok(request)
    });
    (address, thread)
}

fn make_origin(address: SocketAddr) -> Origin {
    Origin::parse(&format!("https://localhost:{}", address.port())).expect("origin")
}

fn connection(
    origin: &Origin,
    address: SocketAddr,
    root_der: Vec<u8>,
    alpn: &[&[u8]],
    requirement: AlpnRequirement,
) -> originweave_tls::AuthenticatedTlsConnection {
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [address.ip()],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("resolution approval");
    let tcp = ConnectionPlan::new(&snapshot, address, Duration::from_secs(2), 1)
        .expect("connection plan")
        .connect()
        .expect("TCP connection");
    let roots = TrustRootBundle::new(
        TrustBundleIdentifier::parse("http_exchange_test:v1").expect("trust identifier"),
        vec![root_der],
    )
    .expect("trust roots");
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        alpn.iter().map(|value| value.to_vec()).collect(),
        requirement,
    )
    .expect("TLS policy");
    TlsHandshakePlan::new(origin.clone(), tcp, roots, policy)
        .expect("TLS plan")
        .authenticate()
        .expect("TLS authentication")
}

fn request(origin: Origin) -> HttpRequest {
    HttpRequest::new(
        HttpMethod::Get,
        origin,
        HttpRequestTarget::parse("/health").expect("request target"),
    )
    .expect("request")
}

#[test]
fn executes_one_real_authenticated_http_exchange_and_closes_reuse() {
    let material = server_material();
    let (address, server) = spawn_server(
        server_config(
            ServerMaterial {
                root_der: material.root_der.clone(),
                chain: material.chain,
                key: material.key,
            },
            &[b"http/1.1"],
        ),
        Some(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello"),
    );
    let origin = make_origin(address);
    let exchange = HttpExchange::execute(
        connection(
            &origin,
            address,
            material.root_der,
            &[b"http/1.1"],
            AlpnRequirement::Required,
        ),
        &request(origin.clone()),
        &HttpExchangePolicy::new(TEST_TIMEOUT, 128).expect("HTTP policy"),
    )
    .expect("bounded exchange");
    assert_eq!(exchange.response().status_code(), 200);
    assert_eq!(exchange.response().body(), b"hello");
    assert_eq!(exchange.tls_evidence().origin(), &origin);
    assert!(exchange.duration() <= TEST_TIMEOUT);
    let received = server
        .join()
        .expect("server thread")
        .expect("server result");
    assert!(received.starts_with(b"GET /health HTTP/1.1\r\n"));
    assert!(received.ends_with(b"Connection: close\r\n\r\n"));
}

#[test]
fn rejects_wrong_alpn_absence_and_origin_before_request_io() {
    let material = server_material();
    let (address, server) = spawn_server(
        server_config(
            ServerMaterial {
                root_der: material.root_der.clone(),
                chain: material.chain,
                key: material.key,
            },
            &[b"h2"],
        ),
        None,
    );
    let origin = make_origin(address);
    let error = HttpExchange::execute(
        connection(
            &origin,
            address,
            material.root_der,
            &[b"h2"],
            AlpnRequirement::Required,
        ),
        &request(origin),
        &HttpExchangePolicy::default(),
    )
    .expect_err("h2 must not enter HTTP/1.1");
    assert!(matches!(error, HttpError::UnexpectedAlpn));
    server
        .join()
        .expect("server thread")
        .expect("server result");

    let material = server_material();
    let (address, server) = spawn_server(
        server_config(
            ServerMaterial {
                root_der: material.root_der.clone(),
                chain: material.chain,
                key: material.key,
            },
            &[],
        ),
        None,
    );
    let origin = make_origin(address);
    let authenticated = connection(
        &origin,
        address,
        material.root_der,
        &[],
        AlpnRequirement::Optional,
    );
    let error = HttpExchange::execute(
        authenticated,
        &request(origin.clone()),
        &HttpExchangePolicy::default(),
    )
    .expect_err("absent ALPN requires explicit direct-test policy");
    assert!(matches!(error, HttpError::AbsentAlpnNotPermitted));
    server
        .join()
        .expect("server thread")
        .expect("server result");

    let material = server_material();
    let (address, server) = spawn_server(
        server_config(
            ServerMaterial {
                root_der: material.root_der.clone(),
                chain: material.chain,
                key: material.key,
            },
            &[b"http/1.1"],
        ),
        None,
    );
    let origin = make_origin(address);
    let other_origin = Origin::parse("https://other.test").expect("other HTTPS origin");
    let error = HttpExchange::execute(
        connection(
            &origin,
            address,
            material.root_der,
            &[b"http/1.1"],
            AlpnRequirement::Required,
        ),
        &request(other_origin),
        &HttpExchangePolicy::default(),
    )
    .expect_err("TLS and request origins must match");
    assert!(matches!(error, HttpError::RequestAuthorityMismatch));
    server
        .join()
        .expect("server thread")
        .expect("server result");
}

#[test]
fn permits_absent_alpn_only_with_explicit_direct_test_policy() {
    let material = server_material();
    let (address, server) = spawn_server(
        server_config(
            ServerMaterial {
                root_der: material.root_der.clone(),
                chain: material.chain,
                key: material.key,
            },
            &[],
        ),
        Some(b"HTTP/1.1 204 No Content\r\n\r\n"),
    );
    let origin = make_origin(address);
    let exchange = HttpExchange::execute(
        connection(
            &origin,
            address,
            material.root_der,
            &[],
            AlpnRequirement::Optional,
        ),
        &request(origin),
        &HttpExchangePolicy::new(TEST_TIMEOUT, 128)
            .expect("HTTP policy")
            .permit_absent_alpn(),
    )
    .expect("explicit absent-ALPN test policy");
    assert_eq!(exchange.response().status_code(), 204);
    server
        .join()
        .expect("server thread")
        .expect("server result");
}

fn parse_public(input: &[u8], method: HttpMethod) -> Result<HttpResponse, HttpError> {
    HttpResponse::parse(
        &mut Cursor::new(input),
        method,
        &HttpExchangePolicy::new(Duration::from_secs(1), 32).expect("parser policy"),
    )
}

struct FailingAfterReader {
    input: Cursor<Vec<u8>>,
}

impl Read for FailingAfterReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.input.read(buffer)?;
        if read == 0 {
            Err(io::Error::other("test I/O failure after bytes"))
        } else {
            Ok(read)
        }
    }
}

#[test]
fn public_parser_exercises_framing_limits_and_credential_free_errors() {
    let response = parse_public(
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Type: text/plain\r\nContent-Encoding: identity\r\nSet-Cookie: hidden\r\n\r\nhello",
        HttpMethod::Get,
    )
    .expect("fixed response");
    assert_eq!(response.reason_phrase(), b"OK");
    assert_eq!(response.body(), b"hello");
    assert!(response.is_complete());
    assert_eq!(response.headers()[0].value(), "5");
    assert!(
        response
            .headers()
            .iter()
            .any(|header| header.name() == "content-encoding")
    );

    for (method, status) in [
        (HttpMethod::Head, 200),
        (HttpMethod::Get, 101),
        (HttpMethod::Get, 204),
        (HttpMethod::Get, 304),
    ] {
        let input = format!("HTTP/1.1 {status} No Content\r\nContent-Length: 5\r\n\r\nhello");
        assert!(
            parse_public(input.as_bytes(), method)
                .expect("no-body response")
                .body()
                .is_empty()
        );
    }

    assert_eq!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\nX-Trace: safe\r\n\r\n",
            HttpMethod::Get,
        )
        .expect("chunked response")
        .body(),
        b"Wikipedia"
    );
    assert_eq!(
        parse_public(b"HTTP/1.1 200 OK\r\n\r\nclose", HttpMethod::Get)
            .expect("close response")
            .body(),
        b"close"
    );
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nBad Trailer\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::MalformedHeader)
    ));

    let malformed = [
        &b"HTTP/1.0 200 OK\r\n\r\n"[..],
        &b"HTTP/1.1 99 No\r\n\r\n"[..],
        &b"HTTP/1.1 A00 OK\r\n\r\n"[..],
        &b"HTTP/1.1 600 OK\r\n\r\n"[..],
        &b"HTTP/1.1 200 \x7f\r\n\r\n"[..],
        &b"HTTP/1.1 200\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\n\n"[..],
        &b"HTTP/1.1 200 OK\rX: y\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\n X: y\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nX y\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nBad Name: y\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nX: \x01\r\n\r\n"[..],
    ];
    for input in malformed {
        assert!(matches!(
            parse_public(input, HttpMethod::Get),
            Err(HttpError::MalformedStatusLine | HttpError::MalformedHeader)
        ));
    }
    assert!(parse_public(b"HTTP/1.1 200 OK\r\nX-Tab: a\tb\r\n\r\n", HttpMethod::Get,).is_ok());
    assert!(matches!(
        parse_public(b"HTTP/1.1 200 OK\r\n: value\r\n\r\n", HttpMethod::Get),
        Err(HttpError::MalformedHeader)
    ));
    let mut oversized_name = b"HTTP/1.1 200 OK\r\n".to_vec();
    oversized_name.extend(std::iter::repeat_n(b'X', MAX_HEADER_NAME_BYTES + 1));
    oversized_name.extend_from_slice(b": value\r\n\r\n");
    assert!(matches!(
        parse_public(&oversized_name, HttpMethod::Get),
        Err(HttpError::MalformedHeader)
    ));
    let mut oversized_value = b"HTTP/1.1 200 OK\r\nX: ".to_vec();
    oversized_value.extend(std::iter::repeat_n(b'a', MAX_HEADER_VALUE_BYTES + 1));
    oversized_value.extend_from_slice(b"\r\n\r\n");
    assert!(matches!(
        parse_public(&oversized_value, HttpMethod::Get),
        Err(HttpError::MalformedHeader)
    ));
    assert!(matches!(
        parse_public(b"", HttpMethod::Get),
        Err(HttpError::IncompleteResponse)
    ));
    assert!(matches!(
        parse_public(b"HTTP/1.1 200 OK\r", HttpMethod::Get),
        Err(HttpError::IncompleteResponse)
    ));
    assert!(matches!(
        parse_public(b"HTTP/1.1 200 OK\r\nX: y\r", HttpMethod::Get),
        Err(HttpError::IncompleteResponse)
    ));

    for input in [
        &b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\na"[..],
        &b"HTTP/1.1 200 OK\r\nContent-Length: one\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nContent-Length:\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nContent-Length: 999999999999999999999999999999999999999999\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n\r\n"[..],
        &b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\n\r\n"[..],
    ] {
        assert!(parse_public(input, HttpMethod::Get).is_err());
    }
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 302 Found\r\nLocation: https://other.test/\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::RedirectNotSupported)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nno",
            HttpMethod::Get,
        ),
        Err(HttpError::IncompleteResponse)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nContent-Length: 33\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::BodyLimitExceeded)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3;bad\r\nabc\r\n0\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::MalformedChunk)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabcX0\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::MalformedChunk)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::MalformedChunk)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nz\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::MalformedChunk)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n21\r\n",
            HttpMethod::Get,
        ),
        Err(HttpError::BodyLimitExceeded)
    ));
    let cumulative_policy =
        HttpExchangePolicy::new(Duration::from_secs(1), 4).expect("cumulative policy");
    assert!(matches!(
        HttpResponse::parse(
            &mut Cursor::new(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n"
                    .to_vec(),
            ),
            HttpMethod::Get,
            &cumulative_policy,
        ),
        Err(HttpError::BodyLimitExceeded)
    ));
    let mut overflowing_chunk = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
    overflowing_chunk.extend(std::iter::repeat_n(b'f', 128));
    overflowing_chunk.extend_from_slice(b"\r\n");
    assert!(matches!(
        parse_public(&overflowing_chunk, HttpMethod::Get),
        Err(HttpError::BodyLimitExceeded)
    ));
    assert!(matches!(
        parse_public(
            b"HTTP/1.1 200 OK\r\n\r\n123456789012345678901234567890123",
            HttpMethod::Get,
        ),
        Err(HttpError::BodyLimitExceeded)
    ));

    let mut many_headers = b"HTTP/1.1 200 OK\r\n".to_vec();
    for _ in 0..=MAX_HEADER_FIELDS {
        many_headers.extend_from_slice(b"X-Test: y\r\n");
    }
    many_headers.extend_from_slice(b"\r\n");
    assert!(matches!(
        parse_public(&many_headers, HttpMethod::Get),
        Err(HttpError::HeaderFieldLimitExceeded)
    ));

    let mut large_line = b"HTTP/1.1 200 OK\r\nX: ".to_vec();
    large_line.extend(std::iter::repeat_n(
        b'a',
        MAX_HEADER_VALUE_BYTES + MAX_HEADER_NAME_BYTES + 5,
    ));
    large_line.extend_from_slice(b"\r\n\r\n");
    assert!(matches!(
        parse_public(&large_line, HttpMethod::Get),
        Err(HttpError::HeaderLineLimitExceeded)
    ));

    let mut large_section = b"HTTP/1.1 200 OK\r\n".to_vec();
    for _ in 0..4 {
        large_section.extend_from_slice(b"X: ");
        large_section.extend(std::iter::repeat_n(b'a', MAX_HEADER_VALUE_BYTES));
        large_section.extend_from_slice(b"\r\n");
    }
    large_section.extend_from_slice(b"\r\n");
    assert!(matches!(
        parse_public(&large_section, HttpMethod::Get),
        Err(HttpError::HeaderSectionLimitExceeded)
    ));

    let wide_policy =
        HttpExchangePolicy::new(Duration::from_secs(1), MAX_HTTP_BODY_BYTES).expect("wide policy");
    let mut chunks = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
    for _ in 0..=MAX_CHUNK_COUNT {
        chunks.extend_from_slice(b"1\r\na\r\n");
    }
    chunks.extend_from_slice(b"0\r\n\r\n");
    assert!(matches!(
        HttpResponse::parse(&mut Cursor::new(chunks), HttpMethod::Get, &wide_policy),
        Err(HttpError::ChunkLimitExceeded)
    ));

    let mut trailers = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX: ".to_vec();
    trailers.extend(std::iter::repeat_n(b'a', MAX_TRAILER_SECTION_BYTES));
    trailers.extend_from_slice(b"\r\n\r\n");
    assert!(matches!(
        parse_public(&trailers, HttpMethod::Get),
        Err(HttpError::TrailerSectionLimitExceeded | HttpError::HeaderLineLimitExceeded)
    ));

    let mut trailer_count = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n".to_vec();
    for _ in 0..=MAX_TRAILER_FIELDS {
        trailer_count.extend_from_slice(b"X: y\r\n");
    }
    trailer_count.extend_from_slice(b"\r\n");
    assert!(matches!(
        parse_public(&trailer_count, HttpMethod::Get),
        Err(HttpError::TrailerLimitExceeded)
    ));

    struct TimeoutReader;
    impl Read for TimeoutReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::TimedOut, "timed out"))
        }
    }
    assert!(matches!(
        HttpResponse::parse(
            &mut TimeoutReader,
            HttpMethod::Get,
            &HttpExchangePolicy::default(),
        ),
        Err(HttpError::ExchangeTimedOut)
    ));
    for input in [
        b"HTTP/1.1 200 OK\r".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nab".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabcX".to_vec(),
    ] {
        let error = HttpResponse::parse(
            &mut FailingAfterReader {
                input: Cursor::new(input),
            },
            HttpMethod::Get,
            &HttpExchangePolicy::default(),
        )
        .expect_err("partial response I/O failure");
        assert!(matches!(error, HttpError::Io { .. }));
    }

    for error in [
        HttpError::RequestAuthorityMismatch,
        HttpError::UnexpectedAlpn,
        HttpError::AbsentAlpnNotPermitted,
        HttpError::HeaderLineLimitExceeded,
        HttpError::HeaderSectionLimitExceeded,
        HttpError::HeaderFieldLimitExceeded,
        HttpError::MalformedStatusLine,
        HttpError::MalformedHeader,
        HttpError::FramingAmbiguous,
        HttpError::DuplicateContentLength,
        HttpError::InvalidContentLength,
        HttpError::UnsupportedTransferCoding,
        HttpError::UnsupportedContentCoding,
        HttpError::MalformedChunk,
        HttpError::ChunkLimitExceeded,
        HttpError::TrailerLimitExceeded,
        HttpError::TrailerSectionLimitExceeded,
        HttpError::RedirectNotSupported,
        HttpError::BodyLimitExceeded,
        HttpError::IncompleteResponse,
        HttpError::ExchangeTimedOut,
        HttpError::Io {
            operation: "test",
            source: io::Error::other("test"),
        },
        HttpError::TimeoutConfiguration {
            operation: "test",
            source: io::Error::other("test"),
        },
    ] {
        assert!(!error.to_string().is_empty());
        let _ = error.source();
    }
    for error in [
        HttpRequestError::InsecureOrigin,
        HttpRequestError::InvalidRequestTarget,
    ] {
        assert!(!error.to_string().is_empty());
    }
    for policy_error in [
        HttpExchangePolicy::new(Duration::ZERO, 1).expect_err("zero timeout"),
        HttpExchangePolicy::new(Duration::from_secs(1), 0).expect_err("zero body"),
    ] {
        assert!(!policy_error.to_string().is_empty());
    }
    assert_eq!(
        HttpRequestTarget::parse("/public")
            .expect("target")
            .as_str(),
        "/public"
    );
    assert!(
        HttpRequestTarget::parse("").is_err()
            && HttpRequestTarget::parse("relative").is_err()
            && HttpRequestTarget::parse("/bad\n").is_err()
            && HttpRequestTarget::parse("//absolute").is_err()
            && HttpRequestTarget::parse("/fragment#bad").is_err()
    );
    assert!(HttpRequestTarget::parse(&format!("/{}", "a".repeat(8192))).is_err());
    let head = HttpRequest::new(
        HttpMethod::Head,
        Origin::parse("https://example.test").expect("head origin"),
        HttpRequestTarget::parse("/").expect("head target"),
    )
    .expect("head request");
    assert_eq!(head.target().as_str(), "/");
    assert!(head.serialize().starts_with(b"HEAD / HTTP/1.1\r\n"));
    assert!(
        HttpRequest::new(
            HttpMethod::Get,
            Origin::parse("http://127.0.0.1").expect("loopback origin"),
            HttpRequestTarget::parse("/").expect("target"),
        )
        .is_err()
    );
    assert!(
        HttpExchangePolicy::new(Duration::from_secs(31), 1).is_err()
            && HttpExchangePolicy::new(Duration::from_secs(1), MAX_HTTP_BODY_BYTES + 1).is_err()
    );
}
