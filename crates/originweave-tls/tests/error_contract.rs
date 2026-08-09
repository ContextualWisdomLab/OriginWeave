#![allow(clippy::expect_used)]

use std::error::Error;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_tls::TlsError;

fn rustls_error(label: &str) -> rustls::Error {
    rustls::Error::General(label.to_owned())
}

fn io_error(kind: io::ErrorKind) -> io::Error {
    io::Error::from(kind)
}

#[test]
fn every_public_error_has_deterministic_display_and_source_semantics() {
    let https_origin = Origin::parse("https://example.com").expect("HTTPS origin");
    let other_origin = Origin::parse("https://other.example").expect("other HTTPS origin");
    let peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 443);
    let other_peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8443);
    let errors = vec![
        (TlsError::InvalidTrustBundleIdentifier, false),
        (
            TlsError::InvalidTrustRootCount {
                root_count: 0,
                maximum_count: 256,
            },
            false,
        ),
        (
            TlsError::InvalidTrustRootBytes {
                byte_count: 3_000_000,
                maximum_bytes: 2_097_152,
            },
            false,
        ),
        (
            TlsError::InvalidTrustRoot {
                root_index: 2,
                source: rustls_error("invalid root"),
            },
            true,
        ),
        (
            TlsError::InvalidHandshakeTimeout {
                timeout: Duration::ZERO,
                maximum_timeout: Duration::from_secs(30),
            },
            false,
        ),
        (
            TlsError::InvalidMinimumLeafValidity {
                minimum_validity: Duration::from_secs(604_801),
                maximum_validity: Duration::from_secs(604_800),
            },
            false,
        ),
        (
            TlsError::InvalidAlpnCount {
                protocol_count: 9,
                maximum_count: 8,
            },
            false,
        ),
        (
            TlsError::InvalidAlpnIdentifier {
                protocol_index: 1,
                protocol_length: 0,
                maximum_length: 255,
            },
            false,
        ),
        (
            TlsError::DuplicateAlpnIdentifier { protocol_index: 2 },
            false,
        ),
        (
            TlsError::InvalidAlpnBytes {
                byte_count: 1_100,
                maximum_bytes: 1_024,
            },
            false,
        ),
        (
            TlsError::OriginRequiresHttps {
                origin: https_origin.clone(),
            },
            false,
        ),
        (
            TlsError::InvalidReferenceIdentity {
                origin: https_origin.clone(),
            },
            false,
        ),
        (
            TlsError::TransportOriginMismatch {
                tls_origin: https_origin.clone(),
                transport_origin: other_origin,
            },
            false,
        ),
        (
            TlsError::InheritedPeerMismatch {
                requested_peer: peer,
                observed_peer: peer,
                current_peer: other_peer,
            },
            false,
        ),
        (
            TlsError::PeerInspectionFailed {
                expected_peer: peer,
                source: io_error(io::ErrorKind::NotConnected),
            },
            true,
        ),
        (
            TlsError::TlsConfigurationFailed {
                source: rustls_error("configuration"),
            },
            true,
        ),
        (
            TlsError::HandshakeTimedOut {
                timeout: Duration::from_secs(3),
            },
            false,
        ),
        (
            TlsError::HandshakeIoFailed {
                source: io_error(io::ErrorKind::ConnectionReset),
            },
            true,
        ),
        (
            TlsError::UnknownIssuer {
                source: rustls_error("unknown issuer"),
            },
            true,
        ),
        (
            TlsError::CertificateExpired {
                source: rustls_error("expired"),
            },
            true,
        ),
        (
            TlsError::CertificateNotYetValid {
                source: rustls_error("future"),
            },
            true,
        ),
        (
            TlsError::ServiceIdentityMismatch {
                source: rustls_error("name mismatch"),
            },
            true,
        ),
        (
            TlsError::InvalidCertificate {
                source: rustls_error("invalid certificate"),
            },
            true,
        ),
        (
            TlsError::TlsProtocolFailed {
                source: rustls_error("protocol"),
            },
            true,
        ),
        (TlsError::MissingProtocolVersion, false),
        (TlsError::UnsupportedProtocolVersion, false),
        (TlsError::MissingCipherSuite, false),
        (TlsError::AlpnRequired, false),
        (TlsError::UnexpectedAlpn, false),
        (TlsError::MissingPeerCertificates, false),
        (
            TlsError::ExcessivePeerCertificateCount {
                certificate_count: 17,
                maximum_count: 16,
            },
            false,
        ),
        (
            TlsError::ExcessivePeerCertificateBytes {
                byte_count: 1_100_000,
                maximum_bytes: 1_048_576,
            },
            false,
        ),
        (TlsError::InvalidLeafCertificate, false),
        (
            TlsError::InsufficientLeafValidity {
                remaining_seconds: 86_400,
                minimum_seconds: 86_401,
            },
            false,
        ),
        (
            TlsError::TimeoutRestorationFailed {
                source: io_error(io::ErrorKind::InvalidInput),
            },
            true,
        ),
    ];

    for (error, has_source) in errors {
        let message = error.to_string();
        assert!(!message.is_empty(), "{error:?}");
        assert_eq!(error.source().is_some(), has_source, "{error:?}");
    }
}
