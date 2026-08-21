"""Contract for post-shutdown process termination in the Agent Task forced-close lane."""

from __future__ import annotations

import pathlib
import runpy
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskForcedCloseProcessTerminationContractTests(unittest.TestCase):
    """Require interruption evidence to include bounded Chromium teardown proof."""

    def test_forced_close_browser_pass_binds_and_waits_for_process_identities(self) -> None:
        """The forced-close pass must prove its sampled browser process set terminates."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_forced_close_browser_pass(")
        end = runner.index("\ndef _run_agent_task_forced_close_trial(", start)
        browser_pass = runner[start:end]
        for expected in (
            'capabilities.get("goog:processID")',
            "_read_linux_proc_stat_process_identity",
            "_snapshot_linux_process_evidence",
            "_read_linux_process_identity_set",
            "_terminate_owned_process_bounded",
            "_wait_for_linux_process_identity_exit",
            "_wait_for_linux_process_identity_set_exit",
            '"driver_process_terminated"',
            '"browser_process_terminated"',
            '"chromium_process_set_terminated"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, browser_pass)

    def test_forced_close_trial_preserves_false_teardown_evidence(self) -> None:
        """A failed teardown proof must not be omitted or normalized into success."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_process_termination_trial"
        )
        trial = namespace["_run_agent_task_forced_close_trial"]

        def fake_browser_pass(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            _profile_dir: str,
        ) -> dict[str, object]:
            return {
                "browser_version": namespace["PINNED_CHROME_VERSION"],
                "forced_close_detected": True,
                "session_survived": True,
                "driver_process_terminated": True,
                "browser_process_terminated": False,
                "chromium_process_set_terminated": False,
            }

        trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fake_browser_pass
        result = trial(
            pathlib.Path("/unused/chrome"),
            pathlib.Path("/unused/chromedriver"),
            "http://127.0.0.1/fixture",
            1,
        )
        self.assertIs(result["driver_process_terminated"], True)
        self.assertIs(result["browser_process_terminated"], False)
        self.assertIs(result["chromium_process_set_terminated"], False)

    def test_forced_close_browser_failure_is_returned_after_teardown_waits(self) -> None:
        """A reviewed browser failure after identity capture must not skip teardown proof."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_forced_close_browser_pass(")
        end = runner.index("\ndef _run_agent_task_forced_close_trial(", start)
        browser_pass = runner[start:end]

        for expected in (
            "browser_failure_type",
            "driver_cleanup_failure_type",
            "except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:",
            'browser_failure_type = type(exc).__name__',
            "failure_evidence",
            '"driver_process_terminated": driver_process_terminated',
            '"browser_process_terminated": browser_process_terminated',
            'failure_evidence["chromium_process_set_terminated"]',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, browser_pass)

        shutdown = browser_pass.index("_terminate_owned_process_bounded(driver)")
        root_wait = browser_pass.index("_wait_for_linux_process_identity_exit(")
        set_wait = browser_pass.index("_wait_for_linux_process_identity_set_exit(")
        failure_return = browser_pass.index(
            "if browser_failure_type is not None or driver_cleanup_failure_type is not None:"
        )
        self.assertLess(shutdown, root_wait)
        self.assertLess(root_wait, failure_return)
        self.assertLess(set_wait, failure_return)

    def test_forced_close_driver_shutdown_timeout_is_bounded_and_typed(self) -> None:
        """A wedged ChromeDriver after SIGKILL must become failure evidence, not escape."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_driver_shutdown_timeout_contract"
        )
        shutdown = namespace["_terminate_owned_process_bounded"]
        timeout_seconds = namespace["PROCESS_EXIT_TIMEOUT_SECONDS"]

        class WedgedProcess:
            def __init__(self) -> None:
                self.terminated = False
                self.killed = False
                self.wait_timeouts: list[float] = []

            def terminate(self) -> None:
                self.terminated = True

            def kill(self) -> None:
                self.killed = True

            def wait(self, timeout: float) -> int:
                self.wait_timeouts.append(timeout)
                raise subprocess.TimeoutExpired("chromedriver", timeout)

        process = WedgedProcess()
        terminated, failure_type = shutdown(process)

        self.assertIs(process.terminated, True)
        self.assertIs(process.killed, True)
        self.assertEqual(process.wait_timeouts, [timeout_seconds, timeout_seconds])
        self.assertIs(terminated, False)
        self.assertEqual(failure_type, "TimeoutExpired")

    def test_forced_close_trial_preserves_cleanup_failure_without_overwriting_browser_failure(self) -> None:
        """A teardown timeout must remain separate from the original browser failure type."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_cleanup_failure_trial"
        )
        trial = namespace["_run_agent_task_forced_close_trial"]

        def fake_browser_pass(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            _profile_dir: str,
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "cleanup_failure_type": "TimeoutExpired",
                "driver_process_terminated": False,
                "browser_process_terminated": True,
                "chromium_process_set_terminated": True,
            }

        trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fake_browser_pass
        result = trial(
            pathlib.Path("/unused/chrome"),
            pathlib.Path("/unused/chromedriver"),
            "http://127.0.0.1/fixture",
            3,
        )
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertEqual(result["cleanup_failure_type"], "TimeoutExpired")
        self.assertIs(result["driver_process_terminated"], False)
        self.assertIs(result["browser_process_terminated"], True)
        self.assertIs(result["chromium_process_set_terminated"], True)
        self.assertIs(result["profile_cleaned"], True)

    def test_forced_close_trial_preserves_failure_process_set_teardown_evidence(self) -> None:
        """False root/set teardown evidence must survive the trial failure envelope."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_failure_process_set_trial"
        )
        trial = namespace["_run_agent_task_forced_close_trial"]

        def fake_browser_pass(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            _profile_dir: str,
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "driver_process_terminated": True,
                "browser_process_terminated": False,
                "chromium_process_set_terminated": False,
            }

        trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fake_browser_pass
        result = trial(
            pathlib.Path("/unused/chrome"),
            pathlib.Path("/unused/chromedriver"),
            "http://127.0.0.1/fixture",
            1,
        )
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["driver_process_terminated"], True)
        self.assertIs(result["browser_process_terminated"], False)
        self.assertIs(result["chromium_process_set_terminated"], False)
        self.assertIs(result["profile_cleaned"], True)

    def test_forced_close_trial_does_not_invent_process_set_teardown_evidence(self) -> None:
        """A failure before process-set capture may retain root proof but not invent set proof."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_failure_root_only_trial"
        )
        trial = namespace["_run_agent_task_forced_close_trial"]

        def fake_browser_pass(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            _profile_dir: str,
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "driver_process_terminated": True,
                "browser_process_terminated": True,
            }

        trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fake_browser_pass
        result = trial(
            pathlib.Path("/unused/chrome"),
            pathlib.Path("/unused/chromedriver"),
            "http://127.0.0.1/fixture",
            2,
        )
        self.assertIs(result["passed"], False)
        self.assertIs(result["driver_process_terminated"], True)
        self.assertIs(result["browser_process_terminated"], True)
        self.assertNotIn("chromium_process_set_terminated", result)
        self.assertIs(result["profile_cleaned"], True)

    def test_main_forced_close_gate_requires_process_termination(self) -> None:
        """Compatibility success must reject a live forced-close process identity."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("forced_close_surfaces_complete = all(")
        end = runner.index("\n\n        evidence = {", start)
        gate = runner[start:end]
        for expected in (
            'trial.get("driver_process_terminated") is True',
            'trial.get("browser_process_terminated") is True',
            'trial.get("chromium_process_set_terminated") is True',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, gate)


if __name__ == "__main__":
    unittest.main()
