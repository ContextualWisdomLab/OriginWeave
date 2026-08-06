use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use originweave_core::Origin;
use originweave_network::{DirectTcpConnection, SocketConnectionEvidence};
use rustls::client::Resumption;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::time_provider::TimeProvider;
use rustls::{CertificateError, ClientConfig, ClientConnection, ProtocolVersion, StreamOwned};
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
            .map_err(|source| TlsError::TlsConfigurationFailed { source })?;
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

        let mut client =
            ClientConnection::new(Arc::new(config), server_name).map_err(classify_rustls_error)?;
        let original_read_timeout = stream
            .read_timeout()
            .map_err(|source| TlsError::HandshakeIoFailed { source })?;
        let original_write_timeout = stream
            .write_timeout()
            .map_err(|source| TlsError::HandshakeIoFailed { source })?;
        let started_at = Instant::now();
        let deadline =
            started_at
                .checked_add(handshake_timeout)
                .ok_or(TlsError::InvalidHandshakeTimeout {
                    timeout: handshake_timeout,
                    maximum_timeout: crate::MAX_TLS_HANDSHAKE_TIMEOUT,
                })?;

        let handshake_result = drive_handshake(
            &mut client,
            &mut stream,
            &network_evidence,
            deadline,
            handshake_timeout,
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

fn drive_handshake(
    client: &mut ClientConnection,
    stream: &mut TcpStream,
    network_evidence: &SocketConnectionEvidence,
    deadline: Instant,
    handshake_timeout: Duration,
) -> Result<(), TlsError> {
    while client.is_handshaking() {
        verify_peer_evidence(stream, network_evidence)?;
        let mut progressed = false;
        if client.wants_write() {
            let remaining = remaining_time(deadline, handshake_timeout)?;
            stream
                .set_write_timeout(Some(remaining))
                .map_err(|source| TlsError::HandshakeIoFailed { source })?;
            let written = client
                .write_tls(stream)
                .map_err(|source| classify_handshake_io(source, handshake_timeout))?;
            if written == 0 {
                return Err(TlsError::HandshakeIoFailed {
                    source: io::Error::new(
                        io::ErrorKind::WriteZero,
                        "TLS handshake write made no progress",
                    ),
                });
            }
            progressed = true;
        }
        if client.wants_read() {
            let remaining = remaining_time(deadline, handshake_timeout)?;
            stream
                .set_read_timeout(Some(remaining))
                .map_err(|source| TlsError::HandshakeIoFailed { source })?;
            let read = client
                .read_tls(stream)
                .map_err(|source| classify_handshake_io(source, handshake_timeout))?;
            if read == 0 {
                return Err(TlsError::HandshakeIoFailed {
                    source: io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "TLS peer closed during handshake",
                    ),
                });
            }
            client
                .process_new_packets()
                .map_err(classify_rustls_error)?;
            progressed = true;
        }
        if !progressed {
            return Err(TlsError::HandshakeIoFailed {
                source: io::Error::new(
                    io::ErrorKind::InvalidData,
                    "TLS handshake requested neither read nor write",
                ),
            });
        }
        if Instant::now() >= deadline && client.is_handshaking() {
            return Err(TlsError::HandshakeTimedOut {
                timeout: handshake_timeout,
            });
        }
    }
    Ok(())
}

fn remaining_time(deadline: Instant, timeout: Duration) -> Result<Duration, TlsError> {
    let now = Instant::now();
    if now >= deadline {
        return Err(TlsError::HandshakeTimedOut { timeout });
    }
    Ok(deadline.duration_since(now))
}

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

fn restore_timeouts(
    stream: &TcpStream,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
) -> Result<(), TlsError> {
    stream
        .set_read_timeout(read_timeout)
        .map_err(|source| TlsError::TimeoutRestorationFailed { source })?;
    stream
        .set_write_timeout(write_timeout)
        .map_err(|source| TlsError::TimeoutRestorationFailed { source })?;
    Ok(())
}

fn verify_peer_evidence(
    stream: &TcpStream,
    evidence: &SocketConnectionEvidence,
) -> Result<(), TlsError> {
    let current_peer = stream
        .peer_addr()
        .map_err(|source| TlsError::PeerInspectionFailed {
            expected_peer: evidence.observed_peer(),
            source,
        })?;
    if evidence.requested_socket() != evidence.observed_peer()
        || current_peer != evidence.observed_peer()
    {
        return Err(TlsError::InheritedPeerMismatch {
            requested_peer: evidence.requested_socket(),
            observed_peer: evidence.observed_peer(),
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
    let protocol_version = match client
        .protocol_version()
        .ok_or(TlsError::MissingProtocolVersion)?
    {
        ProtocolVersion::TLSv1_2 => TlsProtocolVersion::Tls12,
        ProtocolVersion::TLSv1_3 => TlsProtocolVersion::Tls13,
        _other => return Err(TlsError::UnsupportedProtocolVersion),
    };
    let suite = client
        .negotiated_cipher_suite()
        .ok_or(TlsError::MissingCipherSuite)?;
    let cipher_suite_identifier = suite.suite().to_array();
    let cipher_suite_label = format!("{:?}", suite.suite());
    let negotiated_alpn = match client.alpn_protocol() {
        Some(protocol) => {
            if !offered_alpn.iter().any(|offered| offered == protocol) {
                return Err(TlsError::UnexpectedAlpn);
            }
            NegotiatedAlpn::Protocol(protocol.to_vec())
        }
        None if alpn_requirement == AlpnRequirement::Required => {
            return Err(TlsError::AlpnRequired);
        }
        None => NegotiatedAlpn::Absent,
    };

    let certificates = client
        .peer_certificates()
        .ok_or(TlsError::MissingPeerCertificates)?;
    validate_certificate_bounds(certificates)?;
    let leaf = certificates
        .first()
        .ok_or(TlsError::MissingPeerCertificates)?;
    let (remaining, parsed_leaf) =
        parse_x509_certificate(leaf.as_ref()).map_err(|_error| TlsError::InvalidLeafCertificate)?;
    if !remaining.is_empty() {
        return Err(TlsError::InvalidLeafCertificate);
    }
    let presented_certificate_hashes = certificates
        .iter()
        .map(|certificate| hash_bytes(certificate.as_ref()))
        .collect();
    let presented_certificate_bytes = certificates
        .iter()
        .map(|certificate| certificate.len())
        .sum();
    let leaf_certificate_hash = hash_bytes(leaf.as_ref());
    let leaf_spki_hash = hash_bytes(parsed_leaf.tbs_certificate.subject_pki.raw);
    let leaf_not_before_unix_seconds = parsed_leaf.validity().not_before.timestamp();
    let leaf_not_after_unix_seconds = parsed_leaf.validity().not_after.timestamp();

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
        leaf_spki_hash,
        presented_certificate_hashes,
        presented_certificate_count: certificates.len(),
        presented_certificate_bytes,
        leaf_not_before_unix_seconds,
        leaf_not_after_unix_seconds,
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
    let byte_count = certificates.iter().try_fold(0_usize, |total, certificate| {
        total.checked_add(certificate.len())
    });
    let Some(byte_count) = byte_count else {
        return Err(TlsError::ExcessivePeerCertificateBytes {
            byte_count: usize::MAX,
            maximum_bytes: MAX_SERVER_CERTIFICATE_BYTES,
        });
    };
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

fn classify_rustls_error(source: rustls::Error) -> TlsError {
    enum Classification {
        UnknownIssuer,
        Expired,
        NotYetValid,
        NameMismatch,
        InvalidCertificate,
        Protocol,
    }

    let classification = match &source {
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
        Classification::Protocol => TlsError::TlsProtocolFailed { source },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_io_classification_is_explicit() {
        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let error = classify_handshake_io(io::Error::from(kind), Duration::from_secs(1));
            assert!(matches!(error, TlsError::HandshakeTimedOut { .. }));
        }
        let error = classify_handshake_io(
            io::Error::from(io::ErrorKind::ConnectionReset),
            Duration::from_secs(1),
        );
        assert!(matches!(error, TlsError::HandshakeIoFailed { .. }));
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
            let error = classify_rustls_error(rustls::Error::InvalidCertificate(certificate_error));
            assert!(error.to_string().contains(expected));
        }
        let protocol = classify_rustls_error(rustls::Error::General("test".to_owned()));
        assert!(matches!(protocol, TlsError::TlsProtocolFailed { .. }));
    }

    #[test]
    fn certificate_bounds_are_fail_closed() {
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
}
