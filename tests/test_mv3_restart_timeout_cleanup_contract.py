"""Regression contract for MV3 restart teardown timeout cleanup evidence."""

from __future__ import annotations

import pathlib
import runpy
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class Mv3RestartTimeoutCleanupContractTests(unittest.TestCase):
    """Require reviewed ChromeDriver teardown timeouts to retain MV3 cleanup proof."""

    def test_teardown_timeout_returns_profile_cleanup_evidence(self) -> None:
        """A bounded process teardown timeout must become failed-trial evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_restart_timeout_cleanup")
        run_trial = namespace["_run_restart_trial"]

        def timeout_browser_pass(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise subprocess.TimeoutExpired(
                cmd="private-controlled-chromedriver-path",
                timeout=5,
            )

        run_trial.__globals__["_run_browser_pass"] = timeout_browser_pass
        result = run_trial(
            pathlib.Path("controlled-chrome"),
            pathlib.Path("controlled-chromedriver"),
            "http://127.0.0.1/controlled-fixture",
            12,
        )

        self.assertEqual(result["trial_number"], 12)
        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "TimeoutExpired")
        self.assertIs(result["profile_cleaned"], True)
        self.assertNotIn("private-controlled-chromedriver-path", repr(result))


if __name__ == "__main__":
    unittest.main()
