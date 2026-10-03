"""Keep repository-local jobs on the explicit isolated self-hosted routing contract.

Source assertions do not prove that the operator has provisioned an eligible
runner or clean per-job isolation. Successful exact-head Actions jobs remain
separate acceptance evidence.
"""

from __future__ import annotations

import hashlib
import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
SELECTOR = "[self-hosted, Linux, X64, cwlab-ci-isolated]"


class SelfHostedRunnerContractTests(unittest.TestCase):
    """Reject hosted fallback and credential persistence in local job routing."""

    def assert_isolated_jobs(self, filename: str, count: int) -> str:
        """Require every declared job to have the exact isolated label set."""
        workflow = (WORKFLOWS / filename).read_text(encoding="utf-8")
        jobs = re.findall(r"^  [A-Za-z0-9_-]+:\s*$", workflow.split("\njobs:\n", 1)[1], re.MULTILINE)
        selectors = re.findall(r"^    runs-on: (.+)$", workflow, re.MULTILINE)
        self.assertEqual(len(jobs), count, filename)
        self.assertEqual(selectors, [SELECTOR] * count, filename)
        return workflow

    def test_native_ci_uses_only_isolated_self_hosted_jobs(self) -> None:
        """Rust contracts and strict production coverage share no hosted fallback."""
        self.assert_isolated_jobs("ci.yml", 2)

    def test_mv3_uses_only_isolated_self_hosted_jobs(self) -> None:
        """Browser fixtures require the same explicitly isolated Linux capacity."""
        self.assert_isolated_jobs("mv3-compatibility.yml", 1)

    def test_deferred_scheduled_workflow_is_unchanged(self) -> None:
        """Keep the reviewed scheduled authority finding outside this routing slice."""
        workflow = (WORKFLOWS / "hourly-product-development.yml").read_bytes()
        self.assertEqual(
            hashlib.sha256(workflow).hexdigest(),
            "86519d5f1214b75d3ec704d02c27fde31aed4a1b8135ac9b3b31e777579ae15f",
        )

    def test_all_local_checkout_steps_disable_persisted_credentials(self) -> None:
        """Explicit checkout authentication must not remain in Git configuration."""
        for path in sorted(WORKFLOWS.glob("*.yml")):
            text = path.read_text(encoding="utf-8")
            steps = re.split(r"^      - ", text, flags=re.MULTILINE)[1:]
            blocks = [step for step in steps if "uses: actions/checkout@" in step]
            with self.subTest(workflow=path.name):
                self.assertGreater(len(blocks), 0)
                for block in blocks:
                    self.assertIn("persist-credentials: false", block)


if __name__ == "__main__":
    unittest.main()
