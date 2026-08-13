"""Contract for exact browser-root exit evidence before crash detection is credited."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskBrowserCrashExactExitDetectionContractTests(unittest.TestCase):
    """Prevent transient WebDriver errors from masquerading as browser-process exit."""

    def test_crash_detection_requires_exact_root_identity_exit(self) -> None:
        """Credit crash detection only after the signalled PID/start-time identity is gone."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_browser_crash_browser_pass(")
        end = runner.index("\ndef _run_agent_task_browser_crash_trial(", start)
        browser_pass = runner[start:end]

        signal_call = browser_pass.index(
            "_signal_linux_process_identity(browser_process_identity, signal.SIGKILL)"
        )
        crash_credit = browser_pass.index(
            "browser_process_crash_detected = True", signal_call
        )
        exact_exit_wait = browser_pass.index(
            "_wait_for_linux_process_identity_exit(", signal_call, crash_credit
        )

        self.assertLess(signal_call, exact_exit_wait)
        self.assertLess(exact_exit_wait, crash_credit)
        self.assertNotIn(
            "except (OSError, RuntimeError, json.JSONDecodeError):",
            browser_pass[signal_call:crash_credit],
        )


if __name__ == "__main__":
    unittest.main()
