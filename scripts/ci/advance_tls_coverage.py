#!/usr/bin/env python3
"""Apply the first deterministic TLS coverage-closing refactor."""

from __future__ import annotations

from pathlib import Path


def replace_once(text: str, before: str, after: str, description: str) -> str:
    """Replace exactly one reviewed source marker."""

    count = text.count(before)
    if count != 1:
        raise SystemExit(f"expected one {description}, found {count}")
    return text.replace(before, after, 1)


def update_origin_contract() -> None:
    """Expose the already validated scheme and unbracketed host."""

    path = Path("crates/originweave-core/src/lib.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "pub struct Origin {\n    canonical: String,\n}",
        "pub struct Origin {\n    canonical: String,\n    scheme: String,\n    host: String,\n}",
        "Origin fields",
    )
    text = replace_once(
        text,
        '''        let normalized_port = normalize_default_port(&scheme, port);
        let canonical = match normalized_port {
            Some(port_number) => format!("{scheme}://{host}:{port_number}"),
            None => format!("{scheme}://{host}"),
        };
        Ok(Self { canonical })
''',
        '''        let reference_host = if host.starts_with('[') {
            host[1..host.len() - 1].to_owned()
        } else {
            host.clone()
        };
        let normalized_port = normalize_default_port(&scheme, port);
        let canonical = match normalized_port {
            Some(port_number) => format!("{scheme}://{host}:{port_number}"),
            None => format!("{scheme}://{host}"),
        };
        Ok(Self {
            canonical,
            scheme,
            host: reference_host,
        })
''',
        "Origin construction",
    )
    text = replace_once(
        text,
        '''    pub fn as_str(&self) -> &str {
        &self.canonical
    }
''',
        '''    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Return the validated lowercase origin scheme.
    #[must_use]
    pub const fn scheme(&self) -> &str {
        self.scheme.as_str()
    }

    /// Return the validated canonical host without IPv6 brackets.
    #[must_use]
    pub const fn host(&self) -> &str {
        self.host.as_str()
    }
''',
        "Origin accessors",
    )
    path.write_text(text, encoding="utf-8")

    path = Path("crates/originweave-core/tests/contracts.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        '''    assert_eq!(secure_ipv6.as_str(), "https://[2001:db8::1]");
    assert_eq!(secure.to_string(), secure.as_str());
''',
        '''    assert_eq!(secure_ipv6.as_str(), "https://[2001:db8::1]");
    assert_eq!(secure.scheme(), "https");
    assert_eq!(secure.host(), "example.com");
    assert_eq!(localhost.scheme(), "http");
    assert_eq!(localhost.host(), "localhost");
    assert_eq!(ipv4.host(), "127.0.0.1");
    assert_eq!(ipv6.host(), "::1");
    assert_eq!(secure_ipv6.host(), "2001:db8::1");
    assert_eq!(secure.to_string(), secure.as_str());
''',
        "Origin accessor assertions",
    )
    path.write_text(text, encoding="utf-8")


def replace_identity_module() -> None:
    """Derive identity from structured Origin invariants without reparsing authority."""

    path = Path("crates/originweave-tls/src/identity.rs")
    path.write_text(
        '''use std::net::IpAddr;

use originweave_core::Origin;
use rustls::pki_types::ServerName;

use crate::TlsError;

/// The RFC 9525 reference identity derived from one canonical HTTPS origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsReferenceIdentity {
    /// A DNS reference identity that permits a matching SNI value.
    Dns(String),
    /// A literal IPv4 or IPv6 reference identity that sends no SNI value.
    Ip(IpAddr),
}

impl TlsReferenceIdentity {
    /// Derive a TLS reference identity from a canonical OriginWeave origin.
    pub fn from_origin(origin: &Origin) -> Result<Self, TlsError> {
        if origin.scheme() != "https" {
            return Err(TlsError::OriginRequiresHttps {
                origin: origin.clone(),
            });
        }
        if let Ok(address) = origin.host().parse::<IpAddr>() {
            return Ok(Self::Ip(address));
        }
        Ok(Self::Dns(origin.host().to_owned()))
    }

    pub(crate) fn server_name(&self, origin: &Origin) -> Result<ServerName<'static>, TlsError> {
        match self {
            Self::Dns(name) => ServerName::try_from(name.clone()).map_err(|_error| {
                TlsError::InvalidReferenceIdentity {
                    origin: origin.clone(),
                }
            }),
            Self::Ip(address) => Ok(ServerName::IpAddress((*address).into())),
        }
    }

    pub(crate) const fn uses_sni(&self) -> bool {
        matches!(self, Self::Dns(_))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn explicit_invalid_dns_variant_fails_closed() {
        let origin = Origin::parse("https://example.com").expect("HTTPS origin");
        let identity = TlsReferenceIdentity::Dns("contains space".to_owned());
        assert!(matches!(
            identity.server_name(&origin),
            Err(TlsError::InvalidReferenceIdentity { .. })
        ));
    }

    #[test]
    fn sni_is_used_only_for_dns_identity() {
        assert!(TlsReferenceIdentity::Dns("example.com".to_owned()).uses_sni());
        assert!(!TlsReferenceIdentity::Ip(IpAddr::V4(
            std::net::Ipv4Addr::LOCALHOST
        ))
        .uses_sni());
    }
}
''',
        encoding="utf-8",
    )


def update_handshake_helpers() -> None:
    """Remove unreachable integer/deadline overflow branches and test expiry."""

    path = Path("crates/originweave-tls/src/handshake.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        '''        let started_at = Instant::now();
        let deadline =
            started_at
                .checked_add(handshake_timeout)
                .ok_or(TlsError::InvalidHandshakeTimeout {
                    timeout: handshake_timeout,
                    maximum_timeout: crate::MAX_TLS_HANDSHAKE_TIMEOUT,
                })?;
''',
        '''        let started_at = Instant::now();
        let deadline = started_at + handshake_timeout;
''',
        "bounded handshake deadline",
    )
    text = replace_once(
        text,
        '''    let byte_count = certificates.iter().try_fold(0_usize, |total, certificate| {
        total.checked_add(certificate.len())
    });
    let Some(byte_count) = byte_count else {
        return Err(TlsError::ExcessivePeerCertificateBytes {
            byte_count: usize::MAX,
            maximum_bytes: MAX_SERVER_CERTIFICATE_BYTES,
        });
    };
''',
        '''    let byte_count = certificates.iter().fold(0_usize, |total, certificate| {
        total.saturating_add(certificate.len())
    });
''',
        "certificate byte accumulation",
    )
    marker = "    #[test]\n    fn timeout_io_classification_is_explicit() {"
    test = '''    #[test]
    fn an_elapsed_deadline_is_typed_as_timeout() {
        let timeout = Duration::from_secs(1);
        assert!(matches!(
            remaining_time(Instant::now(), timeout),
            Err(TlsError::HandshakeTimedOut { timeout: observed }) if observed == timeout
        ));
    }

'''
    text = replace_once(text, marker, test + marker, "timeout classifier test marker")
    path.write_text(text, encoding="utf-8")


def update_integration_coverage() -> None:
    """Exercise every public evidence and authenticated-stream accessor."""

    path = Path("crates/originweave-tls/tests/handshake_integration.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        '''    TlsHandshakePlan, TlsProtocolVersion, TrustBundleIdentifier, TrustRootBundle,
''',
        '''    TlsHandshakePlan, TlsProtocolVersion, TlsReferenceIdentity, TrustBundleIdentifier,
    TrustRootBundle,
''',
        "TLS integration imports",
    )
    text = replace_once(
        text,
        '''    let authenticated = TlsHandshakePlan::new(
''',
        '''    let mut authenticated = TlsHandshakePlan::new(
''',
        "primary authenticated connection binding",
    )
    text = replace_once(
        text,
        '''    let evidence = authenticated.evidence();
    assert_eq!(evidence.origin(), &origin);
''',
        '''    let evidence = authenticated.evidence().clone();
    assert_eq!(evidence.origin(), &origin);
''',
        "primary evidence clone",
    )
    text = replace_once(
        text,
        '''    assert_eq!(evidence.trusted_time_unix_seconds(), TRUSTED_TIME_SECONDS);
    assert!(evidence.handshake_duration() <= evidence.handshake_timeout());
''',
        '''    assert_eq!(evidence.trusted_time_unix_seconds(), TRUSTED_TIME_SECONDS);
    assert!(evidence.handshake_duration() <= evidence.handshake_timeout());
    assert_eq!(
        evidence.reference_identity(),
        &TlsReferenceIdentity::Dns("localhost".to_owned())
    );
    assert_ne!(evidence.cipher_suite_identifier(), [0_u8; 2]);
    assert!(!evidence.cipher_suite_label().is_empty());
    assert_eq!(evidence.presented_certificate_hashes().len(), 1);
    assert!(evidence
        .presented_certificate_hashes()
        .iter()
        .all(|identifier| identifier.starts_with("sha256:")));
    assert!(
        evidence.leaf_not_before_unix_seconds()
            < evidence.leaf_not_after_unix_seconds()
    );
    assert_eq!(
        evidence.trust_bundle_identifier().as_str(),
        "local_test_roots:v1"
    );
''',
        "evidence accessor assertions",
    )
    text = replace_once(
        text,
        '''    assert_eq!(
        server.join().expect("server thread"),
        Ok(Some(b"h2".to_vec()))
    );
}

#[test]
fn optional_alpn_records_explicit_absence() {
''',
        '''    assert_eq!(
        authenticated
            .stream()
            .sock
            .peer_addr()
            .expect("authenticated peer"),
        socket_address
    );
    let _mutable_stream = authenticated.stream_mut();
    let (_stream, consumed_evidence) = authenticated.into_parts();
    assert_eq!(consumed_evidence, evidence);
    assert_eq!(
        server.join().expect("server thread"),
        Ok(Some(b"h2".to_vec()))
    );
}

#[test]
fn optional_alpn_records_explicit_absence() {
''',
        "authenticated stream accessor assertions",
    )
    marker = "#[test]\nfn required_alpn_rejects_no_common_protocol() {"
    test = '''#[test]
fn required_alpn_rejects_explicit_server_absence() {
    let material = valid_material(vec!["localhost".to_owned()], None);
    let (root_der, config) = server_config(
        material,
        &[],
        &[&rustls::version::TLS13, &rustls::version::TLS12],
    );
    let (socket_address, server) = spawn_server(IpAddr::V4(Ipv4Addr::LOCALHOST), config);
    let origin = origin_for("localhost", socket_address);
    let error = TlsHandshakePlan::new(
        origin.clone(),
        direct_connection(&origin, socket_address),
        trust_bundle(root_der, "required_absence:v1"),
        client_policy(&[b"h2"], AlpnRequirement::Required),
    )
    .expect("required ALPN plan")
    .authenticate()
    .expect_err("explicit ALPN absence must fail policy");
    assert!(matches!(error, TlsError::AlpnRequired));
    assert_eq!(server.join().expect("server thread"), Ok(None));
}

'''
    text = replace_once(text, marker, test + marker, "required ALPN test marker")
    path.write_text(text, encoding="utf-8")


def main() -> int:
    """Apply every first-pass coverage closure."""

    update_origin_contract()
    replace_identity_module()
    update_handshake_helpers()
    update_integration_coverage()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
