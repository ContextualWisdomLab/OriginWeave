"""Architectural documentation fitness for bounded-context ownership."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class DddDocumentationContractTests(unittest.TestCase):
    """Keep the Context Map and Ubiquitous Language executable as repository contracts."""

    def test_context_map_records_mcp_as_protocol_adapter_not_shared_kernel(self) -> None:
        context_map = (ROOT / "docs/context-map.md").read_text(encoding="utf-8")
        self.assertIn("originweave-mcp", context_map)
        self.assertIn("Anti-Corruption Layer", context_map)
        self.assertIn("MCP adapter", context_map)
        self.assertIn("originweave-core", context_map)
        self.assertIn("originweave-policy", context_map)
        self.assertIn("must not depend outward on protocol adapters", context_map)
        self.assertIn("MCP adapter → policy decision", context_map)
        self.assertNotIn("Policy decision → MCP adapter", context_map)
        self.assertIn("policy depends only on `originweave-core`", context_map)

    def test_ubiquitous_language_keeps_authority_terms_distinct(self) -> None:
        glossary = (ROOT / "docs/ubiquitous-language.md").read_text(encoding="utf-8")
        for term in (
            "Origin",
            "Destination",
            "Resolution Snapshot",
            "Direct TCP Connection",
            "TLS Service Identity",
            "Action Request",
            "Approval Scope",
            "Policy Decision",
            "MCP Route",
            "Anti-Corruption Layer",
            "Protected-main truth",
        ):
            with self.subTest(term=term):
                self.assertIn(term, glossary)
        self.assertIn("Origin ≠ destination ≠ TCP peer ≠ TLS service identity", glossary)
        self.assertIn("MCP route ≠ policy decision ≠ browser execution", glossary)

    def test_mcp_traceability_names_current_and_active_ownership_separately(self) -> None:
        trace = (ROOT / "docs/traceability/mcp-authority-route.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)
        self.assertIn("active PR #272", trace)
        self.assertIn("crates/originweave-core/src/mcp.rs", trace)
        self.assertIn("crates/originweave-mcp/src/routing.rs", trace)
        self.assertIn("originweave_mcp::evaluate_mcp", trace)
        self.assertIn("originweave_policy::evaluate", trace)
        self.assertIn("protocol-independent", trace)
        self.assertIn("active-PR evidence, not protected-main behavior", trace)


if __name__ == "__main__":
    unittest.main()
