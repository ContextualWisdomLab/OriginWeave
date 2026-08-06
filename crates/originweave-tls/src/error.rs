use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use originweave_core::Origin;

/// A deterministic reason that TLS service-identity authority failed.
#[derive(Debug)]
pub enum TlsError {
    /// A trust-bundle policy identifier was empty, excessive, or unsafe.
    InvalidTrustBundleIdentifier,
    /// The supplied number of trust roots was outside the accepted range.
    InvalidTrustRootCount {
        /// The rejected number of input roots.
        root_count: usize,
        /// The largest accepted input count.
        maximum_count: usize,
    },
    /// The supplied trust-root bytes exceeded the input budget.
    InvalidTrustRootBytes {
        /// The rejected encoded input size.
        byte_count: usize,
        /// The largest accepted encoded input size.
        maximum_bytes: usize,
    },
    /// One supplied trust root was not valid DER for the rustls root store.
    InvalidTrustRoot {
        /// The zero-based canonical root index.
        root_index: usize,
        /// The rustls trust-store error.
        source: rustls::Error,
    },
    /// The handshake timeout was zero or exceeded the supported maximum.
    InvalidHandshakeTimeout {
        /// The rejected timeout.
        timeout: Duration,
        /// The largest accepted timeout.
        maximum_timeout: Duration,
    },
    /// The ALPN protocol count exceeded the supported maximum or was missing when required.
    InvalidAlpnCount {
        /// The rejected protocol count.
        protocol_count: usize,
        /// The largest accepted protocol count.
        maximum_count: usize,
    },
    /// One ALPN identifier was empty or longer than the supported maximum.
    InvalidAlpnIdentifier {
        /// The zero-based identifier index.
        protocol_index: usize,
        /// The rejected identifier length.
        protocol_length: usize,
        /// The largest accepted identifier length.
        maximum_length: usize,
    },
    /// The ALPN allow-list repeated an identifier.
    DuplicateAlpnIdentifier {
        /// The zero-based repeated identifier index.
        protocol_index: usize,
    },
    /// The total ALPN identifier bytes exceeded the supported maximum.
    InvalidAlpnBytes {
        /// The rejected total byte count.
        byte_count: usize,
        /// The largest accepted total byte count.
        maximum_bytes: usize,
    },
    /// TLS service identity was requested for a non-HTTPS origin.
    OriginRequiresHttps {
        /// The rejected canonical origin.
        origin: Origin,
    },
    /// The canonical origin host could not become a TLS reference identity.
    InvalidReferenceIdentity {
        /// The rejected canonical origin.
        origin: Origin,
    },
    /// The inherited direct TCP evidence or live peer was inconsistent.
    InheritedPeerMismatch {
        /// The socket submitted to the operating system.
        requested_peer: SocketAddr,
        /// The peer previously observed by the direct network kernel.
        observed_peer: SocketAddr,
        /// The peer currently reported by the stream.
        current_peer: SocketAddr,
    },
    /// The current TCP peer could not be inspected.
    PeerInspectionFailed {
        /// The peer expected from direct TCP evidence.
        expected_peer: SocketAddr,
        /// The operating-system inspection error.
        source: io::Error,
    },
    /// A safe rustls client configuration could not be constructed.
    TlsConfigurationFailed {
        /// The rustls configuration error.
        source: rustls::Error,
    },
    /// The configured monotonic handshake deadline elapsed.
    HandshakeTimedOut {
        /// The configured total handshake timeout.
        timeout: Duration,
    },
    /// TLS handshake transport I/O failed.
    HandshakeIoFailed {
        /// The underlying stream error.
        source: io::Error,
    },
    /// The certificate chain did not lead to a configured trust root.
    UnknownIssuer {
        /// The rustls WebPKI error.
        source: rustls::Error,
    },
    /// The leaf or required issuing certificate was expired at the trusted time.
    CertificateExpired {
        /// The rustls WebPKI error.
        source: rustls::Error,
    },
    /// The leaf or required issuing certificate was not yet valid at the trusted time.
    CertificateNotYetValid {
        /// The rustls WebPKI error.
        source: rustls::Error,
    },
    /// The certificate did not contain the required DNS or IP service identity.
    ServiceIdentityMismatch {
        /// The rustls WebPKI error.
        source: rustls::Error,
    },
    /// The certificate or validated chain failed another WebPKI requirement.
    InvalidCertificate {
        /// The rustls certificate-validation error.
        source: rustls::Error,
    },
    /// The TLS peer or protocol violated another handshake requirement.
    TlsProtocolFailed {
        /// The rustls protocol error.
        source: rustls::Error,
    },
    /// No negotiated TLS protocol version was available after the handshake.
    MissingProtocolVersion,
    /// The negotiated protocol version was not TLS 1.2 or TLS 1.3.
    UnsupportedProtocolVersion,
    /// No negotiated cipher suite was available after the handshake.
    MissingCipherSuite,
    /// ALPN negotiation was required but the server selected no protocol.
    AlpnRequired,
    /// The server-selected ALPN identifier was absent from the allow-list.
    UnexpectedAlpn,
    /// The server did not present a certificate chain.
    MissingPeerCertificates,
    /// The server-presented certificate count exceeded the evidence budget.
    ExcessivePeerCertificateCount {
        /// The rejected certificate count.
        certificate_count: usize,
        /// The largest accepted certificate count.
        maximum_count: usize,
    },
    /// The server-presented certificate bytes exceeded the evidence budget.
    ExcessivePeerCertificateBytes {
        /// The rejected DER byte count.
        byte_count: usize,
        /// The largest accepted DER byte count.
        maximum_bytes: usize,
    },
    /// The leaf certificate could not be parsed as one complete X.509 DER object.
    InvalidLeafCertificate,
    /// Clearing the temporary handshake socket timeout failed.
    TimeoutRestorationFailed {
        /// The operating-system timeout error.
        source: io::Error,
    },
}

impl fmt::Display for TlsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTrustBundleIdentifier => formatter.write_str(
                "trust bundle identifier must contain 1..=128 ASCII letters, digits, '.', '_', ':', or '-'",
            ),
            Self::InvalidTrustRootCount {
                root_count,
                maximum_count,
            } => write!(
                formatter,
                "trust root count {root_count} is outside 1..={maximum_count}",
            ),
            Self::InvalidTrustRootBytes {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "trust root input uses {byte_count} bytes, exceeding {maximum_bytes}",
            ),
            Self::InvalidTrustRoot { root_index, .. } => {
                write!(formatter, "trust root at canonical index {root_index} is invalid")
            }
            Self::InvalidHandshakeTimeout {
                timeout,
                maximum_timeout,
            } => write!(
                formatter,
                "TLS handshake timeout {timeout:?} is outside 1ns..={maximum_timeout:?}",
            ),
            Self::InvalidAlpnCount {
                protocol_count,
                maximum_count,
            } => write!(
                formatter,
                "ALPN protocol count {protocol_count} exceeds the supported maximum {maximum_count}",
            ),
            Self::InvalidAlpnIdentifier {
                protocol_index,
                protocol_length,
                maximum_length,
            } => write!(
                formatter,
                "ALPN identifier {protocol_index} has length {protocol_length}, outside 1..={maximum_length}",
            ),
            Self::DuplicateAlpnIdentifier { protocol_index } => write!(
                formatter,
                "ALPN identifier {protocol_index} duplicates an earlier identifier",
            ),
            Self::InvalidAlpnBytes {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "ALPN identifiers use {byte_count} bytes, exceeding {maximum_bytes}",
            ),
            Self::OriginRequiresHttps { origin } => {
                write!(formatter, "TLS service identity requires an HTTPS origin: {origin}")
            }
            Self::InvalidReferenceIdentity { origin } => write!(
                formatter,
                "origin host is not a valid DNS or IP TLS reference identity: {origin}",
            ),
            Self::InheritedPeerMismatch {
                requested_peer,
                observed_peer,
                current_peer,
            } => write!(
                formatter,
                "direct TCP peer evidence is inconsistent: requested {requested_peer}, observed {observed_peer}, current {current_peer}",
            ),
            Self::PeerInspectionFailed { expected_peer, .. } => write!(
                formatter,
                "could not inspect the TCP peer expected to be {expected_peer}",
            ),
            Self::TlsConfigurationFailed { .. } => {
                formatter.write_str("safe TLS client configuration failed")
            }
            Self::HandshakeTimedOut { timeout } => {
                write!(formatter, "TLS handshake exceeded the total timeout {timeout:?}")
            }
            Self::HandshakeIoFailed { .. } => formatter.write_str("TLS handshake I/O failed"),
            Self::UnknownIssuer { .. } => {
                formatter.write_str("TLS certificate chain has no configured trusted issuer")
            }
            Self::CertificateExpired { .. } => {
                formatter.write_str("TLS certificate chain is expired at the trusted time")
            }
            Self::CertificateNotYetValid { .. } => formatter
                .write_str("TLS certificate chain is not yet valid at the trusted time"),
            Self::ServiceIdentityMismatch { .. } => formatter.write_str(
                "TLS certificate subjectAltName does not match the canonical origin identity",
            ),
            Self::InvalidCertificate { .. } => {
                formatter.write_str("TLS certificate chain is invalid")
            }
            Self::TlsProtocolFailed { .. } => formatter.write_str("TLS handshake protocol failed"),
            Self::MissingProtocolVersion => {
                formatter.write_str("TLS handshake produced no protocol version")
            }
            Self::UnsupportedProtocolVersion => formatter
                .write_str("TLS handshake negotiated a protocol other than TLS 1.2 or TLS 1.3"),
            Self::MissingCipherSuite => {
                formatter.write_str("TLS handshake produced no cipher suite")
            }
            Self::AlpnRequired => {
                formatter.write_str("TLS policy requires an ALPN protocol selection")
            }
            Self::UnexpectedAlpn => {
                formatter.write_str("TLS peer selected an ALPN protocol outside the allow-list")
            }
            Self::MissingPeerCertificates => {
                formatter.write_str("TLS peer presented no certificate chain")
            }
            Self::ExcessivePeerCertificateCount {
                certificate_count,
                maximum_count,
            } => write!(
                formatter,
                "TLS peer presented {certificate_count} certificates, exceeding {maximum_count}",
            ),
            Self::ExcessivePeerCertificateBytes {
                byte_count,
                maximum_bytes,
            } => write!(
                formatter,
                "TLS peer presented {byte_count} certificate bytes, exceeding {maximum_bytes}",
            ),
            Self::InvalidLeafCertificate => {
                formatter.write_str("TLS leaf certificate is not one complete X.509 DER object")
            }
            Self::TimeoutRestorationFailed { .. } => {
                formatter.write_str("could not restore TCP stream timeout after TLS handshake")
            }
        }
    }
}

impl std::error::Error for TlsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidTrustRoot { source, .. }
            | Self::TlsConfigurationFailed { source }
            | Self::UnknownIssuer { source }
            | Self::CertificateExpired { source }
            | Self::CertificateNotYetValid { source }
            | Self::ServiceIdentityMismatch { source }
            | Self::InvalidCertificate { source }
            | Self::TlsProtocolFailed { source } => Some(source),
            Self::PeerInspectionFailed { source, .. }
            | Self::HandshakeIoFailed { source }
            | Self::TimeoutRestorationFailed { source } => Some(source),
            Self::InvalidTrustBundleIdentifier
            | Self::InvalidTrustRootCount { .. }
            | Self::InvalidTrustRootBytes { .. }
            | Self::InvalidHandshakeTimeout { .. }
            | Self::InvalidAlpnCount { .. }
            | Self::InvalidAlpnIdentifier { .. }
            | Self::DuplicateAlpnIdentifier { .. }
            | Self::InvalidAlpnBytes { .. }
            | Self::OriginRequiresHttps { .. }
            | Self::InvalidReferenceIdentity { .. }
            | Self::InheritedPeerMismatch { .. }
            | Self::HandshakeTimedOut { .. }
            | Self::MissingProtocolVersion
            | Self::UnsupportedProtocolVersion
            | Self::MissingCipherSuite
            | Self::AlpnRequired
            | Self::UnexpectedAlpn
            | Self::MissingPeerCertificates
            | Self::ExcessivePeerCertificateCount { .. }
            | Self::ExcessivePeerCertificateBytes { .. }
            | Self::InvalidLeafCertificate => None,
        }
    }
}
