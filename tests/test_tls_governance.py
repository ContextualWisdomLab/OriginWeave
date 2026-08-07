"""Static contracts for the authenticated TLS service-identity boundary."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class TlsGovernanceTests(unittest.TestCase):
    """Prevent verifier bypass, reconnect, and ambiguous TLS evidence."""

    def _production_source(self) -> str:
        """Return all TLS production source as one searchable string."""

        source_root = ROOT / "crates/originweave-tls/src"
        return "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(source_root.glob("*.rs"))
        )

    def test_tls_source_uses_explicit_safe_rustls_configuration(self) -> None:
        """TLS must use roots, fixed time, safe versions, and explicit ALPN."""

        source = self._production_source()
        for required in [
            "ClientConfig::builder_with_details",
            "TimeProvider",
            "rustls::version::TLS13",
            "rustls::version::TLS12",
            "with_root_certificates",
            "Resumption::disabled",
            "NoKeyLog",
            "enable_early_data = false",
            "enable_secret_extraction = false",
            "check_selected_alpn = true",
            "peer_certificates",
            "alpn_protocol",
            "protocol_version",
            "negotiated_cipher_suite",
            "peer_addr",
        ]:
            self.assertIn(required, source)

    def test_tls_source_cannot_bypass_transport_or_identity_authority(self) -> None:
        """Production TLS must not reconnect, resolve, proxy, or replace WebPKI."""

        source = self._production_source()
        for forbidden in [
            ".dangerous()",
            "set_certificate_verifier",
            "TcpStream::connect",
            "ToSocketAddrs",
            "std::env",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "KeyLogFile",
            "with_client_auth_cert",
            "enable_early_data = true",
            "enable_secret_extraction = true",
        ]:
            self.assertNotIn(forbidden, source)

    def test_tls_manifest_pins_the_reviewed_security_dependencies(self) -> None:
        """The crate manifest must keep the reviewed TLS dependency surface."""

        manifest = (ROOT / "crates/originweave-tls/Cargo.toml").read_text(
            encoding="utf-8"
        )
        self.assertIn('rustls = { version = "=0.23.42"', manifest)
        self.assertIn('features = ["ring", "std", "tls12"]', manifest)
        self.assertIn('default-features = false', manifest)
        self.assertIn('sha2 = "=0.10.9"', manifest)
        self.assertIn('x509-parser = { version = "=0.18.1"', manifest)
        self.assertIn('rcgen = "=0.14.8"', manifest)

    def test_docs_keep_tcp_tls_http_proxy_and_chromium_separate(self) -> None:
        """TLS identity must be described without overstating product scope."""

        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        adr = (ROOT / "docs/adr/0006-tls-server-identity.md").read_text(
            encoding="utf-8"
        )
        for text in (architecture, adr):
            self.assertIn("TCP peer", text)
            self.assertIn("TLS service identity", text)
            self.assertIn("HTTP", text)
            self.assertIn("proxy", text.lower())
            self.assertIn("Chromium", text)
            self.assertIn("RFC 9525", text)


if __name__ == "__main__":
    unittest.main()
