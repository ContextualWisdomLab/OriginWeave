"""Static contracts for the direct-only TCP authority boundary."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class NetworkGovernanceTests(unittest.TestCase):
    """Prevent DNS, proxy, and protocol scope from entering the socket kernel."""

    def test_network_source_uses_one_explicit_socket_address(self) -> None:
        """The network kernel must connect only to one validated SocketAddr."""

        source = (
            ROOT / "crates/originweave-network/src/connection.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("TcpStream::connect_timeout", source)
        self.assertIn("SocketAddr", source)
        self.assertNotIn("ToSocketAddrs", source)
        self.assertNotIn("TcpStream::connect(", source)

    def test_network_source_does_not_inherit_proxy_environment(self) -> None:
        """Direct-only routing must not inspect ambient proxy variables."""

        source = (
            ROOT / "crates/originweave-network/src/connection.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("std::env", source)
        self.assertNotIn("HTTP_PROXY", source)
        self.assertNotIn("HTTPS_PROXY", source)
        self.assertNotIn("ALL_PROXY", source)

    def test_docs_keep_transport_scope_explicit(self) -> None:
        """Architecture and ADR must preserve the independent adapter boundaries."""

        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        adr = (ROOT / "docs/adr/0005-direct-socket-binding.md").read_text(
            encoding="utf-8"
        )
        for text in (architecture, adr):
            self.assertIn("direct-only", text)
            self.assertIn("TLS", text)
            self.assertIn("proxy", text.lower())
            self.assertIn("Chromium", text)


if __name__ == "__main__":
    unittest.main()
