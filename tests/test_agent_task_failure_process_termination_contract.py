"""Contract for browser-process termination evidence after Agent Task failure."""

from __future__ import annotations

import pathlib
import runpy
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskFailureProcessTerminationContractTests(unittest.TestCase):
    """Require failed browser work to retain exact root-process teardown evidence."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

    def test_failure_cleanup_release_record_preserves_evidence_limits(self) -> None:
        """The changed failure path needs its own release record and evidence limits."""

        changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        doctoring = (ROOT / "docs" / "doctoring.md").read_text(encoding="utf-8")
        self.assertIn(
            "Failed controlled Agent Task runs now report whether their original browser process ended",
            changelog,
        )
        self.assertIn(
            "a failed task never becomes a pass merely because cleanup succeeded", changelog
        )
        self.assertIn("Process-observation errors leave termination unproven", doctoring)
        self.assertIn(
            "the original browser failure type is not retained in that fallback record",
            doctoring,
        )

    def test_real_failure_path_distinguishes_observed_exit_from_observation_error(self) -> None:
        """Exercise both owning helpers without launching a browser or trusting fake success."""

        for exit_observation in (True, False, PermissionError("controlled read failure")):
            with self.subTest(exit_observation=type(exit_observation).__name__):
                namespace = self._namespace("agent_task_failure_observation_boundary")
                browser_pass = namespace["_run_agent_task_browser_pass"]
                run_trial = namespace["_run_agent_task_trial"]
                driver = mock.Mock()

                def request(_port, method, target, *_args):
                    if method == "POST" and target == "/session":
                        return {
                            "value": {
                                "sessionId": "controlled-session",
                                "capabilities": {
                                    "browserVersion": namespace["PINNED_CHROME_VERSION"],
                                    "goog:processID": 321,
                                },
                            }
                        }
                    if method == "POST":
                        raise RuntimeError("private controlled browser failure")
                    return {}

                exit_wait = mock.Mock(return_value=exit_observation)
                if isinstance(exit_observation, Exception):
                    exit_wait.side_effect = exit_observation
                replacements = {
                    "_free_loopback_port": lambda: 12345,
                    "_wait_for_driver": lambda _port: None,
                    "_json_request": request,
                    "_read_linux_proc_stat_process_identity": lambda _pid: (321, 654),
                    "_wait_for_linux_process_identity_exit": exit_wait,
                }
                with mock.patch.dict(browser_pass.__globals__, replacements), mock.patch.object(
                    namespace["subprocess"], "Popen", return_value=driver
                ):
                    result = run_trial(
                        pathlib.Path("controlled-chrome"),
                        pathlib.Path("controlled-driver"),
                        "http://127.0.0.1/controlled-fixture",
                        21,
                    )

                driver.terminate.assert_called_once_with()
                driver.wait.assert_called_once_with(timeout=5)
                exit_wait.assert_called_once_with(321, 654)
                self.assertIs(result["passed"], False)
                self.assertIs(result["profile_cleaned"], True)
                self.assertNotIn("private controlled browser failure", repr(result))
                if isinstance(exit_observation, Exception):
                    self.assertEqual(result["failure_type"], "PermissionError")
                    self.assertNotIn("browser_process_terminated", result)
                else:
                    self.assertEqual(result["failure_type"], "RuntimeError")
                    self.assertIs(result["browser_process_terminated"], exit_observation)

    def test_browser_pass_retains_failure_process_termination_evidence(self) -> None:
        """A browser-pass failure after identity capture must survive teardown as evidence."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_browser_pass(")
        end = runner.index("\ndef _run_agent_task_trial(", start)
        browser_pass = runner[start:end]
        for expected in (
            "browser_failure_type: str | None = None",
            "browser_failure_type = type(exc).__name__",
            '"failure_type": browser_failure_type',
            '"browser_process_terminated": browser_process_terminated',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, browser_pass)

    def test_trial_preserves_failure_process_termination_evidence(self) -> None:
        """The isolated trial must propagate failure teardown evidence after profile cleanup."""

        namespace = self._namespace("agent_task_failure_process_termination_trial")
        run_trial = namespace["_run_agent_task_trial"]

        def fail_after_shutdown(*_args: object, **_kwargs: object) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "browser_process_terminated": True,
            }

        run_trial.__globals__["_run_agent_task_browser_pass"] = fail_after_shutdown
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            11,
        )

        self.assertEqual(result["trial_number"], 11)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["browser_process_terminated"], True)
        self.assertIs(result["profile_cleaned"], True)

    def test_failed_trial_can_report_a_surviving_original_browser_process(self) -> None:
        """Failure evidence must preserve a false result instead of inventing cleanup."""

        namespace = self._namespace("agent_task_failure_process_survival_trial")
        run_trial = namespace["_run_agent_task_trial"]

        def fail_with_surviving_process(
            *_args: object, **_kwargs: object
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "browser_process_terminated": False,
            }

        run_trial.__globals__["_run_agent_task_browser_pass"] = fail_with_surviving_process
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            12,
        )

        self.assertIs(result["passed"], False)
        self.assertIs(result["browser_process_terminated"], False)
        self.assertIs(result["profile_cleaned"], True)


if __name__ == "__main__":
    unittest.main()
