use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use originweave_core::Origin;
use originweave_network::{DirectTcpConnection, SocketConnectionEvidence};
use rustls::client::Resumption;
use rustls::pki_types::{CertificateDer, UnixTime};
use rustls::time_provider::TimeProvider;
use rustls::{
    AlertDescription, CertificateError, ClientConfig, ClientConnection, ProtocolVersion,
    StreamOwned, SupportedCipherSuite,
};
use sha2::{Digest, Sha256};
use x509_parser::parse_x509_certificate;

use crate::evidence::{AuthenticatedTlsConnection, EvidenceInput};
use crate::policy::{MAX_SERVER_CERTIFICATE_BYTES, MAX_SERVER_CERTIFICATE_COUNT};
use crate::trust::sha256_identifier;
use crate::{
    AlpnRequirement, NegotiatedAlpn, TlsClientPolicy, TlsError, TlsProtocolVersion,
    TlsReferenceIdentity, TrustRootBundle,
};

/// A single-use authority to authenticate one HTTPS origin over a verified TCP stream.
#[derive(Debug)]
pub struct TlsHandshakePlan {
    origin: Origin,
    connection: DirectTcpConnection,
    trust_roots: TrustRootBundle,
    policy: TlsClientPolicy,
    reference_identity: TlsReferenceIdentity,
}

impl TlsHandshakePlan {
    /// Validate one TLS service-identity request without emitting TLS bytes.
    pub fn new(
        origin: Origin,
        connection: DirectTcpConnection,
        trust_roots: TrustRootBundle,
        policy: TlsClientPolicy,
    ) -> Result<Self, TlsError> {
        let reference_identity = TlsReferenceIdentity::from_origin(&origin)?;
        let evidence = connection.evidence();
        if evidence.origin() != &origin {
            return Err(TlsError::TransportOriginMismatch {
                tls_origin: origin,
                transport_origin: evidence.origin().clone(),
            });
        }
        verify_peer_evidence(connection.stream(), evidence)?;
        Ok(Self {
            origin,
            connection,
            trust_roots,
            policy,
            reference_identity,
        })
    }

    /// Perform a fixed-time, deadline-bound WebPKI handshake on the existing stream.
    pub fn authenticate(self) -> Result<AuthenticatedTlsConnection, TlsError> {
        let Self {
            origin,
            connection,
            trust_roots,
            policy,
            reference_identity,
        } = self;
        let (mut stream, network_evidence) = connection.into_parts();
        verify_peer_evidence(&stream, &network_evidence)?;

        let (trusted_time, handshake_timeout, alpn_protocols, alpn_requirement) =
            policy.into_parts();
        let (
            trust_bundle_identifier,
            root_store,
            trust_bundle_hash,
            trust_root_count,
            trust_root_bytes,
        ) = trust_roots.into_parts();
        let server_name = reference_identity.server_name(&origin)?;
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let time_provider = Arc::new(FixedTimeProvider { trusted_time });
        let builder = ClientConfig::builder_with_details(provider, time_provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(tls_configuration_error)?;
        let mut config = builder
            .with_root_certificates(root_store)
            .with_no_client_auth();
        config.resumption = Resumption::disabled();
        config.enable_early_data = false;
        config.enable_secret_extraction = false;
        config.key_log = Arc::new(rustls::NoKeyLog {});
        config.alpn_protocols = alpn_protocols.clone();
        config.enable_sni = reference_identity.uses_sni();
        config.check_selected_alpn = true;
        config.cert_compressors.clear();
        config.cert_decompressors.clear();

        let error_mapper = rustls_error_mapper(alpn_requirement);
        let mut client = ClientConnection::new(Arc::new(config), server_name)
            .map_err(error_mapper)?;
        let original_read_timeout = handshake_timeout_query(stream.read_timeout())?;
        let original_write_timeout = handshake_timeout_query(stream.write_timeout())?;
        let started_at = Instant::now();
        let deadline = started_at + handshake_timeout;

        let handshake_result = drive_handshake(
            &mut client,
            &mut stream,
            &network_evidence,
            deadline,
            handshake_timeout,
            error_mapper,
        );
        if let Err(error) = handshake_result {
            let _read_restore = stream.set_read_timeout(original_read_timeout);
            let _write_restore = stream.set_write_timeout(original_write_timeout);
            return Err(error);
        }

        restore_timeouts(&stream, original_read_timeout, original_write_timeout)?;
        verify_peer_evidence(&stream, &network_evidence)?;
        let handshake_duration = started_at.elapsed();
        let evidence = build_evidence(
            &origin,
            &network_evidence,
            &reference_identity,
            &client,
            &alpn_protocols,
            alpn_requirement,
            trust_bundle_identifier,
            trust_bundle_hash,
            trust_root_count,
            trust_root_bytes,
            trusted_time,
            handshake_duration,
            handshake_timeout,
        )?;

        Ok(AuthenticatedTlsConnection {
            stream: StreamOwned::new(client, stream),
            evidence,
        })
    }
}

#[derive(Debug)]
struct FixedTimeProvider {
    trusted_time: UnixTime,
}

impl TimeProvider for FixedTimeProvider {
    fn current_time(&self) -> Option<UnixTime> {
        Some(self.trusted_time)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandshakeIo {
    Write,
    Read,
}

type RustlsErrorMapper = fn(rustls::Error) -> TlsError;

#[derive(Debug, PartialEq, Eq)]
struct LeafCertificateEvidence {
    spki_hash: String,
    not_before_unix_seconds: i64,
    not_after_unix_seconds: i64,
}

fn drive_handshake(
    client: &mut ClientConnection,
    stream: &mut TcpStream,
    network_evidence: &SocketConnectionEvidence,
    deadline: Instant,
    handshake_timeout: Duration,
    error_mapper: RustlsErrorMapper,
) -> Result<(), TlsError> {
    while client.is_handshaking() {
        verify_peer_evidence(stream, network_evidence)?;
        for action in handshake_actions(client.wants_write(), client.wants_read())?
            .into_iter()
            .flatten()
        {
            perform_handshake_io(
                action,
                client,
                stream,
                deadline,
                handshake_timeout,
                error_mapper,
            )?;
        }
        enforce_handshake_deadline(
            Instant::now(),
            deadline,
            client.is_handshaking(),
            handshake_timeout,
        )?;
    }
    Ok(())
}

#[inline(never)]
fn handshake_actions(
    wants_write: bool,
    wants_read: bool,
) -> Result<[Option<HandshakeIo>; 2], TlsError> {
    match (wants_write, wants_read) {
        (true, true) => Ok([Some(HandshakeIo::Write), Some(HandshakeIo::Read)]),
        (true, false) => Ok([Some(HandshakeIo::Write), None]),
        (false, true) => Ok([Some(HandshakeIo::Read), None]),
        (false, false) => Err(TlsError::HandshakeIoFailed {
            source: io::Error::new(
                io::ErrorKind::InvalidData,
                "TLS handshake requested neither read nor write",
            ),
        }),
    }
}

fn perform_handshake_io(
    action: HandshakeIo,
    client: &mut ClientConnection,
    stream: &mut TcpStream,
    deadline: Instant,
    handshake_timeout: Duration,
    error_mapper: RustlsErrorMapper,
) -> Result<(), TlsError> {
    let remaining = remaining_time(deadline, handshake_timeout)?;
    match action {
        HandshakeIo::Write => {
            handshake_timeout_update(stream.set_write_timeout(Some(remaining)))?;
            let written = handshake_transfer(client.write_tls(stream), handshake_timeout)?;
            ensure_handshake_progress(action, written)
        }
        HandshakeIo::Read => {
            handshake_timeout_update(stream.set_read_timeout(Some(remaining)))?;
            let read = handshake_transfer(client.read_tls(stream), handshake_timeout)?;
            ensure_handshake_progress(action, read)?;
            client.process_new_packets().map_err(error_mapper)?;
            Ok(())
        }
    }
}

#[inline(never)]
fn ensure_handshake_progress(action: HandshakeIo, byte_count: usize) -> Result<(), TlsError> {
    if byte_count > 0 {
        return Ok(());
    }
    let (kind, message) = match action {
        HandshakeIo::Write => (
            io::ErrorKind::WriteZero,
            "TLS handshake write made no progress",
        ),
        HandshakeIo::Read => (
            io::ErrorKind::UnexpectedEof,
            "TLS peer closed during handshake",
        ),
    };
    Err(TlsError::HandshakeIoFailed {
        source: io::Error::new(kind, message),
    })
}

#[inline(never)]
fn enforce_handshake_deadline(
    now: Instant,
    deadline: Instant,
    still_handshaking: bool,
    timeout: Duration,
) -> Result<(), TlsError> {
    if still_handshaking && now >= deadline {
        return Err(TlsError::HandshakeTimedOut { timeout });
    }
    Ok(())
}

#[inline(never)]
fn remaining_time(deadline: Instant, timeout: Duration) -> Result<Duration, TlsError> {
    let now = Instant::now();
    if now >= deadline {
        return Err(TlsError::HandshakeTimedOut { timeout });
    }
    Ok(deadline.duration_since(now))
}

#[inline(never)]
fn classify_handshake_io(source: io::Error, timeout: Duration) -> TlsError {
    if matches!(
        source.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ) {
        TlsError::HandshakeTimedOut { timeout }
    } else {
        TlsError::HandshakeIoFailed { source }
    }
}

#[inline(never)]
fn handshake_transfer(result: io::Result<usize>, timeout: Duration) -> Result<usize, TlsError> {
    match result {
        Ok(byte_count) => Ok(byte_count),
        Err(source) => Err(classify_handshake_io(source, timeout)),
    }
}

#[inline(never)]
fn handshake_timeout_query(
    result: io::Result<Option<Duration>>,
) -> Result<Option<Duration>, TlsError> {
    match result {
        Ok(timeout) => Ok(timeout),
        Err(source) => Err(TlsError::HandshakeIoFailed { source }),
    }
}

#[inline(never)]
fn handshake_timeout_update(result: io::Result<()>) -> Result<(), TlsError> {
    match result {
        Ok(()) => Ok(()),
        Err(source) => Err(TlsError::HandshakeIoFailed { source }),
    }
}

fn restore_timeouts(
    stream: &TcpStream,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
) -> Result<(), TlsError> {
    timeout_restoration(stream.set_read_timeout(read_timeout))?;
    timeout_restoration(stream.set_write_timeout(write_timeout))
}

#[inline(never)]
fn timeout_restoration(result: io::Result<()>) -> Result<(), TlsError> {
    match result {
        Ok(()) => Ok(()),
        Err(source) => Err(TlsError::TimeoutRestorationFailed { source }),
    }
}

fn verify_peer_evidence(
    stream: &TcpStream,
    evidence: &SocketConnectionEvidence,
) -> Result<(), TlsError> {
    let expected_peer = evidence.observed_peer();
    let current_peer = peer_inspection(stream.peer_addr(), expected_peer)?;
    validate_peer_addresses(
        evidence.requested_socket(),
        expected_peer,
        current_peer,
    )
}

#[inline(never)]
fn peer_inspection(
    result: io::Result<SocketAddr>,
    expected_peer: SocketAddr,
) -> Result<SocketAddr, TlsError> {
    match result {
        Ok(current_peer) => Ok(current_peer),
        Err(source) => Err(TlsError::PeerInspectionFailed {
            expected_peer,
            source,
        }),
    }
}

#[inline(never)]
fn validate_peer_addresses(
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
    current_peer: SocketAddr,
) -> Result<(), TlsError> {
    if requested_peer != observed_peer {
        return Err(TlsError::InheritedPeerMismatch {
            requested_peer,
            observed_peer,
            current_peer,
        });
    }
    if current_peer != observed_peer {
        return Err(TlsError::InheritedPeerMismatch {
            requested_peer,
            observed_peer,
            current_peer,
        });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_evidence(
    origin: &Origin,
    network_evidence: &SocketConnectionEvidence,
    reference_identity: &TlsReferenceIdentity,
    client: &ClientConnection,
    offered_alpn: &[Vec<u8>],
    alpn_requirement: AlpnRequirement,
    trust_bundle_identifier: crate::TrustBundleIdentifier,
    trust_bundle_hash: String,
    trust_root_count: usize,
    trust_root_bytes: usize,
    trusted_time: UnixTime,
    handshake_duration: Duration,
    handshake_timeout: Duration,
) -> Result<crate::TlsConnectionEvidence, TlsError> {
    let protocol_version = protocol_version_evidence(client.protocol_version())?;
    let (cipher_suite_identifier, cipher_suite_label) =
        cipher_suite_evidence(client.negotiated_cipher_suite())?;
    let negotiated_alpn = negotiated_alpn_evidence(
        client.alpn_protocol(),
        offered_alpn,
        alpn_requirement,
    )?;

    let certificates = peer_certificates(client.peer_certificates())?;
    validate_certificate_bounds(certificates)?;
    let leaf = first_certificate(certificates)?;
    let parsed_leaf = parse_leaf_evidence(leaf)?;
    let presented_certificate_hashes = certificates
        .iter()
        .map(|certificate| hash_bytes(certificate.as_ref()))
        .collect();
    let presented_certificate_bytes = certificates
        .iter()
        .map(|certificate| certificate.len())
        .sum();
    let leaf_certificate_hash = hash_bytes(leaf.as_ref());

    Ok(EvidenceInput {
        origin: origin.clone(),
        requested_peer: network_evidence.requested_socket(),
        observed_peer: network_evidence.observed_peer(),
        reference_identity: reference_identity.clone(),
        protocol_version,
        cipher_suite_identifier,
        cipher_suite_label,
        negotiated_alpn,
        leaf_certificate_hash,
        leaf_spki_hash: parsed_leaf.spki_hash,
        presented_certificate_hashes,
        presented_certificate_count: certificates.len(),
        presented_certificate_bytes,
        leaf_not_before_unix_seconds: parsed_leaf.not_before_unix_seconds,
        leaf_not_after_unix_seconds: parsed_leaf.not_after_unix_seconds,
        trust_bundle_identifier,
        trust_bundle_hash,
        trust_root_count,
        trust_root_bytes,
        trusted_time_unix_seconds: trusted_time.as_secs(),
        handshake_duration,
        handshake_timeout,
    }
    .into())
}

#[inline(never)]
fn protocol_version_evidence(
    protocol_version: Option<ProtocolVersion>,
) -> Result<TlsProtocolVersion, TlsError> {
    match protocol_version {
        Some(ProtocolVersion::TLSv1_2) => Ok(TlsProtocolVersion::Tls12),
        Some(ProtocolVersion::TLSv1_3) => Ok(TlsProtocolVersion::Tls13),
        Some(_other) => Err(TlsError::UnsupportedProtocolVersion),
        None => Err(TlsError::MissingProtocolVersion),
    }
}

#[inline(never)]
fn cipher_suite_evidence(
    suite: Option<SupportedCipherSuite>,
) -> Result<([u8; 2], String), TlsError> {
    match suite {
        Some(suite) => Ok((suite.suite().to_array(), format!("{:?}", suite.suite()))),
        None => Err(TlsError::MissingCipherSuite),
    }
}

#[inline(never)]
fn negotiated_alpn_evidence(
    negotiated_protocol: Option<&[u8]>,
    offered_protocols: &[Vec<u8>],
    requirement: AlpnRequirement,
) -> Result<NegotiatedAlpn, TlsError> {
    match negotiated_protocol {
        Some(protocol) if offered_protocols.iter().any(|offered| offered == protocol) => {
            Ok(NegotiatedAlpn::Protocol(protocol.to_vec()))
        }
        Some(_protocol) => Err(TlsError::UnexpectedAlpn),
        None if requirement == AlpnRequirement::Required => Err(TlsError::AlpnRequired),
        None => Ok(NegotiatedAlpn::Absent),
    }
}

#[inline(never)]
fn peer_certificates<'a>(
    certificates: Option<&'a [CertificateDer<'static>]>,
) -> Result<&'a [CertificateDer<'static>], TlsError> {
    match certificates {
        Some(certificates) => Ok(certificates),
        None => Err(TlsError::MissingPeerCertificates),
    }
}

#[inline(never)]
fn first_certificate<'a>(
    certificates: &'a [CertificateDer<'static>],
) -> Result<&'a CertificateDer<'static>, TlsError> {
    match certificates.first() {
        Some(certificate) => Ok(certificate),
        None => Err(TlsError::MissingPeerCertificates),
    }
}

#[inline(never)]
fn parse_leaf_evidence(
    leaf: &CertificateDer<'_>,
) -> Result<LeafCertificateEvidence, TlsError> {
    let (remaining, parsed_leaf) = match parse_x509_certificate(leaf.as_ref()) {
        Ok(parsed) => parsed,
        Err(_error) => return Err(TlsError::InvalidLeafCertificate),
    };
    if !remaining.is_empty() {
        return Err(TlsError::InvalidLeafCertificate);
    }
    Ok(LeafCertificateEvidence {
        spki_hash: hash_bytes(parsed_leaf.tbs_certificate.subject_pki.raw),
        not_before_unix_seconds: parsed_leaf.validity().not_before.timestamp(),
        not_after_unix_seconds: parsed_leaf.validity().not_after.timestamp(),
    })
}

#[inline(never)]
fn validate_certificate_bounds(certificates: &[CertificateDer<'_>]) -> Result<(), TlsError> {
    if certificates.is_empty() {
        return Err(TlsError::MissingPeerCertificates);
    }
    if certificates.len() > MAX_SERVER_CERTIFICATE_COUNT {
        return Err(TlsError::ExcessivePeerCertificateCount {
            certificate_count: certificates.len(),
            maximum_count: MAX_SERVER_CERTIFICATE_COUNT,
        });
    }
    let byte_count = certificates.iter().fold(0_usize, |total, certificate| {
        total.saturating_add(certificate.len())
    });
    if byte_count > MAX_SERVER_CERTIFICATE_BYTES {
        return Err(TlsError::ExcessivePeerCertificateBytes {
            byte_count,
            maximum_bytes: MAX_SERVER_CERTIFICATE_BYTES,
        });
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    sha256_identifier(digest.as_ref())
}

#[inline(never)]
fn tls_configuration_error(source: rustls::Error) -> TlsError {
    TlsError::TlsConfigurationFailed { source }
}

#[inline(never)]
fn required_rustls_error(source: rustls::Error) -> TlsError {
    classify_rustls_error(source, AlpnRequirement::Required)
}

#[inline(never)]
fn optional_rustls_error(source: rustls::Error) -> TlsError {
    classify_rustls_error(source, AlpnRequirement::Optional)
}

#[inline(never)]
fn rustls_error_mapper(requirement: AlpnRequirement) -> RustlsErrorMapper {
    match requirement {
        AlpnRequirement::Required => required_rustls_error,
        AlpnRequirement::Optional => optional_rustls_error,
    }
}

#[inline(never)]
fn classify_rustls_error(source: rustls::Error, alpn_requirement: AlpnRequirement) -> TlsError {
    enum Classification {
        UnknownIssuer,
        Expired,
        NotYetValid,
        NameMismatch,
        InvalidCertificate,
        AlpnRequired,
        UnexpectedAlpn,
        Protocol,
    }

    let classification = match &source {
        rustls::Error::NoApplicationProtocol
        | rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol) => {
            if alpn_requirement == AlpnRequirement::Required {
                Classification::AlpnRequired
            } else {
                Classification::UnexpectedAlpn
            }
        }
        rustls::Error::InvalidCertificate(certificate_error) => match certificate_error {
            CertificateError::UnknownIssuer => Classification::UnknownIssuer,
            CertificateError::Expired | CertificateError::ExpiredContext { .. } => {
                Classification::Expired
            }
            CertificateError::NotValidYet | CertificateError::NotValidYetContext { .. } => {
                Classification::NotYetValid
            }
            CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. } => {
                Classification::NameMismatch
            }
            _other => Classification::InvalidCertificate,
        },
        _other => Classification::Protocol,
    };

    match classification {
        Classification::UnknownIssuer => TlsError::UnknownIssuer { source },
        Classification::Expired => TlsError::CertificateExpired { source },
        Classification::NotYetValid => TlsError::CertificateNotYetValid { source },
        Classification::NameMismatch => TlsError::ServiceIdentityMismatch { source },
        Classification::InvalidCertificate => TlsError::InvalidCertificate { source },
        Classification::AlpnRequired => TlsError::AlpnRequired,
        Classification::UnexpectedAlpn => TlsError::UnexpectedAlpn,
        Classification::Protocol => TlsError::TlsProtocolFailed { source },
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn socket(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
    }

    #[test]
    fn handshake_action_planning_is_total_and_fail_closed() {
        assert_eq!(
            handshake_actions(true, true).expect("write and read actions"),
            [Some(HandshakeIo::Write), Some(HandshakeIo::Read)]
        );
        assert_eq!(
            handshake_actions(true, false).expect("write action"),
            [Some(HandshakeIo::Write), None]
        );
        assert_eq!(
            handshake_actions(false, true).expect("read action"),
            [Some(HandshakeIo::Read), None]
        );
        let error = handshake_actions(false, false).expect_err("no I/O must fail");
        assert!(matches!(error, TlsError::HandshakeIoFailed { .. }));
    }

    #[test]
    fn zero_byte_handshake_io_is_typed_by_direction() {
        ensure_handshake_progress(HandshakeIo::Write, 1).expect("write progress");
        ensure_handshake_progress(HandshakeIo::Read, 1).expect("read progress");

        let write = ensure_handshake_progress(HandshakeIo::Write, 0)
            .expect_err("zero-byte write must fail");
        assert!(matches!(
            write,
            TlsError::HandshakeIoFailed { ref source }
                if source.kind() == io::ErrorKind::WriteZero
        ));

        let read = ensure_handshake_progress(HandshakeIo::Read, 0)
            .expect_err("zero-byte read must fail");
        assert!(matches!(
            read,
            TlsError::HandshakeIoFailed { ref source }
                if source.kind() == io::ErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn handshake_deadline_checks_both_state_and_time() {
        let timeout = Duration::from_secs(1);
        let now = Instant::now();
        let future = now + timeout;
        enforce_handshake_deadline(now, future, true, timeout).expect("time remains");
        enforce_handshake_deadline(future, now, false, timeout)
            .expect("completed handshake ignores elapsed deadline");
        assert!(matches!(
            enforce_handshake_deadline(future, now, true, timeout),
            Err(TlsError::HandshakeTimedOut { timeout: observed }) if observed == timeout
        ));
    }

    #[test]
    fn an_elapsed_deadline_is_typed_as_timeout() {
        let timeout = Duration::from_secs(1);
        assert!(matches!(
            remaining_time(Instant::now(), timeout),
            Err(TlsError::HandshakeTimedOut { timeout: observed }) if observed == timeout
        ));
        assert!(remaining_time(Instant::now() + timeout, timeout).is_ok());
    }

    #[test]
    fn handshake_io_results_preserve_success_and_classify_failures() {
        let timeout = Duration::from_secs(1);
        assert_eq!(
            handshake_transfer(Ok(7), timeout).expect("transfer success"),
            7
        );
        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let error = handshake_transfer(Err(io::Error::from(kind)), timeout)
                .expect_err("timeout transfer");
            assert!(matches!(error, TlsError::HandshakeTimedOut { .. }));
        }
        let error = handshake_transfer(
            Err(io::Error::from(io::ErrorKind::ConnectionReset)),
            timeout,
        )
        .expect_err("transport failure");
        assert!(matches!(error, TlsError::HandshakeIoFailed { .. }));
    }

    #[test]
    fn socket_timeout_result_mappers_are_fail_closed() {
        let timeout = Some(Duration::from_secs(1));
        assert_eq!(
            handshake_timeout_query(Ok(timeout)).expect("timeout query"),
            timeout
        );
        assert!(matches!(
            handshake_timeout_query(Err(io::Error::from(io::ErrorKind::Other))),
            Err(TlsError::HandshakeIoFailed { .. })
        ));
        handshake_timeout_update(Ok(())).expect("timeout update");
        assert!(matches!(
            handshake_timeout_update(Err(io::Error::from(io::ErrorKind::Other))),
            Err(TlsError::HandshakeIoFailed { .. })
        ));
        timeout_restoration(Ok(())).expect("timeout restoration");
        assert!(matches!(
            timeout_restoration(Err(io::Error::from(io::ErrorKind::Other))),
            Err(TlsError::TimeoutRestorationFailed { .. })
        ));
    }

    #[test]
    fn peer_inspection_and_evidence_consistency_are_fail_closed() {
        let expected = socket(443);
        assert_eq!(
            peer_inspection(Ok(expected), expected).expect("peer inspection"),
            expected
        );
        assert!(matches!(
            peer_inspection(
                Err(io::Error::from(io::ErrorKind::NotConnected)),
                expected,
            ),
            Err(TlsError::PeerInspectionFailed { .. })
        ));
        validate_peer_addresses(expected, expected, expected).expect("consistent peer evidence");
        assert!(matches!(
            validate_peer_addresses(socket(444), expected, expected),
            Err(TlsError::InheritedPeerMismatch { .. })
        ));
        assert!(matches!(
            validate_peer_addresses(expected, expected, socket(444)),
            Err(TlsError::InheritedPeerMismatch { .. })
        ));
    }

    #[test]
    fn negotiated_protocol_and_cipher_evidence_are_total() {
        assert_eq!(
            protocol_version_evidence(Some(ProtocolVersion::TLSv1_2))
                .expect("TLS 1.2 evidence"),
            TlsProtocolVersion::Tls12
        );
        assert_eq!(
            protocol_version_evidence(Some(ProtocolVersion::TLSv1_3))
                .expect("TLS 1.3 evidence"),
            TlsProtocolVersion::Tls13
        );
        assert!(matches!(
            protocol_version_evidence(Some(ProtocolVersion::SSLv3)),
            Err(TlsError::UnsupportedProtocolVersion)
        ));
        assert!(matches!(
            protocol_version_evidence(None),
            Err(TlsError::MissingProtocolVersion)
        ));

        let suite = rustls::crypto::ring::default_provider()
            .cipher_suites
            .first()
            .copied()
            .expect("ring provider cipher suite");
        let (identifier, label) =
            cipher_suite_evidence(Some(suite)).expect("cipher suite evidence");
        assert_ne!(identifier, [0_u8; 2]);
        assert!(!label.is_empty());
        assert!(matches!(
            cipher_suite_evidence(None),
            Err(TlsError::MissingCipherSuite)
        ));
    }

    #[test]
    fn alpn_evidence_covers_match_mismatch_required_and_absent() {
        let offered = vec![b"http/1.1".to_vec(), b"h2".to_vec()];
        assert_eq!(
            negotiated_alpn_evidence(Some(b"h2"), &offered, AlpnRequirement::Required)
                .expect("offered ALPN"),
            NegotiatedAlpn::Protocol(b"h2".to_vec())
        );
        assert!(matches!(
            negotiated_alpn_evidence(Some(b"h3"), &offered, AlpnRequirement::Optional),
            Err(TlsError::UnexpectedAlpn)
        ));
        assert!(matches!(
            negotiated_alpn_evidence(None, &offered, AlpnRequirement::Required),
            Err(TlsError::AlpnRequired)
        ));
        assert_eq!(
            negotiated_alpn_evidence(None, &offered, AlpnRequirement::Optional)
                .expect("optional absence"),
            NegotiatedAlpn::Absent
        );
    }

    #[test]
    fn certificate_selection_and_leaf_parsing_fail_closed() {
        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])
            .expect("test certificate generation")
            .cert
            .der()
            .clone();
        let certificates = vec![certificate.clone()];

        assert!(matches!(
            peer_certificates(None),
            Err(TlsError::MissingPeerCertificates)
        ));
        assert_eq!(
            peer_certificates(Some(&certificates))
                .expect("presented certificates")
                .len(),
            1
        );
        assert!(matches!(
            first_certificate(&[]),
            Err(TlsError::MissingPeerCertificates)
        ));
        assert_eq!(
            first_certificate(&certificates)
                .expect("leaf certificate")
                .as_ref(),
            certificate.as_ref()
        );

        let parsed = parse_leaf_evidence(&certificate).expect("valid X.509 certificate");
        assert!(parsed.spki_hash.starts_with("sha256:"));
        assert!(parsed.not_before_unix_seconds < parsed.not_after_unix_seconds);

        let invalid = CertificateDer::from(vec![0_u8]);
        assert!(matches!(
            parse_leaf_evidence(&invalid),
            Err(TlsError::InvalidLeafCertificate)
        ));
        let mut trailing = certificate.to_vec();
        trailing.push(0);
        let trailing = CertificateDer::from(trailing);
        assert!(matches!(
            parse_leaf_evidence(&trailing),
            Err(TlsError::InvalidLeafCertificate)
        ));
    }

    #[test]
    fn certificate_bounds_are_fail_closed() {
        let valid = vec![CertificateDer::from(vec![1_u8])];
        validate_certificate_bounds(&valid).expect("bounded certificate");
        assert!(matches!(
            validate_certificate_bounds(&[]),
            Err(TlsError::MissingPeerCertificates)
        ));
        let excessive_count =
            vec![CertificateDer::from(vec![1_u8]); MAX_SERVER_CERTIFICATE_COUNT + 1];
        assert!(matches!(
            validate_certificate_bounds(&excessive_count),
            Err(TlsError::ExcessivePeerCertificateCount { .. })
        ));
        let excessive_bytes = vec![CertificateDer::from(vec![
            1_u8;
            MAX_SERVER_CERTIFICATE_BYTES + 1
        ])];
        assert!(matches!(
            validate_certificate_bounds(&excessive_bytes),
            Err(TlsError::ExcessivePeerCertificateBytes { .. })
        ));
    }

    #[test]
    fn configuration_and_rustls_error_mappers_preserve_policy() {
        assert!(matches!(
            tls_configuration_error(rustls::Error::General("config".to_owned())),
            TlsError::TlsConfigurationFailed { .. }
        ));

        let required = rustls_error_mapper(AlpnRequirement::Required);
        let optional = rustls_error_mapper(AlpnRequirement::Optional);
        assert!(matches!(
            required(rustls::Error::NoApplicationProtocol),
            TlsError::AlpnRequired
        ));
        assert!(matches!(
            optional(rustls::Error::NoApplicationProtocol),
            TlsError::UnexpectedAlpn
        ));
    }

    #[test]
    fn rustls_certificate_errors_are_typed() {
        let cases = [
            (CertificateError::UnknownIssuer, "configured trusted issuer"),
            (CertificateError::Expired, "expired"),
            (CertificateError::NotValidYet, "not yet valid"),
            (CertificateError::NotValidForName, "subjectAltName"),
            (CertificateError::BadEncoding, "invalid"),
        ];
        for (certificate_error, expected) in cases {
            let error = classify_rustls_error(
                rustls::Error::InvalidCertificate(certificate_error),
                AlpnRequirement::Optional,
            );
            assert!(error.to_string().contains(expected));
        }
        let protocol = classify_rustls_error(
            rustls::Error::General("test".to_owned()),
            AlpnRequirement::Optional,
        );
        assert!(matches!(protocol, TlsError::TlsProtocolFailed { .. }));
    }

    #[test]
    fn no_application_protocol_is_typed_by_policy() {
        for source in [
            rustls::Error::NoApplicationProtocol,
            rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol),
        ] {
            let required = classify_rustls_error(source, AlpnRequirement::Required);
            assert!(matches!(required, TlsError::AlpnRequired));
        }
        for source in [
            rustls::Error::NoApplicationProtocol,
            rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol),
        ] {
            let optional = classify_rustls_error(source, AlpnRequirement::Optional);
            assert!(matches!(optional, TlsError::UnexpectedAlpn));
        }
    }
}
