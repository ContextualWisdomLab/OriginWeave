"""Contract for post-shutdown process termination in the Agent Task forced-close lane."""

from __future__ import annotations

import pathlib
import runpy
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
            "_wait_for_linux_process_teardown",
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
            "except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:",
            'browser_failure_type = type(exc).__name__',
            "failure_evidence",
            '"browser_process_terminated": browser_process_terminated',
            'failure_evidence["chromium_process_set_terminated"]',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, browser_pass)

        shutdown = browser_pass.index("_terminate_chromedriver_process(driver)")
        teardown_wait = browser_pass.index("_wait_for_linux_process_teardown(")
        failure_return = browser_pass.index("if browser_failure_type is not None:")
        self.assertLess(shutdown, teardown_wait)
        self.assertLess(teardown_wait, failure_return)

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
        self.assertIs(result["browser_process_terminated"], True)
        self.assertNotIn("chromium_process_set_terminated", result)
        self.assertIs(result["profile_cleaned"], True)

    def test_main_forced_close_gate_requires_process_termination(self) -> None:
        """Compatibility success must reject a live forced-close browser identity."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("forced_close_surfaces_complete = all(")
        end = runner.index("\n\n        evidence = {", start)
        gate = runner[start:end]
        for expected in (
            'trial.get("browser_process_terminated") is True',
            'trial.get("chromium_process_set_terminated") is True',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, gate)


if __name__ == "__main__":
    unittest.main()
