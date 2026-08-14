"""Contract for exact browser-root termination evidence before crash credit."""

from __future__ import annotations

import contextlib
import pathlib
import runpy
import signal
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskBrowserCrashExactExitDetectionContractTests(unittest.TestCase):
    """Prevent transport failures or unreaped zombies from faking crash evidence."""

    def test_crash_detection_uses_exact_pidfd_termination_observation(self) -> None:
        """Signal and observe termination through one exact kernel process handle."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_browser_crash_browser_pass(")
        end = runner.index("\ndef _run_agent_task_browser_crash_trial(", start)
        browser_pass = runner[start:end]

        termination_call = browser_pass.index(
            "_signal_and_wait_for_linux_process_identity_termination("
        )
        crash_credit = browser_pass.index(
            "browser_process_crash_detected = True", termination_call
        )

        self.assertLess(termination_call, crash_credit)
        self.assertNotIn(
            "_wait_for_linux_process_identity_exit(",
            browser_pass[termination_call:crash_credit],
        )
        self.assertNotIn(
            "except (OSError, RuntimeError, json.JSONDecodeError):",
            browser_pass[termination_call:crash_credit],
        )

    def test_pidfd_termination_observes_killed_unreaped_child(self) -> None:
        """Kernel termination evidence must not require the parent to reap the child."""

        namespace = runpy.run_path(
            str(RUNNER),
            run_name="agent_task_browser_crash_exact_termination_contract",
        )
        signal_and_wait = namespace[
            "_signal_and_wait_for_linux_process_identity_termination"
        ]
        read_identity = namespace["_read_linux_proc_stat_process_identity"]

        child = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            identity = read_identity(child.pid)
            self.assertIsNotNone(identity)
            assert identity is not None

            self.assertTrue(
                signal_and_wait(identity, signal.SIGKILL, timeout_seconds=1.0)
            )
            self.assertEqual(
                read_identity(child.pid),
                identity,
                "the killed child should still be observable as unreaped procfs identity",
            )
        finally:
            with contextlib.suppress(ProcessLookupError):
                child.kill()
            child.wait(timeout=5)

    def test_pidfd_termination_refuses_stale_identity_without_signalling(self) -> None:
        """A stale PID/start-time identity must never signal the current process owner."""

        namespace = runpy.run_path(
            str(RUNNER),
            run_name="agent_task_browser_crash_stale_identity_contract",
        )
        signal_and_wait = namespace[
            "_signal_and_wait_for_linux_process_identity_termination"
        ]
        read_identity = namespace["_read_linux_proc_stat_process_identity"]

        child = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            identity = read_identity(child.pid)
            self.assertIsNotNone(identity)
            assert identity is not None
            stale_identity = (identity[0], identity[1] + 1)

            self.assertFalse(
                signal_and_wait(stale_identity, signal.SIGKILL, timeout_seconds=0.1)
            )
            self.assertIsNone(
                child.poll(),
                "stale identity validation must happen before any signal is delivered",
            )
        finally:
            with contextlib.suppress(ProcessLookupError):
                child.kill()
            child.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
