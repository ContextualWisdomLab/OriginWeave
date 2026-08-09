"""Keep contributor guidance aligned with the satisfiable review-governance policy."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
AGENTS = ROOT / "AGENTS.md"
QUALITY_GATES = ROOT / "docs" / "quality-gates.md"


class ReviewGovernanceDocumentationConsistencyTests(unittest.TestCase):
    """Prevent the agent contract from recreating an impossible approval gate."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.agents = AGENTS.read_text(encoding="utf-8").lower()
        cls.quality_gates = QUALITY_GATES.read_text(encoding="utf-8").lower()

    def test_agent_contract_references_current_satisfiable_review_governance(self) -> None:
        """Contributor guidance must preserve the same conditional approval semantics."""

        for phrase in (
            "current github rules",
            "on hold",
            "solo-maintainer",
            "fewer than two eligible",
            "re-enabled",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.agents)
                self.assertIn(phrase, self.quality_gates)

    def test_agent_contract_does_not_recreate_unconditional_independent_approval(self) -> None:
        """The old unconditional wording contradicts the merged governance rule."""

        self.assertNotIn(
            "do not bypass required checks, independent approval, or branch protection",
            self.agents,
        )
        self.assertNotIn(
            "a qualifying approval is a formal `approved` review",
            self.agents,
        )

    def test_technical_gates_remain_mandatory(self) -> None:
        """Governance realism must not weaken deterministic product evidence."""

        for phrase in (
            "required checks",
            "branch protection",
            "100%",
            "rustdoc",
            "review",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, self.agents)


if __name__ == "__main__":
    unittest.main()
