#!/usr/bin/env python3
"""Apply the deterministic TLS branch repair before exact-head verification."""

from __future__ import annotations

from pathlib import Path


def replace_once(text: str, before: str, after: str, description: str) -> str:
    """Replace one exact marker and reject ambiguous or stale source."""

    count = text.count(before)
    if count != 1:
        raise SystemExit(f"expected one {description}, found {count}")
    return text.replace(before, after, 1)


def repair_handshake() -> None:
    """Type ALPN negotiation failures and remove unused imports."""

    path = Path("crates/originweave-tls/src/handshake.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "use std::net::{SocketAddr, TcpStream};",
        "use std::net::TcpStream;",
        "unused SocketAddr import",
    )
    text = replace_once(
        text,
        "use rustls::pki_types::{CertificateDer, ServerName, UnixTime};",
        "use rustls::pki_types::{CertificateDer, UnixTime};",
        "unused ServerName import",
    )
    text = replace_once(
        text,
        "use rustls::{CertificateError, ClientConfig, ClientConnection, ProtocolVersion, StreamOwned};",
        "use rustls::{\n    AlertDescription, CertificateError, ClientConfig, ClientConnection, ProtocolVersion,\n    StreamOwned,\n};",
        "rustls import list",
    )
    text = replace_once(
        text,
        "ClientConnection::new(Arc::new(config), server_name).map_err(classify_rustls_error)?;",
        "ClientConnection::new(Arc::new(config), server_name)\n                .map_err(|source| classify_rustls_error(source, alpn_requirement))?;",
        "client construction error mapping",
    )
    text = replace_once(
        text,
        "            handshake_timeout,\n        );",
        "            handshake_timeout,\n            alpn_requirement,\n        );",
        "handshake call argument list",
    )
    text = replace_once(
        text,
        "    handshake_timeout: Duration,\n) -> Result<(), TlsError> {",
        "    handshake_timeout: Duration,\n    alpn_requirement: AlpnRequirement,\n) -> Result<(), TlsError> {",
        "handshake function signature",
    )
    text = replace_once(
        text,
        "            client\n                .process_new_packets()\n                .map_err(classify_rustls_error)?;",
        "            client\n                .process_new_packets()\n                .map_err(|source| classify_rustls_error(source, alpn_requirement))?;",
        "packet-processing error mapping",
    )
    text = replace_once(
        text,
        "fn classify_rustls_error(source: rustls::Error) -> TlsError {",
        "fn classify_rustls_error(\n    source: rustls::Error,\n    alpn_requirement: AlpnRequirement,\n) -> TlsError {",
        "rustls error classifier signature",
    )
    text = replace_once(
        text,
        "        InvalidCertificate,\n        Protocol,",
        "        InvalidCertificate,\n        AlpnRequired,\n        UnexpectedAlpn,\n        Protocol,",
        "classifier variants",
    )
    text = replace_once(
        text,
        "    let classification = match &source {\n        rustls::Error::InvalidCertificate(certificate_error) => match certificate_error {",
        "    let classification = match &source {\n        rustls::Error::NoApplicationProtocol\n        | rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol) => {\n            if alpn_requirement == AlpnRequirement::Required {\n                Classification::AlpnRequired\n            } else {\n                Classification::UnexpectedAlpn\n            }\n        }\n        rustls::Error::InvalidCertificate(certificate_error) => match certificate_error {",
        "ALPN alert classification",
    )
    text = replace_once(
        text,
        "        Classification::InvalidCertificate => TlsError::InvalidCertificate { source },\n        Classification::Protocol => TlsError::TlsProtocolFailed { source },",
        "        Classification::InvalidCertificate => TlsError::InvalidCertificate { source },\n        Classification::AlpnRequired => TlsError::AlpnRequired,\n        Classification::UnexpectedAlpn => TlsError::UnexpectedAlpn,\n        Classification::Protocol => TlsError::TlsProtocolFailed { source },",
        "typed ALPN classifier output",
    )
    text = text.replace(
        "classify_rustls_error(rustls::Error::InvalidCertificate(certificate_error))",
        "classify_rustls_error(\n                rustls::Error::InvalidCertificate(certificate_error),\n                AlpnRequirement::Optional,\n            )",
    )
    text = replace_once(
        text,
        'let protocol = classify_rustls_error(rustls::Error::General("test".to_owned()));',
        'let protocol = classify_rustls_error(\n            rustls::Error::General("test".to_owned()),\n            AlpnRequirement::Optional,\n        );',
        "protocol classifier test",
    )
    marker = "    #[test]\n    fn certificate_bounds_are_fail_closed() {"
    alpn_test = "\n".join(
        [
            "    #[test]",
            "    fn no_application_protocol_is_typed_by_policy() {",
            "        for source in [",
            "            rustls::Error::NoApplicationProtocol,",
            "            rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol),",
            "        ] {",
            "            let required = classify_rustls_error(source, AlpnRequirement::Required);",
            "            assert!(matches!(required, TlsError::AlpnRequired));",
            "        }",
            "        for source in [",
            "            rustls::Error::NoApplicationProtocol,",
            "            rustls::Error::AlertReceived(AlertDescription::NoApplicationProtocol),",
            "        ] {",
            "            let optional = classify_rustls_error(source, AlpnRequirement::Optional);",
            "            assert!(matches!(optional, TlsError::UnexpectedAlpn));",
            "        }",
            "    }",
            "",
        ]
    )
    path.write_text(
        replace_once(text, marker, alpn_test + marker, "certificate-bound test marker"),
        encoding="utf-8",
    )


def repair_integration_tests() -> None:
    """Give unrelated roots distinct subjects and accept server-side ALPN alerts."""

    path = Path("crates/originweave-tls/tests/handshake_integration.rs")
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "fn certificate_authority() -> (Vec<u8>, Issuer<'static, KeyPair>) {",
        "fn certificate_authority(common_name: &str) -> (Vec<u8>, Issuer<'static, KeyPair>) {",
        "certificate authority helper signature",
    )
    text = replace_once(
        text,
        '.push(DnType::CommonName, "OriginWeave test root");',
        ".push(DnType::CommonName, common_name);",
        "certificate authority common name",
    )
    text = replace_once(
        text,
        "let (root_der, issuer) = certificate_authority();",
        'let (root_der, issuer) = certificate_authority("OriginWeave test root");',
        "leaf issuer creation",
    )
    text = replace_once(
        text,
        "let (untrusted_root, _issuer) = certificate_authority();",
        'let (untrusted_root, _issuer) = certificate_authority("Unrelated test root");',
        "untrusted root creation",
    )
    text = replace_once(
        text,
        '    assert!(matches!(error, TlsError::AlpnRequired));\n    assert_eq!(server.join().expect("server thread"), Ok(None));',
        '    assert!(matches!(error, TlsError::AlpnRequired));\n    let _server_result = server.join().expect("server thread");',
        "required ALPN server result",
    )
    path.write_text(text, encoding="utf-8")


def repair_adr() -> None:
    """Replace the obsoleted TLS 1.3 reference with the current RFC."""

    path = Path("docs/adr/0006-tls-server-identity.md")
    text = path.read_text(encoding="utf-8")
    path.write_text(
        replace_once(
            text,
            "RFC 5280 defines the Internet PKIX certificate and CRL profile. RFC 8446 defines TLS 1.3. RFC 9525 defines service identity for TLS, requires applicable subjectAltName identifiers, and supersedes the older RFC 6125 guidance.",
            "RFC 5280 defines the Internet PKIX certificate and CRL profile. RFC 9846 defines TLS 1.3 and obsoletes RFC 8446. RFC 9525 defines service identity for TLS, requires applicable subjectAltName identifiers, and supersedes the older RFC 6125 guidance.",
            "stale TLS standards sentence",
        ),
        encoding="utf-8",
    )


def main() -> int:
    """Apply every reviewed deterministic repair."""

    repair_handshake()
    repair_integration_tests()
    repair_adr()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
