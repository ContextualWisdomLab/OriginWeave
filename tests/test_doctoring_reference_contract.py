"""Regression contracts for standards references that bind OriginWeave design claims."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCTORING = ROOT / "docs" / "doctoring.md"


class DoctoringReferenceContractTests(unittest.TestCase):
    """Keep cited primary-standard authorship aligned with the canonical source."""

    def test_rfc_5280_reference_uses_canonical_author_initials(self) -> None:
        """RFC 5280 must credit Sharon Boeyen as S. Boeyen, matching RFC Editor metadata."""
        text = DOCTORING.read_text(encoding="utf-8")
        expected = (
            "Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W. "
            "(2008). *Internet X.509 public key infrastructure certificate and certificate "
            "revocation list (CRL) profile* (RFC 5280). Internet Engineering Task Force. "
            "https://doi.org/10.17487/RFC5280"
        )
        self.assertIn(expected, text)


if __name__ == "__main__":
    unittest.main()
