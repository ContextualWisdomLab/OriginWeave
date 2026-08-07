use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use originweave_core::Origin;
use rustls::{ClientConnection, StreamOwned};

use crate::{TlsReferenceIdentity, TrustBundleIdentifier};

/// The authenticated TLS protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsProtocolVersion {
    /// TLS version 1.2.
    Tls12,
    /// TLS version 1.3.
    Tls13,
}

/// The explicit result of ALPN negotiation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiatedAlpn {
    /// The TLS peer selected no ALPN value and policy permitted absence.
    Absent,
    /// The TLS peer selected this allowed ALPN identifier.
    Protocol(Vec<u8>),
}

/// The revocation-validation result represented by the first TLS slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationStatus {
    /// No OCSP or CRL evidence was configured; no revocation claim is made.
    NotConfigured,
}

/// Credential-free evidence for one authenticated TLS service identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsConnectionEvidence {
    origin: Origin,
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
    reference_identity: TlsReferenceIdentity,
    protocol_version: TlsProtocolVersion,
    cipher_suite_identifier: [u8; 2],
    cipher_suite_label: String,
    negotiated_alpn: NegotiatedAlpn,
    leaf_certificate_hash: String,
    leaf_spki_hash: String,
    presented_certificate_hashes: Vec<String>,
    presented_certificate_count: usize,
    presented_certificate_bytes: usize,
    leaf_not_before_unix_seconds: i64,
    leaf_not_after_unix_seconds: i64,
    trust_bundle_identifier: TrustBundleIdentifier,
    trust_bundle_hash: String,
    trust_root_count: usize,
    trust_root_bytes: usize,
    trusted_time_unix_seconds: u64,
    revocation_status: RevocationStatus,
    handshake_duration: Duration,
    handshake_timeout: Duration,
}

impl TlsConnectionEvidence {
    /// Return the authenticated canonical HTTPS origin.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the exact socket submitted by the direct network kernel.
    #[must_use]
    pub const fn requested_peer(&self) -> SocketAddr {
        self.requested_peer
    }

    /// Return the exact operating-system peer observed after TLS authentication.
    #[must_use]
    pub const fn observed_peer(&self) -> SocketAddr {
        self.observed_peer
    }

    /// Return the RFC 9525 DNS or IP reference identity.
    #[must_use]
    pub const fn reference_identity(&self) -> &TlsReferenceIdentity {
        &self.reference_identity
    }

    /// Return the authenticated TLS protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> TlsProtocolVersion {
        self.protocol_version
    }

    /// Return the two-byte IANA cipher-suite identifier.
    #[must_use]
    pub const fn cipher_suite_identifier(&self) -> [u8; 2] {
        self.cipher_suite_identifier
    }

    /// Return the stable rustls cipher-suite label recorded for operators.
    #[must_use]
    pub const fn cipher_suite_label(&self) -> &str {
        self.cipher_suite_label.as_str()
    }

    /// Return the selected allowed ALPN identifier or explicit absence.
    #[must_use]
    pub const fn negotiated_alpn(&self) -> &NegotiatedAlpn {
        &self.negotiated_alpn
    }

    /// Return the SHA-256 identifier of the leaf certificate DER.
    #[must_use]
    pub const fn leaf_certificate_hash(&self) -> &str {
        self.leaf_certificate_hash.as_str()
    }

    /// Return the SHA-256 identifier of the leaf SubjectPublicKeyInfo DER.
    #[must_use]
    pub const fn leaf_spki_hash(&self) -> &str {
        self.leaf_spki_hash.as_str()
    }

    /// Return ordered hashes of the server-presented certificates accepted by verification.
    #[must_use]
    pub const fn presented_certificate_hashes(&self) -> &[String] {
        self.presented_certificate_hashes.as_slice()
    }

    /// Return the server-presented certificate count.
    #[must_use]
    pub const fn presented_certificate_count(&self) -> usize {
        self.presented_certificate_count
    }

    /// Return the total server-presented certificate DER bytes.
    #[must_use]
    pub const fn presented_certificate_bytes(&self) -> usize {
        self.presented_certificate_bytes
    }

    /// Return the leaf certificate `notBefore` Unix timestamp.
    #[must_use]
    pub const fn leaf_not_before_unix_seconds(&self) -> i64 {
        self.leaf_not_before_unix_seconds
    }

    /// Return the leaf certificate `notAfter` Unix timestamp.
    #[must_use]
    pub const fn leaf_not_after_unix_seconds(&self) -> i64 {
        self.leaf_not_after_unix_seconds
    }

    /// Return the policy identifier for the explicit trust bundle.
    #[must_use]
    pub const fn trust_bundle_identifier(&self) -> &TrustBundleIdentifier {
        &self.trust_bundle_identifier
    }

    /// Return the canonical trust-bundle SHA-256 identifier.
    #[must_use]
    pub const fn trust_bundle_hash(&self) -> &str {
        self.trust_bundle_hash.as_str()
    }

    /// Return the canonical distinct trust-root count.
    #[must_use]
    pub const fn trust_root_count(&self) -> usize {
        self.trust_root_count
    }

    /// Return the canonical distinct trust-root DER bytes.
    #[must_use]
    pub const fn trust_root_bytes(&self) -> usize {
        self.trust_root_bytes
    }

    /// Return the fixed certificate-validation time.
    #[must_use]
    pub const fn trusted_time_unix_seconds(&self) -> u64 {
        self.trusted_time_unix_seconds
    }

    /// Return the explicit revocation-validation status.
    #[must_use]
    pub const fn revocation_status(&self) -> RevocationStatus {
        self.revocation_status
    }

    /// Return the measured monotonic TLS handshake duration.
    #[must_use]
    pub const fn handshake_duration(&self) -> Duration {
        self.handshake_duration
    }

    /// Return the configured total TLS handshake timeout.
    #[must_use]
    pub const fn handshake_timeout(&self) -> Duration {
        self.handshake_timeout
    }
}

pub(crate) struct EvidenceInput {
    pub(crate) origin: Origin,
    pub(crate) requested_peer: SocketAddr,
    pub(crate) observed_peer: SocketAddr,
    pub(crate) reference_identity: TlsReferenceIdentity,
    pub(crate) protocol_version: TlsProtocolVersion,
    pub(crate) cipher_suite_identifier: [u8; 2],
    pub(crate) cipher_suite_label: String,
    pub(crate) negotiated_alpn: NegotiatedAlpn,
    pub(crate) leaf_certificate_hash: String,
    pub(crate) leaf_spki_hash: String,
    pub(crate) presented_certificate_hashes: Vec<String>,
    pub(crate) presented_certificate_count: usize,
    pub(crate) presented_certificate_bytes: usize,
    pub(crate) leaf_not_before_unix_seconds: i64,
    pub(crate) leaf_not_after_unix_seconds: i64,
    pub(crate) trust_bundle_identifier: TrustBundleIdentifier,
    pub(crate) trust_bundle_hash: String,
    pub(crate) trust_root_count: usize,
    pub(crate) trust_root_bytes: usize,
    pub(crate) trusted_time_unix_seconds: u64,
    pub(crate) handshake_duration: Duration,
    pub(crate) handshake_timeout: Duration,
}

impl From<EvidenceInput> for TlsConnectionEvidence {
    fn from(input: EvidenceInput) -> Self {
        Self {
            origin: input.origin,
            requested_peer: input.requested_peer,
            observed_peer: input.observed_peer,
            reference_identity: input.reference_identity,
            protocol_version: input.protocol_version,
            cipher_suite_identifier: input.cipher_suite_identifier,
            cipher_suite_label: input.cipher_suite_label,
            negotiated_alpn: input.negotiated_alpn,
            leaf_certificate_hash: input.leaf_certificate_hash,
            leaf_spki_hash: input.leaf_spki_hash,
            presented_certificate_hashes: input.presented_certificate_hashes,
            presented_certificate_count: input.presented_certificate_count,
            presented_certificate_bytes: input.presented_certificate_bytes,
            leaf_not_before_unix_seconds: input.leaf_not_before_unix_seconds,
            leaf_not_after_unix_seconds: input.leaf_not_after_unix_seconds,
            trust_bundle_identifier: input.trust_bundle_identifier,
            trust_bundle_hash: input.trust_bundle_hash,
            trust_root_count: input.trust_root_count,
            trust_root_bytes: input.trust_root_bytes,
            trusted_time_unix_seconds: input.trusted_time_unix_seconds,
            revocation_status: RevocationStatus::NotConfigured,
            handshake_duration: input.handshake_duration,
            handshake_timeout: input.handshake_timeout,
        }
    }
}

/// One authenticated rustls stream and its immutable TLS evidence.
#[derive(Debug)]
pub struct AuthenticatedTlsConnection {
    pub(crate) stream: StreamOwned<ClientConnection, TcpStream>,
    pub(crate) evidence: TlsConnectionEvidence,
}

impl AuthenticatedTlsConnection {
    /// Borrow the authenticated TLS stream.
    #[must_use]
    pub const fn stream(&self) -> &StreamOwned<ClientConnection, TcpStream> {
        &self.stream
    }

    /// Mutably borrow the authenticated TLS stream for the next bounded adapter.
    #[must_use]
    pub fn stream_mut(&mut self) -> &mut StreamOwned<ClientConnection, TcpStream> {
        &mut self.stream
    }

    /// Borrow the immutable credential-free TLS evidence.
    #[must_use]
    pub const fn evidence(&self) -> &TlsConnectionEvidence {
        &self.evidence
    }

    /// Consume the wrapper and return the authenticated stream and evidence.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        StreamOwned<ClientConnection, TcpStream>,
        TlsConnectionEvidence,
    ) {
        (self.stream, self.evidence)
    }
}
