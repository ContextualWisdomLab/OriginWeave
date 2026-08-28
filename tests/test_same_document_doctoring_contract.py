"""Canonical standards evidence for same-document mutation authority."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class SameDocumentDoctoringContractTests(unittest.TestCase):
    """Keep design-justifying browser standards in the canonical bibliography."""

    def test_canonical_doctoring_records_same_document_mutation_standards(self) -> None:
        """Epoch rotation must remain traceable to DOM and accessibility standards."""

        doctoring = (ROOT / "docs" / "doctoring.md").read_text(encoding="utf-8")
        self.assertIn("Same-document mutation authority", doctoring)
        self.assertIn("https://dom.spec.whatwg.org/", doctoring)
        self.assertIn("https://www.w3.org/TR/wai-aria-1.2/", doctoring)


if __name__ == "__main__":
    unittest.main()
