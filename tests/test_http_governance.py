"""Repository-level security contracts for the bounded HTTP/1.1 authority."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
HTTP_ROOT = ROOT / "crates/originweave-http"

EXPECTED_PRODUCTION_MODULES = {
    "lib.rs",
    "error.rs",
    "policy.rs",
    "target.rs",
    "field.rs",
    "request.rs",
    "response_head.rs",
    "framing.rs",
    "evidence.rs",
    "exchange.rs",
}

FORBIDDEN_HTTP_PRODUCTION_TOKENS = (
    "TcpStream::connect",
    "connect_timeout",
    "ToSocketAddrs",
    "lookup_host",
    "std::fs::write",
    "File::create",
    "reqwest",
    "hyper::client",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "gh pr",
    "COPILOT_GITHUB_TOKEN",
)


class HttpGovernanceTests(unittest.TestCase):
    """Keep HTTP parsing separate from network, proxy, browser, and persistence authority."""

    def test_fixed_length_slice_has_the_reviewed_module_boundary(self) -> None:
        """The first HTTP slice must keep pure parsing separate from stream orchestration."""

        source_root = HTTP_ROOT / "src"
        present = {path.name for path in source_root.glob("*.rs")}
        self.assertTrue(EXPECTED_PRODUCTION_MODULES.issubset(present))

    def test_production_http_code_cannot_create_an_alternate_authority_path(self) -> None:
        """HTTP must consume the authenticated TLS stream rather than reconnect or proxy."""

        source_root = HTTP_ROOT / "src"
        production = "\n".join(
            path.read_text(encoding="utf-8") for path in sorted(source_root.glob("*.rs"))
        )
        for token in FORBIDDEN_HTTP_PRODUCTION_TOKENS:
            self.assertNotIn(token, production, token)
        self.assertNotIn("unsafe {", production)

    def test_design_plan_and_adr_are_binding_repository_artifacts(self) -> None:
        """The authority boundary must remain discoverable without reading implementation code."""

        for relative in [
            "docs/superpowers/specs/2026-08-07-http11-semantics-design.md",
            "docs/superpowers/plans/2026-08-07-http11-semantics.md",
            "docs/adr/0007-bounded-http11-semantics.md",
        ]:
            self.assertTrue((ROOT / relative).is_file(), relative)

    def test_public_docs_do_not_overstate_the_fixed_length_slice(self) -> None:
        """Documentation must distinguish implemented fixed-length HTTP from later features."""

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        for relative, text in [("README.md", readme), ("ARCHITECTURE.md", architecture)]:
            self.assertIn("originweave-http", text, relative)
            self.assertIn("Content-Length", text, relative)
            self.assertIn("authenticated TLS stream", text, relative)
            self.assertIn("no reconnect", text.lower(), relative)


if __name__ == "__main__":
    unittest.main()
