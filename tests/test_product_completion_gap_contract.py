"""Regression contract for the dated commercial-completion gap baseline."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"


class ProductCompletionGapContractTests(unittest.TestCase):
    """Keep the exact repository snapshot and completion tracks reviewable."""

    def test_current_snapshot_checks_do_not_route_by_phrase_substrings(self) -> None:
        """Current-snapshot assertions must not depend on count-word substrings."""
        source = pathlib.Path(__file__).read_text(encoding="utf-8")
        brittle_condition = '"pull requests" in phrase or ' + '"draft" in phrase'
        self.assertNotIn(brittle_condition, source)

    def test_baseline_records_current_inventory_and_completion_issues(self) -> None:
        """The dated baseline must not retain superseded queue counts or omit buyer tracks."""
        text = BASELINE.read_text(encoding="utf-8")
        end_marker = "#### 2026-08-29 maintenance-loop record"
        self.assertIn(end_marker, text)
        current = text.split("## Observed snapshot: ", 1)[1].split(end_marker, 1)[0]

        for phrase in (
            "108 open pull requests",
            "24 non-draft",
            "84 draft",
            "2026-08-28 116-PR snapshot",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, current)

        for phrase in (
            "#198",
            "#199",
            "#200",
            "#201",
            "#202",
            "#203",
            "Shrink the open-PR queue",
            "durable WARC/PROV replay",
            "stable BAP/MCP runtime API",
            "signed cross-platform Chromium distribution",
            "enterprise control and experience plane",
            "commercial acceptance gate",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

        self.assertNotIn("Shrink the 109-PR queue", text)
        self.assertNotIn(
            "current-head hosted CI, security, Noema, scheduler, and OpenCode workflows were still regenerating at the recheck",
            text,
        )

        for stale_phrase in (
            "100 open pull requests",
            "22 non-draft",
            "78 draft",
            "148 open pull requests",
            "79 draft PRs",
            "40 non-draft",
            "110 draft",
            "150 open pull requests",
            "prior 150-PR snapshot",
            "128 open pull requests",
            "74 draft",
            "115 open pull requests",
            "31 non-draft",
        ):
            with self.subTest(stale_phrase=stale_phrase):
                self.assertNotIn(stale_phrase, current)

    def test_active_github_approval_rule_is_not_documented_as_bypassable(self) -> None:
        """An active counted-approval rule must stop merge without an eligible approver."""
        text = BASELINE.read_text(encoding="utf-8")

        self.assertIn("eligible non-author", text)
        self.assertIn("reviewer-provisioning gap", text)
        self.assertNotIn("owner-directed administrative merge", text)

    def test_changelog_marks_superseded_warc_head_as_historical(self) -> None:
        """A predecessor WARC head must not look like the current exact evidence."""
        text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")

        self.assertIn("`d83748a70bd1b16dbfec46007fe02989ba6ce188` was superseded", text)
        self.assertIn("0341079331f9cea669eb9a5cc21842fd6027431e", text)

    def test_baseline_records_central_required_workflow_failures(self) -> None:
        """The baseline must preserve exact central workflow provenance and failures."""
        text = BASELINE.read_text(encoding="utf-8")

        for phrase in (
            "central `.github` repository",
            "repository ID `1274066402`",
            "close-empty-pr",
            "opencode-review",
            "pr-review-merge-scheduler",
            "security-scan",
            "strix",
            "sast-semgrep",
            "noema-review",
            "OpenCode current-head verdict",
            "Strix provider/backend",
            "internal server error",
            "33177641855",
            "33177641888",
            "33182772296",
            "MODEL_OUTPUT_UNAVAILABLE",
            "model pool exhausted",
            "33182749298",
            "#1391",
            "e4ba6b599cd1e50d0139762885682607b731655d",
            "did not prove missing workflow identities",
            "does not authorize bypass",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

    def test_same_day_prior_inventory_is_bound_to_the_maintenance_record(self) -> None:
        """A same-day comparison must retain its exact prior observation in the record."""
        text = BASELINE.read_text(encoding="utf-8")
        record_end = "#### Current exact-head active PR evidence"
        self.assertIn(record_end, text)
        record = text.split("#### 2026-08-29 maintenance-loop record", 1)[1].split(
            record_end, 1
        )[0]
        self.assertIn("115 open pull requests (31 ready, 84 draft)", record)

    def test_issue_table_distinguishes_open_issues_from_governance_signals(self) -> None:
        """The table total must distinguish product issues from governance signals."""
        text = BASELINE.read_text(encoding="utf-8")
        table_end = "## Buyer-visible and technical gap matrix"
        self.assertIn(table_end, text)
        table = text.split("### Open issues and governance signals", 1)[1].split(
            table_end, 1
        )[0]
        self.assertIn("11 open issues plus 2 governance signals", table)
        self.assertIn("Issue or signal", table)

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
            "--paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/issues?state=open&per_page=100'",
            '"$EVIDENCE_DIR/open-issue-pages.json"',
            'map(select(has("pull_request") | not))',
            "open_non_pr_issues",
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
