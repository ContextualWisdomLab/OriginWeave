"""Keep repository-local jobs on the explicit isolated self-hosted routing contract.

Source assertions do not prove that the operator has provisioned an eligible
runner or clean per-job isolation. Successful exact-head Actions jobs remain
separate acceptance evidence.
"""

from __future__ import annotations

import hashlib
import pathlib
import re
import tempfile
import unittest
from unittest import mock

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

    def checkout_fixture_result(self, step: str, suffix: str = ".yml") -> unittest.TestResult:
        """Run the real checkout contract against one temporary workflow."""
        with tempfile.TemporaryDirectory() as directory:
            workflows = pathlib.Path(directory)
            (workflows / f"fixture{suffix}").write_text(
                "jobs:\n  check:\n    steps:\n      - " + step, encoding="utf-8"
            )
            result = unittest.TestResult()
            case = SelfHostedRunnerContractTests(
                "test_all_local_checkout_steps_disable_persisted_credentials"
            )
            with mock.patch(__name__ + ".WORKFLOWS", workflows):
                case.run(result)
            return result

    def assert_fixture_rejected(self, result: unittest.TestResult) -> None:
        """Require an assertion rejection, never an unexpected checker error."""
        self.assertEqual(result.errors, [])
        self.assertEqual(len(result.failures), 1)
        self.assertEqual(result.skipped, [])

    def test_negative_fixture_oracle_rejects_checker_errors(self) -> None:
        """A checker exception must fail the negative-fixture test, not prove rejection."""
        case = SelfHostedRunnerContractTests(
            "test_checkout_rejects_misleading_credential_text"
        )
        result = unittest.TestResult()
        with mock.patch.object(
            SelfHostedRunnerContractTests,
            "assert_checkout_credential_not_persisted",
            side_effect=RuntimeError("fixture checker error"),
        ):
            case.run(result)
        self.assertFalse(result.wasSuccessful())
        self.assertEqual(result.errors, [])
        self.assertEqual(len(result.failures), 6)

    def test_checkout_rejects_misleading_credential_text(self) -> None:
        """Comments, sibling values and duplicate mappings cannot disable credentials."""
        unsafe_steps = [
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: true # persist-credentials: false\n",
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: true\n        env:\n"
            "          NOTE: 'persist-credentials: false'\n",
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: true\n          ref: |\n"
            "            persist-credentials: false\n",
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: false\n          persist-credentials: true\n",
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: false\n        with:\n"
            "          persist-credentials: true\n",
            "uses: actions/checkout@fixture\n        env:\n"
            "          persist-credentials: false\n",
        ]
        for step in unsafe_steps:
            with self.subTest(step=step):
                self.assert_fixture_rejected(self.checkout_fixture_result(step))

    def test_checkout_accepts_explicit_mapping_false(self) -> None:
        """Actual false mapping values remain valid with comments and sibling scalars."""
        steps = [
            "uses: actions/checkout@fixture\n        with:\n"
            "          persist-credentials: false\n",
            "uses: actions/checkout@fixture\n        with: # explicit mapping\n"
            "          ref: |\n            persist-credentials: true\n"
            "          persist-credentials: false # intended value\n",
        ]
        for step in steps:
            with self.subTest(step=step):
                result = self.checkout_fixture_result(step)
                self.assertTrue(result.wasSuccessful(), result.failures + result.errors)

    def test_checkout_checks_yaml_extension(self) -> None:
        """A workflow's alternate YAML extension must not avoid the credential check."""
        step = "uses: actions/checkout@fixture\n        with:\n          persist-credentials: true\n"
        self.assert_fixture_rejected(self.checkout_fixture_result(step, ".yaml"))

    def assert_checkout_credential_not_persisted(self, step: str, path: pathlib.Path) -> None:
        """Require the checkout step's effective with mapping to set false."""
        headers = re.findall(r"^        with:[ \t]*(?:#.*)?$", step, re.MULTILINE)
        self.assertEqual(len(headers), 1, path)
        with_match = re.search(
            r"^        with:[ \t]*(?:#.*)?$((?:\n(?: {10,}[^\n]*|[ \t]*$))*)",
            step,
            re.MULTILINE,
        )
        if with_match is None:
            self.fail(f"checkout step has no with mapping: {path}")
        values = re.findall(
            r"^          persist-credentials:([^\n]*)$",
            with_match.group(1),
            re.MULTILINE,
        )
        self.assertEqual([value.split("#", 1)[0].strip() for value in values], ["false"], path)

    def test_all_local_checkout_steps_disable_persisted_credentials(self) -> None:
        """Every YAML workflow checkout disables credentials in its with mapping."""
        paths = sorted({*WORKFLOWS.glob("*.yml"), *WORKFLOWS.glob("*.yaml")})
        self.assertGreater(len(paths), 0)
        for path in paths:
            text = path.read_text(encoding="utf-8")
            steps = re.split(r"^      - ", text, flags=re.MULTILINE)[1:]
            blocks = [
                step for step in steps
                if re.search(r"^(?:        )?uses: actions/checkout@", step, re.MULTILINE)
            ]
            with self.subTest(workflow=path.name):
                self.assertGreater(len(blocks), 0)
                for block in blocks:
                    self.assert_checkout_credential_not_persisted(block, path)


if __name__ == "__main__":
    unittest.main()
