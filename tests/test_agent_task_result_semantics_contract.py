"""Contract for semantic discovery of the controlled Agent Task result."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"


class AgentTaskResultSemanticsContractTests(unittest.TestCase):
    """Require the observed result to be found by browser-computed semantics."""

    def test_result_post_condition_uses_exact_role_and_name(self) -> None:
        """The result must not fall back to a fixture CSS identifier after submit."""

        fixture = FIXTURE.read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")

        self.assertIn('role="status"', fixture)
        self.assertIn('aria-label="Task result"', fixture)
        self.assertIn('"status"', runner)
        self.assertIn('"Task result"', runner)
        self.assertIn('"result_semantics_verified"', runner)
        self.assertNotIn(
            '_find_element(driver_port, session_id, "#task-result")',
            runner,
        )


if __name__ == "__main__":
    unittest.main()
