"""Contract for one total post-shutdown teardown deadline in the ordinary Agent Task lane."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskSharedTeardownDeadlineContractTests(unittest.TestCase):
    """Prevent ordinary Agent Task teardown polling from multiplying the budget."""

    def test_browser_pass_uses_only_the_combined_teardown_waiter(self) -> None:
        """Root and sampled-set evidence must share one timeout authority after shutdown."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_browser_pass(")
        end = runner.index("\ndef _run_agent_task_trial(", start)
        browser_pass = runner[start:end]

        self.assertIn("_wait_for_linux_process_teardown(", browser_pass)
        self.assertNotIn("_wait_for_linux_process_identity_exit(", browser_pass)
        self.assertNotIn("_wait_for_linux_process_identity_set_exit(", browser_pass)

        shutdown = browser_pass.index("driver.wait(timeout=5)")
        teardown_wait = browser_pass.index("_wait_for_linux_process_teardown(")
        failure_return = browser_pass.index("if browser_failure_type is not None:")
        self.assertLess(shutdown, teardown_wait)
        self.assertLess(teardown_wait, failure_return)


if __name__ == "__main__":
    unittest.main()
