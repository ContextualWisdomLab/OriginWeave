"""Focused regression contracts for reviewed documentation discoverability gaps."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class DocumentationDiscoverabilityFollowupTests(unittest.TestCase):
    """Keep canonical diagrams and maturity vocabulary machine-discoverable."""

    def test_extension_authority_view_is_indexed_mermaid(self) -> None:
        """The extension authority view must exist, be indexed, and remain diagram-as-code."""
        uml_index = (ROOT / "docs" / "uml" / "README.md").read_text(encoding="utf-8")
        authority_view = ROOT / "docs" / "uml" / "extension-authority.md"

        self.assertTrue(authority_view.is_file())
        self.assertIn("](extension-authority.md)", uml_index)
        self.assertIn("```mermaid", authority_view.read_text(encoding="utf-8"))

    def test_traceability_keeps_complete_maturity_vocabulary(self) -> None:
        """Every canonical capability maturity label must remain explicit."""
        traceability = (ROOT / "docs" / "traceability" / "README.md").read_text(
            encoding="utf-8"
        )
        for label in (
            "IMPLEMENTED_ON_PROTECTED_MAIN",
            "IMPLEMENTED_ON_ACTIVE_PR",
            "PARTIAL",
            "ACCEPTED_ARCHITECTURE",
            "PLANNED",
            "RESEARCH_ONLY",
            "SUPERSEDED",
            "OUT_OF_SCOPE",
        ):
            with self.subTest(label=label):
                self.assertIn(label, traceability)


if __name__ == "__main__":
    unittest.main()
