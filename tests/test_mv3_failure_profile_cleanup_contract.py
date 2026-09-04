"""Contract for MV3 restart-trial profile cleanup evidence on browser failure."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class Mv3FailureProfileCleanupContractTests(unittest.TestCase):
    """Require failed extension-compatibility trials to retain teardown evidence."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

    def test_failed_restart_pass_returns_profile_cleanup_evidence(self) -> None:
        """A failed initial or restarted browser pass must retain profile cleanup proof."""

        namespace = self._namespace("mv3_failure_cleanup_behavior")
        run_trial = namespace["_run_restart_trial"]

        def fail_browser_pass(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise RuntimeError("synthetic MV3 browser failure")

        run_trial.__globals__["_run_browser_pass"] = fail_browser_pass
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            11,
        )

        self.assertEqual(result["trial_number"], 11)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertIs(result["profile_cleaned"], True)
        self.assertNotIn("synthetic MV3 browser failure", repr(result))

    def test_surface_failure_preserves_cleanup_and_surface_evidence(self) -> None:
        """A compatibility regression must not be mislabeled as profile cleanup failure."""

        namespace = self._namespace("mv3_surface_failure_cleanup_behavior")
        run_trial = namespace["_run_restart_trial"]
        browser_pass_number = 0

        def browser_pass(*_args: object, **_kwargs: object) -> dict[str, object]:
            nonlocal browser_pass_number
            browser_pass_number += 1
            persisted = browser_pass_number == 2
            return {
                "browser_version": "controlled-browser",
                "worker_start_count": browser_pass_number,
                "storage_persistence": "persisted" if persisted else "initialized",
                "surfaces": {
                    "service-worker": True,
                    "real-browser-click": not persisted,
                },
            }

        run_trial.__globals__["_run_browser_pass"] = browser_pass
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            12,
        )

        self.assertEqual(result["trial_number"], 12)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "CompatibilitySurfaceFailure")
        self.assertIs(result["profile_cleaned"], True)
        self.assertIs(result["surfaces"]["real-browser-click"], False)

    def test_acceptance_gate_requires_cleanup_evidence_for_every_mv3_trial(self) -> None:
        """Failed MV3 trials must not be filtered out of the profile-cleanup gate."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("mv3_profiles_cleaned = all(", runner)
        self.assertIn('trial.get("profile_cleaned") is True', runner)
        self.assertIn('"profiles_cleaned": mv3_profiles_cleaned', runner)
        self.assertIn("Manifest V3 profile cleanup gate failed", runner)


if __name__ == "__main__":
    unittest.main()
