"""Regression contracts for volatile active-PR evidence in canonical documentation."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
FITNESS = DOCS / "DOCUMENTATION_FITNESS.md"
MATURITY = DOCS / "evidence" / "2026-08-10-active-pr-maturity.md"


def active_pr_row(text: str, pr_number: int) -> str:
    """Return exactly one maturity row for an active pull request."""
    prefix = f"| #{pr_number} |"
    rows = [line for line in text.splitlines() if line.startswith(prefix)]
    if len(rows) != 1:
        raise AssertionError(
            f"expected exactly one active maturity row for PR #{pr_number}, got {len(rows)}"
        )
    return rows[0]


class ActivePullRequestDocumentationContractTests(unittest.TestCase):
    """Keep volatile implementation evidence separate from protected-main truth."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.fitness = FITNESS.read_text(encoding="utf-8")
        cls.maturity = MATURITY.read_text(encoding="utf-8")

    def test_dependency_stacks_are_explicit_and_non_shipped(self) -> None:
        """Current browser, network, sensitive and compatibility stacks stay active-only."""
        for pr_number in (52, 53, 54, 55, 56):
            with self.subTest(pr_number=pr_number):
                row = active_pr_row(self.maturity, pr_number)
                self.assertIn("**IMPLEMENTED_ON_ACTIVE_PR**", row)
                self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", row)

        for stack in (
            "#47 → #50 → #54",
            "#45 → #46 → #53 → #55",
            "#40→#52",
            "#43→#49/#56",
        ):
            with self.subTest(stack=stack):
                self.assertIn(stack, self.fitness)

    def test_semantic_relationship_evidence_stays_bounded_and_authority_scoped(self) -> None:
        """PR #52 cannot turn relationship metadata into browser or execution authority."""
        row = active_pr_row(self.maturity, 52)
        for marker in ("128", "relationship", "session/context/origin/document"):
            with self.subTest(marker=marker):
                self.assertIn(marker, row)

        for marker in (
            "same browser session, browsing context, canonical origin and document epoch",
            "Self-parent/self-child relationships and duplicate child handles fail closed",
            "relationship graph remains descriptive evidence",
            "not a browser observation adapter",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.fitness)

    def test_sensitive_audience_evidence_does_not_claim_authentication(self) -> None:
        """An internal audience field is not authenticated workload/service identity."""
        row = active_pr_row(self.maturity, 55)
        self.assertIn("authenticated workload/service identity", row)
        self.assertIn(
            "audience string accepted by the value/policy primitive is **not authentication**",
            self.fitness,
        )
        self.assertIn("new deployment topology or physical ERD entity", self.fitness)

    def test_bookmark_mutation_is_compatibility_not_agent_authority(self) -> None:
        """Real MV3 mutation evidence must remain separate from OriginWeave capability grants."""
        row = active_pr_row(self.maturity, 56)
        for marker in ("create", "get", "remove", "compatibility evidence only"):
            with self.subTest(marker=marker):
                self.assertIn(marker, row)

        self.assertIn("Manifest V3 compatibility", self.fitness)
        self.assertIn(
            "Chromium permission or browser compatibility success is not an OriginWeave Agent capability",
            self.fitness,
        )
        self.assertIn("does not justify", self.fitness)

    def test_erd_stays_conceptual_without_persistence_owner(self) -> None:
        """Active in-memory/value primitives must not manufacture a physical data model."""
        self.assertIn("Conceptual ERD/domain model", self.fitness)
        self.assertIn("add no OriginWeave-owned durable store", self.fitness)
        self.assertIn("false architecture", self.fitness)


if __name__ == "__main__":
    unittest.main()
