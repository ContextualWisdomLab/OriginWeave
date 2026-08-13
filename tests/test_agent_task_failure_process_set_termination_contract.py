"""Contract for Chromium process-set termination evidence after Agent Task failure."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskFailureProcessSetTerminationContractTests(unittest.TestCase):
    """Retain sampled descendant teardown evidence when controlled browser work fails."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

    def test_browser_pass_retains_sampled_process_set_teardown_after_failure(self) -> None:
        """Late failure must not discard identities already captured before shutdown."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_browser_pass(")
        end = runner.index("\ndef _run_agent_task_trial(", start)
        browser_pass = runner[start:end]
        for expected in (
            "if chromium_process_identities is not None:",
            "chromium_process_set_terminated = _wait_for_linux_process_identity_set_exit(",
            'failure_evidence["chromium_process_set_terminated"]',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, browser_pass)

    def test_trial_preserves_failed_process_set_teardown_evidence(self) -> None:
        """Profile cleanup must preserve both root and sampled-set termination outcomes."""

        namespace = self._namespace("agent_task_failure_process_set_termination_trial")
        run_trial = namespace["_run_agent_task_trial"]

        def fail_after_sampled_set_shutdown(
            *_args: object, **_kwargs: object
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "browser_process_terminated": True,
                "chromium_process_set_terminated": False,
            }

        run_trial.__globals__["_run_agent_task_browser_pass"] = (
            fail_after_sampled_set_shutdown
        )
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            13,
        )

        self.assertEqual(result["trial_number"], 13)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["browser_process_terminated"], True)
        self.assertIs(result["chromium_process_set_terminated"], False)
        self.assertIs(result["profile_cleaned"], True)

    def test_failure_before_process_set_capture_does_not_invent_set_evidence(self) -> None:
        """A failure without sampled identities must remain explicit rather than fabricated."""

        namespace = self._namespace("agent_task_failure_before_process_set_capture")
        run_trial = namespace["_run_agent_task_trial"]

        def fail_before_process_set_capture(
            *_args: object, **_kwargs: object
        ) -> dict[str, object]:
            return {
                "failure_type": "RuntimeError",
                "browser_process_terminated": True,
            }

        run_trial.__globals__["_run_agent_task_browser_pass"] = fail_before_process_set_capture
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            14,
        )

        self.assertIs(result["passed"], False)
        self.assertNotIn("chromium_process_set_terminated", result)
        self.assertIs(result["profile_cleaned"], True)


if __name__ == "__main__":
    unittest.main()