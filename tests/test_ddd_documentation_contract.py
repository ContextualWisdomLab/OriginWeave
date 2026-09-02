"""Architectural documentation fitness for bounded-context ownership."""

from __future__ import annotations

import pathlib
import tomllib
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
        self.assertIn("route/action mismatch remains adapter-owned", context_map)

    def test_cargo_dependency_direction_enforces_mcp_anti_corruption_layer(self) -> None:
        manifests: dict[str, dict[str, object]] = {}
        for crate in ("originweave-core", "originweave-policy", "originweave-mcp"):
            manifest_path = ROOT / "crates" / crate / "Cargo.toml"
            manifests[crate] = tomllib.loads(manifest_path.read_text(encoding="utf-8"))

        core_dependencies = manifests["originweave-core"].get("dependencies", {})
        policy_dependencies = manifests["originweave-policy"].get("dependencies", {})
        mcp_dependencies = manifests["originweave-mcp"].get("dependencies", {})

        self.assertNotIn("originweave-mcp", core_dependencies)
        self.assertNotIn("originweave-mcp", policy_dependencies)
        self.assertEqual(
            policy_dependencies.get("originweave-core"),
            {"path": "../originweave-core"},
        )
        self.assertEqual(
            mcp_dependencies.get("originweave-core"),
            {"path": "../originweave-core"},
        )
        self.assertEqual(
            mcp_dependencies.get("originweave-policy"),
            {"path": "../originweave-policy"},
        )

    def test_context_map_names_product_bounded_contexts_explicitly(self) -> None:
        context_map = (ROOT / "docs/context-map.md").read_text(encoding="utf-8")
        for bounded_context in (
            "Browser Session",
            "Profile/Identity Boundary",
            "Semantic Observation",
            "Typed Action Execution",
            "Resource Governance",
            "Scraping/Extraction",
            "Provenance",
            "Extension Policy",
            "Secret Broker",
            "Agent Integration",
        ):
            with self.subTest(bounded_context=bounded_context):
                self.assertIn(bounded_context, context_map)
        self.assertIn("protected-main", context_map)
        self.assertIn("active PR", context_map)
        self.assertIn("planned", context_map)
        self.assertIn("does not transfer authority", context_map)

    def test_tactical_ddd_map_keeps_patterns_truthful_and_protocol_independent(self) -> None:
        tactical_map = (ROOT / "docs/tactical-ddd-map.md").read_text(encoding="utf-8")
        for pattern in (
            "Value Object",
            "Entity",
            "Aggregate / Aggregate Root",
            "Domain Service",
            "Repository",
            "Domain Event",
            "Invariant",
        ):
            with self.subTest(pattern=pattern):
                self.assertIn(pattern, tactical_map)

        self.assertIn("No explicit mutable domain Entity", tactical_map)
        self.assertIn("No explicit Aggregate Root", tactical_map)
        self.assertIn("No domain Repository contract", tactical_map)
        self.assertIn("No explicit OriginWeave domain-event type", tactical_map)
        self.assertIn("originweave_policy::evaluate", tactical_map)
        self.assertIn("browsingContext.navigationCommitted", tactical_map)
        self.assertIn("integration events", tactical_map)
        self.assertIn("A command ACK is likewise not a post-condition event", tactical_map)
        self.assertIn("multi-word `snake_case`", tactical_map)
        self.assertIn("context-graph-contracts", tactical_map)
        self.assertIn("must not import MCP, WebDriver BiDi, CDP", tactical_map)

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
            "MCP Route Rejection",
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
        self.assertIn("McpRouteRejection", trace)
        self.assertIn("originweave_policy::evaluate", trace)
        self.assertIn("protocol-independent", trace)
        self.assertIn("active-PR evidence, not protected-main behavior", trace)


if __name__ == "__main__":
    unittest.main()
