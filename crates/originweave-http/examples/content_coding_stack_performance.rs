use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder};
use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_http::{HttpClientPolicy, HttpExchangePlan, HttpMethod, HttpRequestTarget};
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
const SAMPLE_COUNT: usize = 31;
const P95_BUDGET_MICROSECONDS: u128 = 20_000;
const SEED_BLOCK_BYTES: usize = 64 * 1024;

type ServerResult = Result<Vec<u8>, String>;

struct CertificateMaterial {
    root_der: Vec<u8>,
    certificate_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

struct PerformanceProfile {
    name: &'static str,
    content_encoding: &'static str,
    decoded_bytes: usize,
    wire_body: Vec<u8>,
    expected_content: Vec<u8>,
}

struct PerformanceReceipt {
    profile: &'static str,
    decoded_bytes: usize,
    coded_bytes: usize,
    samples: usize,
    p50_microseconds: u128,
    p95_microseconds: u128,
    maximum_microseconds: u128,
}

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
        "OriginWeave HTTP content-coding performance root",
    );
    let key_pair = KeyPair::generate().map_err(|error| format!("CA key generation: {error:?}"))?;
    let certificate = parameters
        .self_signed(&key_pair)
        .map_err(|error| format!("CA certificate generation: {error:?}"))?;
    Ok((certificate.der().to_vec(), Issuer::new(parameters, key_pair)))
}

fn certificate_material() -> Result<CertificateMaterial, String> {
    let (root_der, issuer) = certificate_authority()?;
    let mut parameters = CertificateParams::new(vec!["localhost".to_owned()])
        .map_err(|error| format!("localhost SAN rejected: {error:?}"))?;
    parameters.not_before = rcgen::date_time_ymd(2025, 1, 1);
    parameters.not_after = rcgen::date_time_ymd(2030, 1, 1);
    parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    parameters.use_authority_key_identifier_extension = true;
    let key_pair = KeyPair::generate().map_err(|error| format!("leaf key generation: {error:?}"))?;
    let certificate: Certificate = parameters
        .signed_by(&key_pair, &issuer)
        .map_err(|error| format!("leaf certificate generation: {error:?}"))?;
    Ok(CertificateMaterial {
        root_der,
        certificate_chain: vec![certificate.der().clone()],
        private_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der())),
    })
}

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
        tls.write_all(&response).map_err(|error| error.to_string())?;
        tls.flush().map_err(|error| error.to_string())?;
        tls.conn.send_close_notify();
        let _ = tls.flush();
        Ok(request)
    });
    Ok((socket_address, handle))
}

fn origin_for(socket_address: SocketAddr) -> Result<Origin, String> {
    Origin::parse(&format!("https://localhost:{}", socket_address.port()))
        .map_err(|error| format!("test origin rejected: {error:?}"))
}

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

fn authenticated_connection(
    origin: &Origin,
    socket_address: SocketAddr,
    root_der: Vec<u8>,
) -> Result<originweave_tls::AuthenticatedTlsConnection, String> {
    let trust_identifier = TrustBundleIdentifier::parse("http_content_coding_performance:v1")
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

fn deterministic_payload(byte_count: usize) -> Vec<u8> {
    let mut seed_block = Vec::with_capacity(SEED_BLOCK_BYTES);
    let mut state = 0x9e37_79b9_u32;
    while seed_block.len() < SEED_BLOCK_BYTES {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed_block.extend_from_slice(&state.to_le_bytes());
    }
    seed_block.truncate(SEED_BLOCK_BYTES);

    let mut payload = Vec::with_capacity(byte_count);
    while payload.len() < byte_count {
        let remaining = byte_count - payload.len();
        let count = remaining.min(seed_block.len());
        payload.extend_from_slice(&seed_block[..count]);
    }
    payload
}

fn gzip(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|error| format!("gzip input: {error:?}"))?;
    encoder
        .finish()
        .map_err(|error| format!("gzip finish: {error:?}"))
}

fn zlib_deflate(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|error| format!("deflate input: {error:?}"))?;
    encoder
        .finish()
        .map_err(|error| format!("deflate finish: {error:?}"))
}

fn profile(
    name: &'static str,
    content_encoding: &'static str,
    decoded_bytes: usize,
) -> Result<PerformanceProfile, String> {
    let expected_content = deterministic_payload(decoded_bytes);
    let wire_body = match content_encoding {
        "gzip, deflate" => zlib_deflate(&gzip(&expected_content)?)?,
        "deflate, gzip" => gzip(&zlib_deflate(&expected_content)?)?,
        _ => return Err("unsupported benchmark profile coding order".to_owned()),
    };
    Ok(PerformanceProfile {
        name,
        content_encoding,
        decoded_bytes,
        wire_body,
        expected_content,
    })
}

fn wire_response(profile: &PerformanceProfile) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Encoding: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
        profile.wire_body.len(), profile.content_encoding
    )
    .into_bytes();
    response.extend_from_slice(&profile.wire_body);
    response
}

fn require_request_contract(request: &[u8], path: &str) -> Result<(), String> {
    let expected_request_line = format!("GET {path} HTTP/1.1\r\n");
    if !request.starts_with(expected_request_line.as_bytes()) {
        return Err("performance fixture did not receive the expected request line".to_owned());
    }
    if !request
        .windows(b"\r\nAccept-Encoding: gzip, deflate\r\n".len())
        .any(|window| window == b"\r\nAccept-Encoding: gzip, deflate\r\n")
    {
        return Err("performance fixture did not observe the governed coding advertisement".to_owned());
    }
    Ok(())
}

fn percentile(sorted_microseconds: &[u128], numerator: usize) -> Result<u128, String> {
    if sorted_microseconds.is_empty() || numerator == 0 || numerator > 100 {
        return Err("invalid percentile request".to_owned());
    }
    let rank = sorted_microseconds.len().saturating_mul(numerator).div_ceil(100);
    let index = rank.saturating_sub(1);
    sorted_microseconds
        .get(index)
        .copied()
        .ok_or_else(|| "percentile index escaped the sample set".to_owned())
}

fn measure_profile(
    root_der: &[u8],
    config: &Arc<ServerConfig>,
    profile: &PerformanceProfile,
) -> Result<PerformanceReceipt, String> {
    let response = wire_response(profile);
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample_index in 0..SAMPLE_COUNT {
        let (socket_address, server) = spawn_http_server(Arc::clone(config), response.clone())?;
        let origin = origin_for(socket_address)?;
        let connection = authenticated_connection(&origin, socket_address, root_der.to_vec())?;
        let path = format!("/content-coding-performance/{}/{}", profile.name, sample_index);
        let target = HttpRequestTarget::parse(origin, &path)
            .map_err(|error| format!("request target rejected: {error:?}"))?;
        let plan = HttpExchangePlan::new(
            connection,
            HttpMethod::Get,
            target,
            &[],
            HttpClientPolicy::strict_defaults(),
        )
        .map_err(|error| format!("HTTP plan construction failed: {error:?}"))?;

        // Start after TLS authentication so this receipt measures the governed HTTP exchange,
        // response parsing and stacked decoding path rather than certificate setup/handshake cost.
        let started = Instant::now();
        let response = plan
            .execute()
            .map_err(|error| format!("HTTP performance exchange failed: {error:?}"))?;
        let elapsed = started.elapsed();

        if response.content() != profile.expected_content {
            return Err(format!(
                "{} returned the wrong decoded payload on sample {sample_index}",
                profile.name
            ));
        }
        let request = server
            .join()
            .map_err(|_panic| "performance server thread panicked".to_owned())??;
        require_request_contract(&request, &path)?;
        samples.push(elapsed.as_micros());
    }

    samples.sort_unstable();
    let p50_microseconds = percentile(&samples, 50)?;
    let p95_microseconds = percentile(&samples, 95)?;
    let maximum_microseconds = samples
        .last()
        .copied()
        .ok_or_else(|| "performance sample set is empty".to_owned())?;

    Ok(PerformanceReceipt {
        profile: profile.name,
        decoded_bytes: profile.decoded_bytes,
        coded_bytes: profile.wire_body.len(),
        samples: samples.len(),
        p50_microseconds,
        p95_microseconds,
        maximum_microseconds,
    })
}

fn write_receipt(receipt: &PerformanceReceipt) -> Result<(), String> {
    let status = if receipt.p95_microseconds <= P95_BUDGET_MICROSECONDS {
        "PASS"
    } else {
        "FAIL"
    };
    let line = format!(
        "profile={} decoded_bytes={} coded_bytes={} samples={} p50_us={} p95_us={} max_us={} budget_us={} status={}\n",
        receipt.profile,
        receipt.decoded_bytes,
        receipt.coded_bytes,
        receipt.samples,
        receipt.p50_microseconds,
        receipt.p95_microseconds,
        receipt.maximum_microseconds,
        P95_BUDGET_MICROSECONDS,
        status
    );
    std::io::stdout()
        .lock()
        .write_all(line.as_bytes())
        .map_err(|error| format!("performance receipt write failed: {error:?}"))
}

fn main() -> Result<(), String> {
    let material = certificate_material()?;
    let (root_der, config) = server_config(material)?;
    let profiles = [
        profile("gzip-deflate-256k", "gzip, deflate", 256 * 1024)?,
        profile("deflate-gzip-1m", "deflate, gzip", 1024 * 1024)?,
    ];

    let mut p95_failure = false;
    for current in &profiles {
        let receipt = measure_profile(&root_der, &config, current)?;
        write_receipt(&receipt)?;
        if receipt.p95_microseconds > P95_BUDGET_MICROSECONDS {
            p95_failure = true;
        }
    }

    if p95_failure {
        return Err(format!(
            "stacked content-coding buyer path exceeded the {} us p95 budget",
            P95_BUDGET_MICROSECONDS
        ));
    }
    Ok(())
}
