"""Regression contracts for OriginWeave's purpose-bound data-governance documentation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class DataGovernanceDocumentationContractTests(unittest.TestCase):
    """Keep privacy and sensitive-data authority explicit and acquisition-reviewable."""

    def test_data_governance_is_part_of_the_authoritative_graph(self) -> None:
        """The canonical index must expose data governance without chat reconstruction."""

        governance_path = ROOT / "docs/DATA_GOVERNANCE.md"
        self.assertTrue(governance_path.is_file())
        index = (ROOT / "docs/README.md").read_text(encoding="utf-8")
        self.assertIn("DATA_GOVERNANCE.md", index)
        self.assertIn("data-governance", index)

    def test_data_governance_rejects_both_blanket_masking_and_ambient_access(self) -> None:
        """Privacy controls must preserve legitimate enterprise workflows without ambient disclosure."""

        governance = (ROOT / "docs/DATA_GOVERNANCE.md").read_text(encoding="utf-8")
        for phrase in (
            "blanket masking",
            "ambient raw-value propagation",
            "purpose-bound",
            "field-scoped",
            "just-in-time disclosure",
            "opaque handle",
            "trusted broker",
            "model/provider/region",
            "break-glass",
            "retention",
            "deletion",
            "data residency",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, governance)

    def test_data_governance_keeps_core_and_host_persistence_authority_separate(self) -> None:
        """A conceptual ERD must not silently invent an OriginWeave production database."""

        governance = (ROOT / "docs/DATA_GOVERNANCE.md").read_text(encoding="utf-8")
        for phrase in (
            "does not claim a production application database",
            "separate accepted ADR",
            "backup/restore",
            "two-or-more-word `snake_case`",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, governance)

    def test_sensitive_evidence_contract_never_uses_protected_values_as_audit_metadata(self) -> None:
        """Evidence may reconstruct authority without becoming another sensitive-data copy."""

        governance = (ROOT / "docs/DATA_GOVERNANCE.md").read_text(encoding="utf-8")
        for phrase in (
            "without copying the protected value",
            "field identifiers and classification",
            "approval or break-glass reference",
            "encryption-key reference/rotation epoch without key material",
            "not by itself a durable append-only audit service",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, governance)

    def test_compliance_language_is_readiness_not_certification(self) -> None:
        """Engineering documentation must not synthesize CSAP or SOC 2 certification claims."""

        governance = (ROOT / "docs/DATA_GOVERNANCE.md").read_text(encoding="utf-8")
        self.assertIn("CSAP/SOC 2 readiness", governance)
        self.assertIn("does not claim certification", governance)
        self.assertIn("independent certification/examination result", governance)


if __name__ == "__main__":
    unittest.main()
