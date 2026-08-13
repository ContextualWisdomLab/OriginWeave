"""Contract for Agent Task profile cleanup evidence on failed browser trials."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskFailureCleanupContractTests(unittest.TestCase):
    """Require failed trials to retain credential-free teardown evidence."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

    def test_failed_browser_pass_returns_profile_cleanup_evidence(self) -> None:
        """A browser-pass failure must not discard proof that its task profile was removed."""

        namespace = self._namespace("agent_task_failure_cleanup_behavior")
        run_trial = namespace["_run_agent_task_trial"]

        def fail_browser_pass(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise RuntimeError("synthetic controlled browser failure")

        run_trial.__globals__["_run_agent_task_browser_pass"] = fail_browser_pass
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            7,
        )

        self.assertEqual(result["trial_number"], 7)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["profile_cleaned"], True)
        self.assertNotIn("synthetic controlled browser failure", repr(result))

    def test_failed_forced_close_pass_returns_profile_cleanup_evidence(self) -> None:
        """A forced-close probe failure must still prove that its task profile was removed."""

        namespace = self._namespace("agent_task_forced_close_cleanup_behavior")
        run_trial = namespace["_run_agent_task_forced_close_trial"]

        def fail_browser_pass(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise RuntimeError("synthetic forced-close browser failure")

        run_trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fail_browser_pass
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            9,
        )

        self.assertEqual(result["trial_number"], 9)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["profile_cleaned"], True)
        self.assertNotIn("synthetic forced-close browser failure", repr(result))

    def test_acceptance_gate_requires_cleanup_evidence_for_every_trial(self) -> None:
        """Failed trials must not be filtered out of either profile-cleanup gate."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("agent_task_profiles_cleaned = all(", runner)
        self.assertIn("forced_close_profiles_cleaned = all(", runner)
        self.assertIn('trial.get("profile_cleaned") is True', runner)
        self.assertIn('"profiles_cleaned": agent_task_profiles_cleaned', runner)
        self.assertIn('"profiles_cleaned": forced_close_profiles_cleaned', runner)
        self.assertIn("Agent Task profile cleanup gate failed", runner)
        self.assertIn("Agent Task forced-close profile cleanup gate failed", runner)


if __name__ == "__main__":
    unittest.main()
