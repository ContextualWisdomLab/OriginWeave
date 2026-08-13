"""Contract for browser-process termination evidence after Agent Task failure."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskFailureProcessTerminationContractTests(unittest.TestCase):
    """Require failed browser work to retain exact root-process teardown evidence."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

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
