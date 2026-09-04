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


def bounded_section(text: str, start: str, end: str) -> str:
    """Return one explicitly bounded documentation section, failing closed on drift."""
    if start not in text:
        raise AssertionError(f"missing section start marker: {start}")
    remainder = text.split(start, 1)[1]
    if end not in remainder:
        raise AssertionError(f"missing section end marker after {start}: {end}")
    return remainder.split(end, 1)[0]


class ActivePullRequestDocumentationContractTests(unittest.TestCase):
    """Keep volatile implementation evidence separate from protected-main truth."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.fitness = FITNESS.read_text(encoding="utf-8")
        cls.maturity = MATURITY.read_text(encoding="utf-8")
        cls.baseline = BASELINE.read_text(encoding="utf-8")
        cls.changelog = CHANGELOG.read_text(encoding="utf-8")

    def test_latest_live_pr_snapshot_is_recorded_in_the_product_baseline(self) -> None:
        """The current section must preserve exact heads for the newest active slices."""
        current = bounded_section(
            self.baseline,
            "#### Current exact-head active PR evidence",
            "#### Historical 2026-08-26 maintenance-loop record",
        )
        for marker in (
            "| #73 | Draft | `600d3975c02b68da1974a4c73069b966b39dce7b` | `ce1b138509ab4f52cb0f80290f104358473c6ed3` |",
            "| #72 | Draft | `f86ce504138e79d6e95141a441f60b40920e1fa6` | `600d3975c02b68da1974a4c73069b966b39dce7b` |",
            "| #46 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `373113119446d99f578febd39efc19366e7736b1` |",
            "| #70 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `441a8ce1d09c329c5c1168f4906d9a38fd0abc01` |",
            "| #82 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `f5776f5f233ac0a7c05e3f4a2846436c23438043` |",
            "| #152 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `81407a0e5189a413d1be0963fea90a0c2f254ce1` |",
            "| #210 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `7946dce9a3dd074047d93fca299d48c7aef40e47` |",
            "| #237 | Draft | `542ca1e9c0a863595b8b6697790005d2471f5413` | `2459af602e72fbfe1ce816919473a1075ec0c41f` |",
            "| #239 | Draft | `e45cd6cdcdee73b5c16dc942e6c98cb7e745fae0` | `e840ca299d29a15223c8b9bb1397002c4f41b4a3` |",
            "| #229 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `0145ccba5901e301b41d4be674ca1ed23483ad37` |",
            "| #220 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `b11db2be68f9b6d71aa4c4290b97a8b22097b353` |",
            "| #211 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `52a918577958a5701e1146c7eb8b62fe8f8ccd44` |",
            "| #195 | Draft | `6922dd98779e8f8aad132a3b1f563d7ba6e6d070` | `05e440948840afff1dc6e62cdb6fa52e03ebdaa9` |",
            "| #124 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `fdb88698ca20626a6643bc2ad7944fb968835700` |",
            "| #37 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `5e3dfcbd7a4daea297782cb99635990368589232` |",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)
        self.assertNotIn("| #238 |", current)
        self.assertIn("PR #238 itself", current)

    def test_baseline_refresh_changelog_matches_the_live_snapshot(self) -> None:
        """The current changelog refresh item must carry its own live queue evidence."""
        added = self.changelog.split("### Added", 1)[1].split("### Changed", 1)[0]
        changed = self.changelog.split("### Changed", 1)[1].split("### Security", 1)[0]
        refresh_prefix = "- Revalidated the product-gap queue at"
        refresh_lines = [line for line in added.splitlines() if line.startswith(refresh_prefix)]
        self.assertEqual(1, len(refresh_lines))
        refresh_line = refresh_lines[0]
        self.assertIn("on 2026-09-04", refresh_line)
        self.assertIn("124 open pull requests (9 ready, 115 draft)", refresh_line)
        self.assertIn("13 open non-PR issues", refresh_line)
        self.assertIn("3772d6eddfd556b24397afc80780ef3cc980791e", refresh_line)
        self.assertIn("35d12949bde5e5cbc801fdfb433f4a9914bd4fb0", refresh_line)
        self.assertIn("0a7070b8fdc4a53d4b35f0b93be79a404d8d68c1", refresh_line)
        self.assertIn(
            "Revalidated the active ruleset inventory at 7 required workflows",
            changed,
        )
        self.assertIn("`codeql-pr`", changed)
        self.assertIn("Made the open non-PR issue count reproducible", changed)
        self.assertNotIn("- Corrected the 2026-08-28 product-gap snapshot", added)
        self.assertNotIn("- Revalidated the product-gap queue at", changed)
        self.assertNotIn("115 open pull requests (31 ready, 84 draft)", refresh_line)
        self.assertNotIn("126 open pull requests (54 ready, 72 draft)", refresh_line)
        self.assertNotIn("128 open pull requests (54 ready, 74 draft)", refresh_line)
        self.assertNotIn("153 open pull requests (39 ready, 114 draft)", refresh_line)

    def test_current_warc_provider_failures_are_bound_to_current_head(self) -> None:
        """WARC evidence must describe the current parent head after stack merge."""
        for marker in (
            "PR #210 current exact head is `7946dce9a3dd074047d93fca299d48c7aef40e47`",
            "`Rust contracts` job `98942518975` and `Production coverage` job `98942518680` succeeded",
            "`noema-review` job `98942513421` and `strix` job `98942803402` remain in progress",
            "Its predecessor head `5f59947f5e4b0d3bc0aa5b2d4c6722d3b7c43047`, prior stack merge head `66f360ccac5cec60c72222cc79d58e39f6f00088`, earlier exact head `bea65643109449d63d367a35b8d9bf327ee7cb2c`, and their OpenCode/Strix provider failures remain historical evidence only",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.baseline)

    def test_current_opencode_dispatch_failures_are_bound_to_current_heads(self) -> None:
        """OpenCode dispatch authorization failures must remain explicit blockers."""
        for marker in (
            "A direct central `opencode-review` dispatch run `33192478312` / job `98921183278` was rejected",
            "The immediately preceding PR #238 head `d0b0d1ed92f891f14646fc673b8e1c0d912586fd` remains historical",
            "automatic OpenCode run `33193822920` / job `98926243116` failed closed without a current-head verdict",
            "central dispatch run `33194506918` / job `98928580387` also failed closed at OpenCode",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.baseline)

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
