"""Behavioral contracts for race-safe bounded ChromeDriver process cleanup."""

from __future__ import annotations

import pathlib
import runpy
import subprocess
import unittest
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


def _runner_symbol(name: str) -> Any:
    """Load one private runner symbol without invoking the compatibility entry point."""

    return runpy.run_path(str(RUNNER))[name]


class _ExitBeforeTerminateDriver:
    """Model a child that exits immediately before the TERM syscall boundary."""

    def __init__(self) -> None:
        self.wait_timeouts: list[float] = []
        self.kill_calls = 0

    def terminate(self) -> None:
        """Report ESRCH because the child has already exited."""

        raise ProcessLookupError("ChromeDriver already exited")

    def wait(self, *, timeout: float) -> int:
        """Reap the already exited child."""

        self.wait_timeouts.append(timeout)
        return 0

    def kill(self) -> None:
        """Record any unnecessary escalation."""

        self.kill_calls += 1


class _ExitBeforeKillDriver:
    """Model a child that exits after TERM wait expiry but before KILL."""

    def __init__(self) -> None:
        self.wait_timeouts: list[float] = []
        self.kill_calls = 0

    def terminate(self) -> None:
        """Accept the initial TERM request."""

    def wait(self, *, timeout: float) -> int:
        """Timeout once, then reap the child after the KILL race."""

        self.wait_timeouts.append(timeout)
        if len(self.wait_timeouts) == 1:
            raise subprocess.TimeoutExpired("chromedriver", timeout)
        return 0

    def kill(self) -> None:
        """Report ESRCH because the child exited before escalation."""

        self.kill_calls += 1
        raise ProcessLookupError("ChromeDriver exited before KILL")


class _PermissionDeniedDriver:
    """Model an unexpected signaling failure that must not be normalized."""

    def terminate(self) -> None:
        """Reject the signal for a reason other than child exit."""

        raise PermissionError("TERM denied")

    def wait(self, *, timeout: float) -> int:
        """Fail if cleanup incorrectly continues after permission denial."""

        raise AssertionError(f"unexpected wait({timeout})")

    def kill(self) -> None:
        """Fail if cleanup incorrectly escalates after permission denial."""

        raise AssertionError("unexpected KILL")


class ChromeDriverExitRaceCleanupTests(unittest.TestCase):
    """Keep normal child-exit races harmless without suppressing real failures."""

    def test_exit_before_terminate_is_reaped_without_kill(self) -> None:
        """An ESRCH from TERM is normal exit evidence, not teardown failure."""

        cleanup = _runner_symbol("_terminate_chromedriver_process")
        driver = _ExitBeforeTerminateDriver()

        cleanup(driver)

        self.assertEqual(driver.kill_calls, 0)
        self.assertEqual(len(driver.wait_timeouts), 1)
        self.assertGreaterEqual(driver.wait_timeouts[0], 0.0)

    def test_exit_before_kill_is_reaped_without_leaking_process_lookup(self) -> None:
        """An ESRCH from KILL must still finish the bounded reap path."""

        cleanup = _runner_symbol("_terminate_chromedriver_process")
        driver = _ExitBeforeKillDriver()

        cleanup(driver)

        self.assertEqual(driver.kill_calls, 1)
        self.assertEqual(len(driver.wait_timeouts), 2)
        self.assertGreaterEqual(driver.wait_timeouts[1], 0.0)
        self.assertLessEqual(driver.wait_timeouts[1], driver.wait_timeouts[0])

    def test_unexpected_signal_error_propagates(self) -> None:
        """Permission failures must remain visible rather than being broadly suppressed."""

        cleanup = _runner_symbol("_terminate_chromedriver_process")

        with self.assertRaisesRegex(PermissionError, "TERM denied"):
            cleanup(_PermissionDeniedDriver())

    def test_all_browser_lanes_use_the_race_safe_cleanup_boundary(self) -> None:
        """Every pinned browser lane must share the same ChromeDriver cleanup contract."""

        runner = RUNNER.read_text(encoding="utf-8")
        boundaries = (
            ("def _run_browser_pass(", "\ndef _run_trial("),
            ("def _run_agent_task_browser_pass(", "\ndef _run_agent_task_trial("),
            (
                "def _run_agent_task_forced_close_browser_pass(",
                "\ndef _run_agent_task_forced_close_trial(",
            ),
        )
        for start_marker, end_marker in boundaries:
            with self.subTest(start_marker=start_marker):
                start = runner.index(start_marker)
                end = runner.index(end_marker, start)
                browser_lane = runner[start:end]
                self.assertIn("_terminate_chromedriver_process(driver)", browser_lane)
                self.assertNotIn("driver.terminate()", browser_lane)
                self.assertNotIn("driver.kill()", browser_lane)


if __name__ == "__main__":
    unittest.main()
