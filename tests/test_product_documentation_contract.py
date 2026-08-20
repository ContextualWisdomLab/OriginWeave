"""Regression contracts for OriginWeave's authoritative product documentation graph."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ProductDocumentationContractTests(unittest.TestCase):
    """Keep product requirements, technical design, diagrams, and traceability discoverable."""

    @staticmethod
    def _subsection(text: str, heading: str) -> str:
        """Return one fourth-level documentation subsection."""
        start = text.index(heading) + len(heading)
        remainder = text[start:]
        end = remainder.find("\n#### ")
        return remainder if end == -1 else remainder[:end]

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
            "docs/product-technical-gap-baseline.md",
        }
        missing = sorted(path for path in required_paths if not (ROOT / path).is_file())
        self.assertEqual(missing, [])

    def test_product_technical_gap_baseline_records_live_delivery_state(self) -> None:
        """Buyers and maintainers must see implementation gaps and current delivery blockers together."""
        baseline = ROOT / "docs/product-technical-gap-baseline.md"
        self.assertTrue(baseline.is_file())
        text = baseline.read_text(encoding="utf-8")
        for phrase in (
            "Observed snapshot: 2026-08-20",
            "Protected-main truth",
            "Open pull requests",
            "Open issues",
            "#195",
            "#149",
            "reviewer-provisioning gap",
            "Phase 1",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

        protected_main = text.split("### Open pull requests", 1)[0]
        open_pull_requests = text.split("### Open pull requests", 1)[1].split(
            "### Review and merge authority", 1
        )[0]
        self.assertIn("Phase 1 is **in progress**, not shipped.", protected_main)
        self.assertIn(
            "It remains draft evidence and cannot be treated as shipped behavior.",
            open_pull_requests,
        )
        bidi_status = self._subsection(
            open_pull_requests, "#### #195/#198 WebDriver BiDi opening path status"
        )
        vpn_status = self._subsection(
            open_pull_requests, "#### #149 VPN/profile intent status"
        )
        self.assertIn("Phase 1 is **in progress**, not shipped.", bidi_status)
        self.assertIn(
            "It remains draft evidence and cannot be treated as shipped behavior.",
            vpn_status,
        )

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
            "docs/product-technical-gap-baseline.md",
        ):
            with self.subTest(link=link):
                self.assertIn(link, architecture)

    def test_security_policy_links_the_product_threat_model(self) -> None:
        """Vulnerability reporters and operators must be able to find modeled trust boundaries."""
        self.assertIn(
            "docs/THREAT_MODEL.md",
            (ROOT / "SECURITY.md").read_text(encoding="utf-8"),
        )

    def test_agent_contract_is_work_conserving_instead_of_one_action_per_run(self) -> None:
        """Finishing one bounded slice must return maintenance to the live queue."""
        contract = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
        for phrase in (
            "A completed action is an intermediate state",
            "one write-active slice at a time",
            "Mandatory exit sweep",
            "termination is prohibited",
            "blocks only that item",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, contract)

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

    def test_target_architecture_adr_set_is_detailed(self) -> None:
        """Product direction must be reconstructable from durable, reviewable decisions."""
        required_adrs = {
            "docs/adr/0001-chromium-compatibility-kernel.md": (
                "Chromium",
                "browser-engine rewrite",
            ),
            "docs/adr/0100-rust-control-plane-boundary.md": (
                "Rust control plane",
                "Chromium compatibility kernel",
            ),
            "docs/adr/0101-isolated-execution-profile-modes.md": (
                "Human",
                "Assist",
                "Agent Task",
                "Crawler",
            ),
            "docs/adr/0102-typed-actions-and-arbitrary-js.md": (
                "typed action",
                "arbitrary JavaScript",
            ),
            "docs/adr/0103-semantic-observation-and-stale-node-identity.md": (
                "WebMCP",
                "accessibility",
                "document epoch",
                "stale",
            ),
            "docs/adr/0104-prompt-injection-and-secret-authority.md": (
                "prompt injection",
                "opaque",
                "secret",
            ),
            "docs/adr/0105-resource-governor-priority.md": (
                "resource governor",
                "GPU",
                "browser",
                "model",
            ),
            "docs/adr/0106-provenance-evidence-model.md": (
                "WARC",
                "PROV",
                "evidence",
            ),
            "docs/adr/0107-browser-protocol-adapter-strategy.md": (
                "WebDriver BiDi",
                "Chrome DevTools Protocol",
                "WebMCP",
                "Model Context Protocol",
            ),
            "docs/adr/0108-crawler-policy.md": ("robots", "rate", "CAPTCHA"),
            "docs/adr/0109-hourly-automation-operational-closure.md": (
                "NVIDIA_NIM_API_KEY",
                "protected-main",
                "open_pull_request",
            ),
        }
        sections = (
            "## Context",
            "## Options considered",
            "## Decision",
            "## Consequences",
            "## Failure and degraded behavior",
            "## Security / privacy / governance impact",
            "## Tests and acceptance evidence",
            "## Migration and rollback",
            "## Supersession / reversal conditions",
        )
        fields = ("- Status:", "- Date:", "- Supersedes:", "- Superseded by:")
        for path, phrases in required_adrs.items():
            with self.subTest(path=path):
                text = (ROOT / path).read_text(encoding="utf-8")
                for field in fields:
                    self.assertIn(field, text)
                for section in sections:
                    self.assertIn(section, text)
                for phrase in phrases:
                    self.assertIn(phrase, text)

    def test_stale_node_adr_defines_action_linearization_race(self) -> None:
        """A mutation between handle validation and dispatch must never produce a stale side effect."""
        adr = (
            ROOT / "docs/adr/0103-semantic-observation-and-stale-node-identity.md"
        ).read_text(encoding="utf-8")
        for phrase in (
            "action linearization point",
            "side effect",
            "competing mutation",
            "re-observation",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, adr)

    def test_hourly_automation_adr_requires_exit_sweep(self) -> None:
        """Automation closure must re-sweep all actionable lanes instead of stopping after one result."""
        adr = (
            ROOT / "docs/adr/0109-hourly-automation-operational-closure.md"
        ).read_text(encoding="utf-8")
        for phrase in (
            "mandatory exit sweep",
            "open OriginWeave PRs and issues",
            "release state",
            "documentation",
            "product gaps",
            "safe actionable work remains",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, adr)

    def test_uml_and_erd_are_diagram_as_code(self) -> None:
        """Architecture flows and the conceptual domain model must be reviewable in Git."""
        uml = (ROOT / "docs/uml/README.md").read_text(encoding="utf-8")
        authority_view = ROOT / "docs/uml/extension-authority.md"
        self.assertTrue(authority_view.is_file())
        self.assertIn("](extension-authority.md)", uml)
        self.assertIn("```mermaid", authority_view.read_text(encoding="utf-8"))

        erd = (ROOT / "docs/erd/README.md").read_text(encoding="utf-8")
        self.assertGreaterEqual(uml.count("```mermaid"), 8)
        self.assertIn("sequenceDiagram", uml)
        self.assertIn("stateDiagram-v2", uml)
        for heading in (
            "Secret-fill sequence",
            "Read/write risk approval flow",
            "Resource-pressure and fallback flow",
            "Hourly product-development gate-to-model flow",
        ):
            with self.subTest(heading=heading):
                self.assertIn(heading, uml)
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

    def test_hourly_uml_fails_closed_before_secret_or_publication(self) -> None:
        """Denied credentials and failed validation must terminate before secret use or publication."""
        uml = (ROOT / "docs/uml/README.md").read_text(encoding="utf-8")
        for phrase in (
            "credential denied or broker unavailable",
            "stop without secret materialization",
            "validation failed",
            "fail closed without publication",
            "validation passed",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, uml)

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
            "docs/OPERABILITY.md": ("SLI", "SLO", "quarantine", "break-glass"),
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

    def test_release_contract_never_bypasses_evidence_or_reproducibility(self) -> None:
        """Emergency release handling must preserve exact-head gates and reproducible artifacts."""
        release = (ROOT / "docs/RELEASE_AND_ROLLBACK.md").read_text(encoding="utf-8")
        for phrase in (
            "Emergency releases do not bypass required gates",
            "current-head checks",
            "complete coverage",
            "branch protection",
            "reproducible artifact",
            "nondeterministic signing",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, release)
        self.assertNotIn("residual unrun evidence", release)

    def test_traceability_labels_conversation_derived_future_work(self) -> None:
        """Conversation decisions must preserve canonical maturity instead of becoming shipped claims."""
        traceability = (ROOT / "docs/traceability/README.md").read_text(encoding="utf-8")
        for phrase in (
            "IMPLEMENTED_ON_PROTECTED_MAIN",
            "IMPLEMENTED_ON_ACTIVE_PR",
            "PARTIAL",
            "ACCEPTED_ARCHITECTURE",
            "PLANNED",
            "RESEARCH_ONLY",
            "SUPERSEDED",
            "OUT_OF_SCOPE",
            "conversation-derived",
            "docs/doctoring.md",
            "Active-PR behavior is never protected-main truth",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, traceability)


if __name__ == "__main__":
    unittest.main()
