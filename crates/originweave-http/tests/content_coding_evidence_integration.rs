use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{
    AuthenticatedHttpResponse, ContentCodingLayerEvidence, ContentCodingName,
    ContentDecoderOutcome, HttpClientPolicy, HttpExchangePlan, HttpMethod, HttpRequestTarget,
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

/// Creates the isolated trust anchor used only by this authenticated loopback fixture.
fn certificate_authority() -> Result<(Vec<u8>, Issuer<'static, KeyPair>), String> {
    let mut parameters = CertificateParams::new(Vec::new())
        .map_err(|error| format!("empty CA SAN list rejected: {error:?}"))?;
    parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    parameters.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    parameters.distinguished_name.push(
        DnType::CommonName,
        "OriginWeave HTTP content-coding evidence test root",
    );
    let key_pair = KeyPair::generate().map_err(|error| format!("CA key generation: {error:?}"))?;
    let certificate = parameters
        .self_signed(&key_pair)
        .map_err(|error| format!("CA certificate generation: {error:?}"))?;
    Ok((
        certificate.der().to_vec(),
        Issuer::new(parameters, key_pair),
    ))
}

/// Issues a localhost leaf certificate valid at the fixture's pinned trusted time.
fn certificate_material() -> Result<CertificateMaterial, String> {
    let (root_der, issuer) = certificate_authority()?;
    let mut parameters = CertificateParams::new(vec!["localhost".to_owned()])
        .map_err(|error| format!("localhost SAN rejected: {error:?}"))?;
    parameters.not_before = rcgen::date_time_ymd(2025, 1, 1);
    parameters.not_after = rcgen::date_time_ymd(2030, 1, 1);
    parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    parameters.use_authority_key_identifier_extension = true;
    let key_pair =
        KeyPair::generate().map_err(|error| format!("leaf key generation: {error:?}"))?;
    let certificate: Certificate = parameters
        .signed_by(&key_pair, &issuer)
        .map_err(|error| format!("leaf certificate generation: {error:?}"))?;
    Ok(CertificateMaterial {
        root_der,
        certificate_chain: vec![certificate.der().clone()],
        private_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der())),
    })
}

/// Restricts the fixture server to the HTTP/1.1 ALPN contract consumed by OriginWeave.
fn server_config(material: CertificateMaterial) -> Result<(Vec<u8>, Arc<ServerConfig>), String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .map_err(|error| format!("test protocol versions: {error:?}"))?;
    let mut config = builder
        .with_no_client_auth()
        .with_single_cert(material.certificate_chain, material.private_key)
        .map_err(|error| format!("test certificate and key mismatch: {error:?}"))?;
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok((material.root_der, Arc::new(config)))
}

/// Serves one bounded TLS response and returns the request head for negotiation evidence.
fn spawn_http_server(
    config: Arc<ServerConfig>,
    response: Vec<u8>,
) -> Result<(SocketAddr, JoinHandle<ServerResult>), String> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| format!("loopback listener bind: {error:?}"))?;
    let socket_address = listener
        .local_addr()
        .map_err(|error| format!("loopback listener address: {error:?}"))?;
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
                Err(error) => return Err(error.to_string()),
            }
        }
        tls.write_all(&response)
            .map_err(|error| error.to_string())?;
        tls.flush().map_err(|error| error.to_string())?;
        tls.conn.send_close_notify();
        let _ = tls.flush();
        Ok(request)
    });
    Ok((socket_address, handle))
}

/// Binds the request origin to the fixture's dynamically allocated loopback port.
fn origin_for(socket_address: SocketAddr) -> Result<Origin, String> {
    Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .map_err(|error| format!("test origin rejected: {error:?}"))
}

/// Authorizes only the fixture's loopback address before opening the TCP connection.
fn direct_connection(
    origin: &Origin,
    socket_address: SocketAddr,
) -> Result<DirectTcpConnection, String> {
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .map_err(|error| format!("managed loopback resolution rejected: {error:?}"))?;
    let plan = ConnectionPlan::new(&snapshot, socket_address, Duration::from_secs(2), 1)
        .map_err(|error| format!("direct connection plan rejected: {error:?}"))?;
    plan.connect()
        .map_err(|error| format!("loopback TCP connection failed: {error:?}"))
}

/// Authenticates the fixture peer against only its generated root and requires HTTP/1.1 ALPN.
fn authenticated_connection(
    origin: &Origin,
    socket_address: SocketAddr,
    root_der: Vec<u8>,
) -> Result<originweave_tls::AuthenticatedTlsConnection, String> {
    let trust_identifier = TrustBundleIdentifier::parse("http_content_coding_evidence_loopback:v1")
        .map_err(|error| format!("trust identifier rejected: {error:?}"))?;
    let roots = TrustRootBundle::new(trust_identifier, vec![root_der])
        .map_err(|error| format!("test root bundle rejected: {error:?}"))?;
    let policy = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
        TEST_TIMEOUT,
        vec![b"http/1.1".to_vec()],
        AlpnRequirement::Required,
    )
    .map_err(|error| format!("TLS client policy rejected: {error:?}"))?;
    let plan = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(origin, socket_address)?,
        roots,
        policy,
    )
    .map_err(|error| format!("TLS handshake plan rejected: {error:?}"))?;
    plan.authenticate()
        .map_err(|error| format!("authenticated loopback TLS failed: {error:?}"))
}

/// Applies one RFC 1952 gzip layer.
fn gzip(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|error| format!("gzip input: {error:?}"))?;
    encoder
        .finish()
        .map_err(|error| format!("gzip finish: {error:?}"))
}

/// Applies the standards-defined zlib-wrapped deflate content coding.
fn zlib_deflate(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|error| format!("deflate input: {error:?}"))?;
    encoder
        .finish()
        .map_err(|error| format!("deflate finish: {error:?}"))
}

/// Applies raw RFC 1951 DEFLATE for the bounded non-conforming-peer compatibility cases.
fn raw_deflate(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|error| format!("raw deflate input: {error:?}"))?;
    encoder
        .finish()
        .map_err(|error| format!("raw deflate finish: {error:?}"))
}

/// Executes one authenticated exchange while retaining both response content and audit evidence.
fn execute_evidence_fixture(
    path: &str,
    content_encoding: &str,
    wire_body: &[u8],
) -> Result<AuthenticatedHttpResponse, String> {
    let mut wire_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Encoding: {content_encoding}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
        wire_body.len()
    )
    .into_bytes();
    wire_response.extend_from_slice(wire_body);

    let material = certificate_material()?;
    let (root_der, config) = server_config(material)?;
    let (socket_address, server) = spawn_http_server(config, wire_response)?;
    let origin = origin_for(socket_address)?;
    let connection = authenticated_connection(&origin, socket_address, root_der)?;
    let target = HttpRequestTarget::parse(origin, path)
        .map_err(|error| format!("request target rejected: {error:?}"))?;

    let execution = HttpExchangePlan::new(
        connection,
        HttpMethod::Get,
        target,
        &[],
        HttpClientPolicy::strict_defaults(),
    )
    .map_err(|error| format!("HTTP plan construction failed: {error:?}"))?
    .execute();

    let request = server
        .join()
        .map_err(|_panic| "server thread panicked".to_owned())??;
    let expected_request_line = format!("GET {path} HTTP/1.1\r\n");
    if !request.starts_with(expected_request_line.as_bytes()) {
        return Err("fixture did not deliver the expected request to the TLS peer".to_owned());
    }
    if !request
        .windows(b"\r\nAccept-Encoding: gzip, deflate\r\n".len())
        .any(|window| window == b"\r\nAccept-Encoding: gzip, deflate\r\n")
    {
        return Err("fixture did not observe the advertised gzip/deflate codings".to_owned());
    }

    execution.map_err(|error| format!("authenticated HTTP exchange failed: {error:?}"))
}

/// Requires one exact wire-order evidence layer and its actual decoder outcome.
fn require_layer(
    coding: originweave_http::ContentCoding,
    index: usize,
    expected_name: ContentCodingName,
    expected_outcome: ContentDecoderOutcome,
) -> Result<(), String> {
    let Some(layer): Option<ContentCodingLayerEvidence> = coding.layer(index) else {
        return Err(format!("missing content-coding evidence layer at index {index}"));
    };
    if layer.coding() != expected_name || layer.outcome() != expected_outcome {
        return Err(format!(
            "content-coding evidence layer {index} was {:?}/{:?}, expected {expected_name:?}/{expected_outcome:?}",
            layer.coding(),
            layer.outcome()
        ));
    }
    Ok(())
}

/// Proves standard stacked decoders retain the exact declared wire order in exchange evidence.
#[test]
fn authenticated_tls_exchange_retains_standard_stack_evidence_in_wire_order() -> Result<(), String> {
    let original = b"evidence for a standards-valid gzip then deflate representation";
    let gzip_first = gzip(original)?;
    let wire_body = zlib_deflate(&gzip_first)?;
    let response = execute_evidence_fixture(
        "/content-coding-evidence-standard",
        "gzip, deflate",
        &wire_body,
    )?;

    if response.content() != original {
        return Err("standard stacked evidence fixture returned the wrong content".to_owned());
    }
    let coding = response.evidence().content_coding();
    if coding.layer_count() != 2 {
        return Err(format!(
            "standard stacked evidence retained {} layers instead of 2",
            coding.layer_count()
        ));
    }
    require_layer(
        coding,
        0,
        ContentCodingName::Gzip,
        ContentDecoderOutcome::Standard,
    )?;
    require_layer(
        coding,
        1,
        ContentCodingName::Deflate,
        ContentDecoderOutcome::Standard,
    )
}

/// Proves raw-DEFLATE fallback is attributed only to the outer declared deflate layer.
#[test]
fn authenticated_tls_exchange_retains_outer_raw_deflate_outcome() -> Result<(), String> {
    let original = b"evidence for raw deflate compatibility in the outer layer";
    let gzip_first = gzip(original)?;
    let wire_body = raw_deflate(&gzip_first)?;
    let response = execute_evidence_fixture(
        "/content-coding-evidence-raw-outer",
        "gzip, deflate",
        &wire_body,
    )?;

    if response.content() != original {
        return Err("outer raw-DEFLATE evidence fixture returned the wrong content".to_owned());
    }
    let coding = response.evidence().content_coding();
    require_layer(
        coding,
        0,
        ContentCodingName::Gzip,
        ContentDecoderOutcome::Standard,
    )?;
    require_layer(
        coding,
        1,
        ContentCodingName::Deflate,
        ContentDecoderOutcome::RawDeflateCompatibility,
    )
}

/// Proves reverse decoding does not reverse evidence when raw compatibility occurs on the inner layer.
#[test]
fn authenticated_tls_exchange_retains_inner_raw_deflate_outcome_in_wire_order()
-> Result<(), String> {
    let original = b"evidence for raw deflate compatibility in the inner layer";
    let raw_deflate_first = raw_deflate(original)?;
    let wire_body = gzip(&raw_deflate_first)?;
    let response = execute_evidence_fixture(
        "/content-coding-evidence-raw-inner",
        "deflate, gzip",
        &wire_body,
    )?;

    if response.content() != original {
        return Err("inner raw-DEFLATE evidence fixture returned the wrong content".to_owned());
    }
    let coding = response.evidence().content_coding();
    require_layer(
        coding,
        0,
        ContentCodingName::Deflate,
        ContentDecoderOutcome::RawDeflateCompatibility,
    )?;
    require_layer(
        coding,
        1,
        ContentCodingName::Gzip,
        ContentDecoderOutcome::Standard,
    )
}
