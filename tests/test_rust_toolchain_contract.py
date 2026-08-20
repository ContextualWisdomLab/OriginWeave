"""Regression contracts for the reproducible Rust compiler baseline."""

from __future__ import annotations

import tomllib
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
RUST_TOOLCHAIN = REPOSITORY_ROOT / "rust-toolchain.toml"
CI_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"
HOURLY_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "hourly-product-development.yml"
REFRESH_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "apply-rust-nightly-refresh.yml"
DEPENDABOT = REPOSITORY_ROOT / ".github" / "dependabot.yml"


class RustToolchainContractTests(unittest.TestCase):
    """Keep stable builds reproducible and branch coverage intentionally fresh."""

    def test_stable_toolchain_is_exact_and_automatically_tracked(self) -> None:
        """The stable compiler changes only through a reviewable manifest update."""

        manifest = tomllib.loads(RUST_TOOLCHAIN.read_text(encoding="utf-8"))
        self.assertEqual(manifest["toolchain"]["channel"], "1.97.1")

        dependabot = DEPENDABOT.read_text(encoding="utf-8")
        self.assertIn('package-ecosystem: "rust-toolchain"', dependabot)
        self.assertIn('directory: "/"', dependabot)
        self.assertIn('interval: "weekly"', dependabot)

    def test_branch_coverage_uses_one_current_date_pinned_nightly(self) -> None:
        """Every branch-coverage command uses the same reviewed nightly snapshot."""

        workflow = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertEqual(workflow.count("nightly-2026-08-18"), 3)
        self.assertNotIn("nightly-2026-08-01", workflow)

        hourly_workflow = HOURLY_WORKFLOW.read_text(encoding="utf-8")
        self.assertEqual(hourly_workflow.count("nightly-2026-08-18"), 2)
        self.assertNotIn("nightly-2026-08-01", hourly_workflow)

    def test_nightly_refresh_accepts_only_old_or_already_refreshed_source(self) -> None:
        """The one-shot materializer remains valid after the source is refreshed."""
        workflow = REFRESH_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("old_count = source.count(old)", workflow)
        self.assertIn("new_count = source.count(new)", workflow)
        self.assertIn("if old_count == 2 and new_count == 0:", workflow)
        self.assertIn("elif old_count == 0 and new_count == 2:", workflow)


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
