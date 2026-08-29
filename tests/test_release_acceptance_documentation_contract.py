"""Regression contracts for benchmark release-acceptance architecture documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ReleaseAcceptanceDocumentationContractTests(unittest.TestCase):
    """Keep active release-evidence semantics discoverable without PR archaeology."""

    def test_architecture_records_active_release_acceptance_boundary(self) -> None:
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        for phrase in (
            "Active PR #240 — benchmark release acceptance",
            "BenchmarkFailureClass",
            "ZeroEventSafetyEvidence",
            "ZeroEventSafetyThreshold",
            "Inconclusive",
            "does not grant release authority",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, architecture)

    def test_release_acceptance_adr_is_proposed_and_indexed(self) -> None:
        adr_path = ROOT / "docs/adr/0015-benchmark-release-acceptance.md"
        self.assertTrue(adr_path.is_file())
        adr = adr_path.read_text(encoding="utf-8")
        for phrase in (
            "- Status: Proposed",
            "## Context",
            "## Decision",
            "## Failure and degraded behavior",
            "## Security / privacy / governance impact",
            "## Tests and acceptance evidence",
            "## Migration and rollback",
            "## Supersession / reversal conditions",
            "BenchmarkFailureClass",
            "DeclaredLimitation",
            "ZeroEventSafetyEvidence",
            "ZeroEventSafetyThreshold",
            "Inconclusive",
            "does not grant release authority",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, adr)

        index = (ROOT / "docs/adr/README.md").read_text(encoding="utf-8")
        self.assertIn("[0015](0015-benchmark-release-acceptance.md)", index)
        self.assertIn("Benchmark release acceptance evidence and safety gates", index)


if __name__ == "__main__":
    unittest.main()
