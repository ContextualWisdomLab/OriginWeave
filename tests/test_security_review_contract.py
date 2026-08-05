"""Regression contracts for security-review findings in pull request one."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"


class SecurityReviewContractTests(unittest.TestCase):
    """Keep the hourly agent credential and egress boundaries fail closed."""

    def test_hourly_runner_allows_rust_registry_endpoints(self) -> None:
        """Future locked Rust dependencies must be installable under blocked egress."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        for endpoint in ["crates.io:443", "index.crates.io:443", "static.crates.io:443"]:
            self.assertIn(endpoint, workflow)

    def test_publication_uses_one_dedicated_non_review_token(self) -> None:
        """The agent PR token must not share the review or merge credential classes."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        dedicated_secret = "secrets." + "OPENCODE_PR_TOKEN"
        self.assertIn(dedicated_secret, workflow)
        self.assertNotIn("PR_REVIEW_MERGE_TOKEN", workflow)
        self.assertNotIn("OPENCODE_APPROVE_TOKEN", workflow)
        publication = workflow.index("Recheck repository state and publish the verified PR")
        self.assertNotIn(dedicated_secret, workflow[:publication])
        self.assertNotIn("gh pr merge", workflow)
        self.assertNotIn("gh pr review", workflow)

    def test_publication_rechecks_live_default_branch_and_release_blockers(self) -> None:
        """A stale checkout cannot publish after main or release policy changed."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        publication = workflow[workflow.index("Recheck repository state and publish the verified PR") :]
        self.assertIn("git/ref/heads/${DEFAULT_BRANCH}", publication)
        self.assertIn("release-blocker", publication)
        self.assertIn("EXPECTED_BASE_SHA", publication)
        self.assertIn("publication cancelled", publication)


if __name__ == "__main__":
    unittest.main()
