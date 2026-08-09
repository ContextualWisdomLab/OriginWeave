"""Regression contracts for OriginWeave's authoritative product documentation graph."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ProductDocumentationContractTests(unittest.TestCase):
    """Keep product requirements, technical design, diagrams, and traceability discoverable."""

    def test_authoritative_product_documentation_graph_exists(self) -> None:
        """Major product decisions must not require reconstructing chat or PR history."""

        required_paths = {
            "docs/PRD.md",
            "docs/TRD.md",
            "docs/adr/README.md",
            "docs/uml/README.md",
            "docs/erd/README.md",
            "docs/traceability/README.md",
            "docs/THREAT_MODEL.md",
            "docs/TEST_STRATEGY.md",
            "docs/OPERABILITY.md",
            "docs/API_CONTRACT.md",
            "docs/RELEASE_AND_ROLLBACK.md",
        }
        missing = sorted(path for path in required_paths if not (ROOT / path).is_file())
        self.assertEqual(missing, [])

    def test_root_architecture_links_the_authoritative_product_graph(self) -> None:
        """Architecture readers must be able to reach requirements, decisions, diagrams, and data."""

        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        for link in (
            "docs/PRD.md",
            "docs/TRD.md",
            "docs/adr/README.md",
            "docs/uml/README.md",
            "docs/erd/README.md",
            "docs/traceability/README.md",
        ):
            with self.subTest(link=link):
                self.assertIn(link, architecture)

    def test_security_policy_links_the_product_threat_model(self) -> None:
        """Vulnerability reporters and operators must be able to find modeled trust boundaries."""

        security = (ROOT / "SECURITY.md").read_text(encoding="utf-8")
        self.assertIn("docs/THREAT_MODEL.md", security)

    def test_prd_covers_product_family_modes_and_buyer_acceptance(self) -> None:
        """The PRD must describe the actual product family rather than one kernel slice."""

        prd = (ROOT / "docs/PRD.md").read_text(encoding="utf-8")
        for phrase in (
            "Browse. Act. Prove.",
            "Human Mode",
            "Assist Mode",
            "Agent Task Mode",
            "Crawler Mode",
            "OriginWeave Browser",
            "OriginWeave Runtime",
            "OriginWeave Observe",
            "OriginWeave Capture",
            "OriginWeave Governor",
            "OriginWeave Policy",
            "OriginWeave Evidence",
            "OriginWeave Protocol",
            "Non-goals",
            "Buyer-visible acceptance",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, prd)

    def test_trd_distinguishes_shipped_architecture_from_future_work(self) -> None:
        """Technical documentation must not silently describe planned work as shipped."""

        trd = (ROOT / "docs/TRD.md").read_text(encoding="utf-8")
        for phrase in (
            "Implemented",
            "Accepted architecture",
            "Planned",
            "logical origin",
            "resolved destination",
            "TCP peer",
            "TLS service identity",
            "WebDriver BiDi",
            "Chrome DevTools Protocol",
            "WebMCP",
            "Model Context Protocol",
            "NVIDIA_NIM_API_KEY",
            "COPILOT_GITHUB_TOKEN",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, trd)

    def test_uml_and_erd_are_diagram_as_code(self) -> None:
        """Architecture flows and the conceptual domain model must be reviewable in Git."""

        uml = (ROOT / "docs/uml/README.md").read_text(encoding="utf-8")
        erd = (ROOT / "docs/erd/README.md").read_text(encoding="utf-8")
        self.assertGreaterEqual(uml.count("```mermaid"), 4)
        self.assertIn("sequenceDiagram", uml)
        self.assertIn("stateDiagram-v2", uml)
        self.assertIn("erDiagram", erd)
        for entity in (
            "agent_session",
            "browser_profile",
            "page_snapshot",
            "semantic_node",
            "action_event",
            "policy_decision",
            "provenance_record",
            "resource_budget",
        ):
            with self.subTest(entity=entity):
                self.assertIn(entity, erd)

    def test_operational_documents_preserve_fail_closed_product_boundaries(self) -> None:
        """Security, operations, APIs, tests, and rollback must agree on core authority boundaries."""

        documents = {
            "docs/THREAT_MODEL.md": (
                "renderer compromise",
                "prompt injection",
                "confused deputy",
                "cross-tenant",
            ),
            "docs/TEST_STRATEGY.md": (
                "true production boundary",
                "100%",
                "hostile",
                "protected-main",
            ),
            "docs/OPERABILITY.md": (
                "SLI",
                "SLO",
                "quarantine",
                "break-glass",
            ),
            "docs/API_CONTRACT.md": (
                "OriginWeave Protocol",
                "idempotency",
                "post-condition",
                "opaque",
            ),
            "docs/RELEASE_AND_ROLLBACK.md": (
                "SBOM",
                "provenance",
                "rollback",
                "protected main",
            ),
        }
        for path, phrases in documents.items():
            text = (ROOT / path).read_text(encoding="utf-8")
            for phrase in phrases:
                with self.subTest(path=path, phrase=phrase):
                    self.assertIn(phrase, text)

    def test_traceability_labels_conversation_derived_future_work(self) -> None:
        """Conversation decisions must preserve implementation status instead of becoming claims."""

        traceability = (ROOT / "docs/traceability/README.md").read_text(encoding="utf-8")
        for phrase in (
            "Implemented",
            "Accepted architecture",
            "Proposed",
            "Open",
            "conversation-derived",
            "docs/doctoring.md",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, traceability)


if __name__ == "__main__":
    unittest.main()
