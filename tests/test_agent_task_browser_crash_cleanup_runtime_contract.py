"""Fail-closed contract for unexpected browser-crash cleanup runtime failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskBrowserCrashCleanupRuntimeContractTests(unittest.TestCase):
    """Keep unknown WebDriver/runtime cleanup failures visible after a browser crash."""

    def test_unknown_runtime_error_is_not_suppressed(self) -> None:
        """Only reviewed post-crash transport loss may be converted to cleanup success."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="agent_task_browser_crash_cleanup_runtime_contract"
        )
        cleanup_session = namespace["_cleanup_crashed_browser_session"]

        def unexpected_runtime_failure(*_args: object, **_kwargs: object) -> object:
            raise RuntimeError("unexpected WebDriver protocol failure")

        cleanup_session.__globals__["_json_request"] = unexpected_runtime_failure
        with self.assertRaisesRegex(RuntimeError, "unexpected WebDriver protocol failure"):
            cleanup_session(9222, "session-1")

    def test_primary_failure_survives_secondary_session_cleanup_failure(self) -> None:
        """Cleanup diagnostics must not replace the first browser failure stage."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="agent_task_browser_crash_primary_failure_contract"
        )
        run_pass = namespace["_run_agent_task_browser_crash_browser_pass"]
        pinned_version = namespace["PINNED_CHROME_VERSION"]
        driver = mock.Mock()
        driver.poll.return_value = 0

        def request(
            _driver_port: int,
            method: str,
            path: str,
            _payload: object,
        ) -> dict[str, object]:
            if method == "POST" and path == "/session":
                return {
                    "value": {
                        "sessionId": "session-1",
                        "capabilities": {
                            "browserVersion": pinned_version,
                            "goog:processID": 777,
                        },
                    }
                }
            if method == "POST" and path.endswith("/url"):
                raise RuntimeError("primary fixture navigation failure")
            if method == "DELETE":
                raise RuntimeError("secondary cleanup failure")
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        with (
            mock.patch.dict(
                run_pass.__globals__,
                {
                    "_free_loopback_port": lambda: 9222,
                    "_wait_for_driver": lambda _port: None,
                    "_json_request": request,
                    "_read_linux_proc_stat_process_identity": lambda _pid: (777, 42),
                },
            ),
            mock.patch.object(
                run_pass.__globals__["subprocess"],
                "Popen",
                return_value=driver,
            ),
        ):
            result = namespace["_run_agent_task_browser_crash_trial"](
                pathlib.Path("/controlled/chrome"),
                pathlib.Path("/controlled/chromedriver"),
                "http://127.0.0.1/agent-task",
                1,
            )

        self.assertFalse(result["passed"])
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertEqual(result["failure_stage"], "fixture_navigation")
        self.assertEqual(result["reason_code"], "runtime_error")
        self.assertEqual(result["session_cleanup_failure_type"], "RuntimeError")
        self.assertNotIn("primary fixture navigation failure", repr(result))
        self.assertNotIn("secondary cleanup failure", repr(result))
        self.assertTrue(result["profile_cleaned"])
        driver.wait.assert_called_once_with(timeout=5)


if __name__ == "__main__":
    unittest.main()
