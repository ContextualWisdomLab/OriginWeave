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
            "149 open pull requests",
            "38 non-draft",
            "111 draft",
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
            "79 draft PRs",
        ):
            with self.subTest(stale_phrase=stale_phrase):
                self.assertNotIn(stale_phrase, text)

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
            "workflow_runs: [$workflow_runs[]?.workflow_runs[]?],",
            "reviewThreads(first: 100, after: $endCursor)",
            "rules/branches/main?per_page=100",
            '"$EVIDENCE_DIR/main-branch-rule-pages.json"',
            '.state == "APPROVED"',
            ".submitted_at != null",
            ".commit_id == $head",
            '.type == "workflows"',
            ".parameters.workflows",
            "required_status_checks",
            '"$EVIDENCE_DIR/pr-${PR}-merge-verdict.json"',
            "for ATTEMPT in 1 2 3; do",
            "RECHECKED_HEAD_SHA=",
            '[[ "$RECHECKED_HEAD_SHA" == "$HEAD_SHA" ]]',
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, shell)

        self.assertNotIn("while :; do", shell)
        self.assertNotIn("/tmp/originweave-open-pr", shell)


if __name__ == "__main__":
    unittest.main()
