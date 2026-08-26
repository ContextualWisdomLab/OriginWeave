"""Regression contract for the dated commercial-completion gap baseline."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"


class ProductCompletionGapContractTests(unittest.TestCase):
    """Keep the exact repository snapshot and completion tracks reviewable."""

    def test_baseline_records_current_inventory_and_completion_issues(self) -> None:
        """The dated baseline must not retain superseded queue counts or omit buyer tracks."""
        text = BASELINE.read_text(encoding="utf-8")

        for phrase in (
            "130 open pull requests",
            "44 non-draft",
            "86 draft",
            "#198",
            "#199",
            "#200",
            "#201",
            "#202",
            "#203",
            "durable WARC/PROV replay",
            "stable BAP/MCP runtime API",
            "signed cross-platform Chromium distribution",
            "enterprise control and experience plane",
            "commercial acceptance gate",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

        for stale_phrase in (
            "100 open pull requests",
            "22 non-draft",
            "78 draft",
            "148 open pull requests",
            "79 draft PRs",
            "40 non-draft",
            "110 draft",
            "150 open pull requests",
        ):
            with self.subTest(stale_phrase=stale_phrase):
                self.assertNotIn(stale_phrase, text)

    def test_active_github_approval_rule_is_not_documented_as_bypassable(self) -> None:
        """An active counted-approval rule must stop merge without an eligible approver."""
        text = BASELINE.read_text(encoding="utf-8")

        self.assertIn("eligible non-author", text)
        self.assertIn("reviewer-provisioning gap", text)
        self.assertNotIn("owner-directed administrative merge", text)

    def test_evidence_commands_reproduce_inventory_checks_and_review_state(self) -> None:
        """The evidence procedure must paginate the queue and inspect each exact PR head."""
        text = BASELINE.read_text(encoding="utf-8")
        evidence = text.split("## Evidence commands", 1)[1].split("\n## ", 1)[0]
        shell = evidence.split("```bash", 1)[1].split("```", 1)[0]

        for phrase in (
            "--paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/pulls?state=open&per_page=100'",
            "set -euo pipefail",
            'EVIDENCE_DIR="$(mktemp -d /tmp/originweave-evidence.XXXXXX)"',
            '"$EVIDENCE_DIR/open-pr-pages.json"',
            "jq '[.[][]]' \"$EVIDENCE_DIR/open-pr-pages.json\"",
            '"repos/ContextualWisdomLab/OriginWeave/pulls/$PR"',
            '"repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/check-runs?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/statuses?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/pulls/$PR/reviews?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/actions/runs?head_sha=$HEAD_SHA&per_page=100"',
            "check_runs: [$checks[][].check_runs[]?],",
            "legacy_statuses: [$statuses[][][]?]",
            "workflow_runs: [$workflow_runs[][].workflow_runs[]?],",
            "reviewThreads(first: 100, after: $endCursor)",
            "rules/branches/main?per_page=100",
            '"$EVIDENCE_DIR/main-branch-rule-pages.json"',
            '"$EVIDENCE_DIR/collaborator-pages.json"',
            '"$EVIDENCE_DIR/collaborators.json"',
            '"$EVIDENCE_DIR/pr-${PR}-merge-verdict.json.tmp"',
            '.state == "APPROVED"',
            ".submitted_at != null",
            ".commit_id == $head",
            "group_by(.reviewer)",
            "required_approving_review_count",
            "require_last_push_approval",
            "last_push_approval_authority",
            '"github_rule_evaluation_required"',
            "if $pull_request_parameters.require_last_push_approval == true then false",
            "$pr[0].user.login",
            '.type == "workflows"',
            ".parameters.workflows",
            "required_status_checks",
            '"$EVIDENCE_DIR/pr-${PR}-merge-verdict.json"',
            "for ATTEMPT in 1 2 3; do",
            "RECHECKED_HEAD_SHA=",
            "RECHECKED_BASE_SHA=",
            'if [[ "$RECHECKED_HEAD_SHA" == "$HEAD_SHA" && "$RECHECKED_BASE_SHA" == "$BASE_SHA" ]]; then',
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, shell)

        self.assertNotIn("while :; do", shell)
        self.assertNotIn("/tmp/originweave-open-pr", shell)
        self.assertNotIn("check_runs: [$checks[]?.check_runs[]?],", shell)
        self.assertNotIn("legacy_statuses: [$statuses[][]?]", shell)
        self.assertNotIn("workflow_runs: [$workflow_runs[]?.workflow_runs[]?],", shell)
        self.assertNotIn("$reviews[][]?\n          | select(.state", shell)
        self.assertNotIn("head-commit.json", shell)
        self.assertNotIn("$head_commit[0].committer.login", shell)
        self.assertNotIn("$head_commit[0].author.login", shell)


if __name__ == "__main__":
    unittest.main()
