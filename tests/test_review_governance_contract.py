"""Repository contract for satisfiable pull-request review governance."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
QUALITY_GATES = ROOT / "docs/quality-gates.md"


class ReviewGovernanceContractTests(unittest.TestCase):
    """Prevent automation from recreating an impossible unconditional approval gate."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.quality_gates = QUALITY_GATES.read_text(encoding="utf-8")
        cls.lower = cls.quality_gates.lower()

    def test_solo_maintainer_review_gate_is_explicitly_on_hold(self) -> None:
        """A locally impossible review gate must be represented, not silently bypassed."""

        self.assertIn("on hold", self.lower)
        self.assertIn("solo-maintainer", self.lower)
        self.assertIn("fewer than two eligible", self.lower)
        self.assertIn("current github", self.lower)

    def test_technical_and_security_gates_remain_mandatory(self) -> None:
        """Review-governance realism must never weaken deterministic quality evidence."""

        for requirement in (
            "exact head",
            "production function, line, region, and branch coverage each at 100%",
            "rustdoc",
            "review threads are resolved",
            "repository security checks",
            "branch protection",
        ):
            with self.subTest(requirement=requirement):
                self.assertIn(requirement, self.lower)

    def test_non_author_approval_reenables_when_governance_becomes_satisfiable(self) -> None:
        """The hold must end when two eligible reviewers exist or GitHub requires approval."""

        self.assertIn("at least two eligible", self.lower)
        self.assertIn("github rules", self.lower)
        self.assertIn("re-enabled", self.lower)

    def test_review_evidence_cannot_be_fabricated(self) -> None:
        """Automated verdicts and author-controlled identities cannot impersonate approval."""

        for forbidden_substitute in (
            "self-approval",
            "synthesized",
            "comment",
            "status",
            "automated review",
        ):
            with self.subTest(forbidden_substitute=forbidden_substitute):
                self.assertIn(forbidden_substitute, self.lower)

    def test_old_unconditional_gate_phrase_is_removed(self) -> None:
        """The old conjunction made an unavailable reviewer an unconditional merge gate."""

        self.assertNotIn(
            "required independent approval and repository security checks pass",
            self.lower,
        )


if __name__ == "__main__":
    unittest.main()
