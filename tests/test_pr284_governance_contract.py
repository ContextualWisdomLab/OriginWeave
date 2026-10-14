"""Regression contracts for the PR #284/#286 CI-governance repairs."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class WorkflowGovernanceRepairTests(unittest.TestCase):
    """Keep workflow optimization, lifecycle admission, and docs in one contract."""

    def test_concurrency_and_draft_lifecycle_contracts_remain_composed(self) -> None:
        """Main must retain both exact concurrency scoping and Draft-run admission rules."""

        ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        mv3 = (ROOT / ".github/workflows/mv3-compatibility.yml").read_text(encoding="utf-8")
        concurrency = (
            "concurrency:\n"
            "  group: ${{ github.workflow }}-${{ github.repository }}-"
            "${{ github.event.pull_request.number || github.run_id }}\n"
            "  cancel-in-progress: ${{ github.event_name == 'pull_request' }}"
        )
        lifecycle = (
            "types: [opened, synchronize, reopened, ready_for_review, "
            "converted_to_draft, closed]"
        )

        self.assertIn(concurrency, ci)
        self.assertIn(concurrency, mv3)
        self.assertIn(lifecycle, ci)
        self.assertIn(lifecycle, mv3)
        self.assertEqual(ci.count("github.event.pull_request.draft == false"), 2)
        self.assertIn("github.event.pull_request.draft == false", mv3)

    def test_quality_gate_and_changelog_describe_the_optimized_ci_boundary(self) -> None:
        """Repository docs must retain the post-optimization compile/check semantics."""

        quality_gates = (ROOT / "docs/quality-gates.md").read_text(encoding="utf-8")
        changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        self.assertIn("Clippy is the workspace compile/check gate", quality_gates)
        self.assertIn("Scoped pull-request workflow cancellation", changelog)


if __name__ == "__main__":
    unittest.main()
