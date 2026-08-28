"""Regression contracts for volatile active-PR evidence in canonical documentation."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
FITNESS = DOCS / "DOCUMENTATION_FITNESS.md"
MATURITY = DOCS / "evidence" / "2026-08-10-active-pr-maturity.md"
BASELINE = DOCS / "product-technical-gap-baseline.md"
CHANGELOG = ROOT / "CHANGELOG.md"


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
        cls.baseline = BASELINE.read_text(encoding="utf-8")
        cls.changelog = CHANGELOG.read_text(encoding="utf-8")

    def test_latest_live_pr_snapshot_is_recorded_in_the_product_baseline(self) -> None:
        """The baseline must preserve exact heads for the newest active product slices."""
        for marker in (
            "Current exact-head active PR evidence",
            "| #73 | Draft | `da99395b09b419845b4a1222a0725482e9231466` | `7861d88d21ed0f0adaeb467957e809826f835071` |",
            "| #208 | Draft | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `c3b6e1a475dce333f6115e5113cae9c07974835f` |",
            "| #209 | Draft | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `69bc738bd45a1b61a4673b122dc3eec8814baa22` |",
            "| #210 | Draft | `69bc738bd45a1b61a4673b122dc3eec8814baa22` | `999979a511c3a890ba93a1a09da8810858877940` |",
            "| #211 | Draft | `c3b6e1a475dce333f6115e5113cae9c07974835f` | `f6e3a3adcfb9cc7a60ef1d79e2aeee27ba54c084` |",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.baseline)

    def test_baseline_refresh_changelog_matches_the_live_snapshot(self) -> None:
        """The changelog must classify and state the same baseline refresh."""
        refresh = "Refreshed the product and technical gap baseline with the current open-PR inventory"
        added = self.changelog.split("### Added", 1)[1].split("### Changed", 1)[0]
        changed = self.changelog.split("### Changed", 1)[1].split("### Security", 1)[0]
        self.assertIn(refresh, added)
        self.assertNotIn(refresh, changed)
        self.assertIn("150 open pull requests, 110 drafts", self.changelog)
        self.assertNotIn("150 open pull requests, 112 drafts", self.changelog)

    def test_dependency_stacks_are_explicit_and_non_shipped(self) -> None:
        """Current browser, network, sensitive and compatibility stacks stay active-only."""
        for pr_number in (52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66):
            with self.subTest(pr_number=pr_number):
                row = active_pr_row(self.maturity, pr_number)
                self.assertIn("**IMPLEMENTED_ON_ACTIVE_PR**", row)
                self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", row)

        for stack in (
            "#47 → #50 → #54",
            "#45 → #46 → #53 → #55",
            "#40→#52→#57→#58",
            "#43→#56→#59→#60→#61",
            "#51→#66",
        ):
            with self.subTest(stack=stack):
                self.assertIn(stack, self.fitness)

        self.assertIn("#49", self.fitness)

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

    def test_typed_semantic_query_evidence_stays_descriptive_and_bounded(self) -> None:
        """PR #57 cannot turn semantic matching into selector or execution authority."""
        row = active_pr_row(self.maturity, 57)
        for marker in (
            "SemanticNodeQuery",
            "role",
            "accessible-name",
            "typed-action",
            "no CSS/XPath/raw DOM selector language",
            "browser I/O or action authority",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, row)

        self.assertIn("Draft stacked on exact #52 head", row)
        self.assertIn("CI run `31429995885`", row)
        self.assertIn("CodeRabbit exact-head status succeed", row)
        self.assertIn("remains Draft because #52/#40 are active prerequisites", row)

    def test_sensitive_audience_evidence_does_not_claim_authentication(self) -> None:
        """An internal audience field is not authenticated workload/service identity."""
        row = active_pr_row(self.maturity, 55)
        self.assertIn("authenticated workload/service identity", row)
        self.assertIn(
            "audience string accepted by the value/policy primitive is **not authentication**",
            self.fitness,
        )
        self.assertIn("new deployment topology or physical ERD entity", self.fitness)

    def test_mv3_mutation_and_isolation_are_compatibility_not_agent_authority(self) -> None:
        """Real MV3 evidence must remain separate from OriginWeave capability grants."""
        bookmark_row = active_pr_row(self.maturity, 56)
        for marker in ("create", "get", "remove", "compatibility evidence only"):
            with self.subTest(marker=marker):
                self.assertIn(marker, bookmark_row)

        for pr_number in (59, 60, 61):
            with self.subTest(pr_number=pr_number):
                row = active_pr_row(self.maturity, pr_number)
                self.assertIn("**IMPLEMENTED_ON_ACTIVE_PR**", row)
                self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", row)

        self.assertIn("Manifest V3 compatibility", self.fitness)
        self.assertIn(
            "Chromium permission or browser compatibility success is not an OriginWeave Agent capability",
            self.fitness,
        )
        self.assertIn(
            "#43/#49/#56/#59/#60/#61 are active compatibility evidence only",
            self.fitness,
        )
        self.assertIn("Update migration is intentionally distinct from restart persistence", self.fitness)
        self.assertIn("isolated-world behavior is intentionally distinct from injection alone", self.fitness)

    def test_extension_proposal_grant_does_not_become_agent_policy_authority(self) -> None:
        """PR #62 must remain a policy-isolation regression, not a new action grant."""
        row = active_pr_row(self.maturity, 62)
        for marker in (
            "ProposeTypedAction",
            "out-of-grant target origin",
            "missing core `Navigate` capability",
            "untrusted instruction source",
            "adds no production API or real Chromium adapter",
            "does not convert extension proposal authority into Agent action/origin authority",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, row)
        self.assertIn("CI run `31436844685`", row)
        self.assertIn("Security Scan run `31436844615`", row)
        self.assertIn("SAST Semgrep run `31436844646`", row)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", row)

    def test_latest_agent_task_and_secret_composition_evidence_remains_partial(self) -> None:
        """Newest active slices must not be promoted into a complete browser or broker runtime."""
        secret_approval = active_pr_row(self.maturity, 63)
        for marker in (
            "ProposeTypedAction",
            "RequireApproval(RiskClass::R3)",
            "no secret broker",
        ):
            with self.subTest(pr_number=63, marker=marker):
                self.assertIn(marker, secret_approval)

        action_outcome = active_pr_row(self.maturity, 64)
        for marker in (
            "PostConditionPredatesDispatch",
            "monotonic",
            "not a browser dispatcher",
        ):
            with self.subTest(pr_number=64, marker=marker):
                self.assertIn(marker, action_outcome)

        controlled_fixture = active_pr_row(self.maturity, 65)
        for marker in (
            "controlled",
            "prompt-injection",
            "not a browser adapter",
        ):
            with self.subTest(pr_number=65, marker=marker):
                self.assertIn(marker, controlled_fixture)

        process_set = active_pr_row(self.maturity, 66)
        for marker in (
            "process-set RSS",
            "duplicate",
            "does not discover Chromium PIDs",
        ):
            with self.subTest(pr_number=66, marker=marker):
                self.assertIn(marker, process_set)

        for marker in (
            "#62/#63",
            "#64",
            "#65",
            "#51→#66",
            "real Chromium",
        ):
            with self.subTest(fitness_marker=marker):
                self.assertIn(marker, self.fitness)

    def test_erd_stays_conceptual_without_persistence_owner(self) -> None:
        """Active in-memory/value primitives must not manufacture a physical data model."""
        self.assertIn("Conceptual ERD/domain model", self.fitness)
        self.assertIn("add no OriginWeave-owned durable store", self.fitness)
        self.assertIn("false architecture", self.fitness)


if __name__ == "__main__":
    unittest.main()
