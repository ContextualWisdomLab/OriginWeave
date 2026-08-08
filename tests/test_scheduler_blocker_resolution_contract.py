"""Regression contracts for evidence-based scheduler blocker remediation."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
AGENT_CONTRACT = ROOT / "AGENTS.md"


class SchedulerBlockerResolutionContractTests(unittest.TestCase):
    """Keep automated maintenance action-oriented without inventing authority."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = AGENT_CONTRACT.read_text(encoding="utf-8")

    def test_blockers_follow_rca_feasibility_action_and_state_verification(self) -> None:
        """A blocker must trigger a realistic corrective-action loop before reporting."""

        required_sequence = (
            "Refetch exact live evidence",
            "Identify the root cause",
            "Enumerate candidate corrective actions",
            "Validate each candidate",
            "actual tool support",
            "actor permissions",
            "Execute the first safe and feasible action",
            "verify the authoritative state transition",
            "Only report an external blocker",
        )
        positions = []
        for phrase in required_sequence:
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.contract)
                positions.append(self.contract.index(phrase))
        self.assertEqual(positions, sorted(positions))

    def test_approval_wait_is_classified_by_real_reviewer_availability(self) -> None:
        """Comments or unavailable identities must never masquerade as approval supply."""

        for phrase in (
            "formal `APPROVED` review",
            "non-author",
            "repository collaborator",
            "reviewer-provisioning gap",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.contract)


if __name__ == "__main__":
    unittest.main()