"""Regression contracts for the authoritative OriginWeave documentation graph."""

from pathlib import Path
import re
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DOCS_ROOT = REPOSITORY_ROOT / "docs"
ADR_ROOT = DOCS_ROOT / "adr"


class DocumentationFitnessContractTests(unittest.TestCase):
    """Keep architecture discovery and ADR lifecycle metadata coherent."""

    def test_documentation_index_links_fitness_assessment(self) -> None:
        """The semantic fitness audit must remain discoverable from the docs index."""
        index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("[Documentation fitness assessment](DOCUMENTATION_FITNESS.md)", index)
        self.assertTrue((DOCS_ROOT / "DOCUMENTATION_FITNESS.md").is_file())

    def test_documentation_fitness_distinguishes_design_from_protected_main(self) -> None:
        """A broad design pack must not be mislabeled as code-current closure."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        self.assertIn("DESIGN-SUFFICIENT", assessment)
        self.assertIn("PROTECTED-MAIN-PARTIAL", assessment)
        self.assertIn("File existence alone is never sufficient", assessment)
        self.assertIn("Historical HTTP PR", assessment)
        self.assertIn("MV3 compatibility evidence", assessment)
        self.assertIn("Browser authority", assessment)

    def test_accepted_protected_main_adrs_are_discoverable(self) -> None:
        """Accepted protected-main ADRs must appear in both documentation indexes."""
        docs_index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        adr_index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
        accepted_paths = [
            "0001-chromium-compatibility-kernel.md",
            "0002-agent-safety-kernel.md",
            "0003-provenance-native-observation.md",
            "0004-resolved-destination-policy.md",
            "0005-direct-socket-binding.md",
            "0006-tls-server-identity.md",
            "0007-purpose-bound-sensitive-data-authority.md",
            "0008-leaf-validity-horizon.md",
            "0010-session-context-bound-node-authority.md",
        ]
        for path in accepted_paths:
            with self.subTest(path=path):
                self.assertTrue((ADR_ROOT / path).is_file())
                self.assertIn(path, docs_index)
                self.assertIn(path, adr_index)

    def test_proposed_protected_main_adrs_are_discoverable_without_promotion(self) -> None:
        """Proposed ADR files may be on main but must remain visibly Proposed."""
        docs_index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        adr_index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
        proposed_paths = [
            "0009-hourly-agent-credential-boundary.md",
            "0100-rust-control-plane-boundary.md",
            "0101-isolated-execution-profile-modes.md",
            "0102-typed-actions-and-arbitrary-js.md",
            "0103-semantic-observation-and-stale-node-identity.md",
            "0104-prompt-injection-and-secret-authority.md",
            "0105-resource-governor-priority.md",
            "0106-provenance-evidence-model.md",
            "0107-browser-protocol-adapter-strategy.md",
            "0108-crawler-policy.md",
            "0109-hourly-automation-operational-closure.md",
        ]
        for path in proposed_paths:
            with self.subTest(path=path):
                file_text = (ADR_ROOT / path).read_text(encoding="utf-8")
                self.assertRegex(file_text, r"(?im)^- \*\*Status:\*\* Proposed|^- Status: Proposed")
                self.assertIn(path, docs_index)
                self.assertIn(path, adr_index)

    def test_adr_index_does_not_use_change_local_language_as_timeless_authority(self) -> None:
        """The protected-main ADR index must not describe its ADRs as only `this change`."""
        adr_index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertNotIn("Proposed target-architecture decisions in this change", adr_index)
        self.assertIn("Index completeness rule", adr_index)

    def test_fitness_audit_tracks_current_replacement_and_buyer_gap_lanes(self) -> None:
        """The dated audit must identify the current implementation lanes it evaluated."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        for marker in ("PR #37", "PR #40", "Issue #27", "issue #28"):
            with self.subTest(marker=marker):
                self.assertIn(marker, assessment)

    def test_documentation_index_has_no_duplicate_adr_links(self) -> None:
        """Each ADR target should have one index entry per section, not duplicate drift."""
        docs_index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        targets = re.findall(r"\(adr/(\d{4}[-\w]*\.md)\)", docs_index)
        self.assertEqual(len(targets), len(set(targets)))


if __name__ == "__main__":
    unittest.main()
