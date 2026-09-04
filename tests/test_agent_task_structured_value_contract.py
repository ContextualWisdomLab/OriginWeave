"""Contract for credential-safe structured extraction in the controlled Agent Task."""

from __future__ import annotations

import hashlib
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"


class AgentTaskStructuredValueContractTests(unittest.TestCase):
    """Require bounded semantic extraction without retaining the extracted value."""

    def test_result_is_discovered_by_exact_browser_semantics(self) -> None:
        """The result node must be located by browser-computed role/name, not fixture CSS."""

        runner = RUNNER.read_text(encoding="utf-8")
        fixture = FIXTURE.read_text(encoding="utf-8")
        self.assertIn('aria-label="Task result"', fixture)
        self.assertIn('"status"', runner)
        self.assertIn('"Task result"', runner)
        self.assertIn('"result_semantics_verified"', runner)
        self.assertNotIn('_find_element(driver_port, session_id, "#task-result")', runner)

    def test_structured_value_hash_is_bounded_and_canonical(self) -> None:
        """Only a canonical SHA-256 digest may leave the controlled extraction boundary."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_structured_value_contract")
        self.assertIn("MAX_AGENT_TASK_STRUCTURED_VALUE_BYTES", namespace)
        self.assertIn("_hash_agent_task_structured_value", namespace)
        helper = namespace["_hash_agent_task_structured_value"]
        maximum = namespace["MAX_AGENT_TASK_STRUCTURED_VALUE_BYTES"]

        value = "synthetic structured value"
        expected = "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()
        digest = helper(value)
        self.assertEqual(digest, expected)
        self.assertNotIn(value, digest)
        self.assertEqual(len(digest), len("sha256:") + 64)

        with self.assertRaises(ValueError):
            helper("")
        with self.assertRaises(ValueError):
            helper("x" * (maximum + 1))
        with self.assertRaises(TypeError):
            helper(42)

    def test_agent_task_evidence_reports_field_and_digest_not_raw_result(self) -> None:
        """Trial evidence must expose a field identifier and digest, not extracted text."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            '"structured_value_field"',
            '"structured_value_sha256"',
            '"task_result"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
