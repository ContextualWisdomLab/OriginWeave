"""Contract for truthful WebDriver session cleanup in the forced-close Agent Task lane."""

from __future__ import annotations

import json
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskForcedCloseSessionCleanupContractTests(unittest.TestCase):
    """Require reviewed session-delete failures to remain explicit failure evidence."""

    def test_session_delete_helper_is_bounded_typed_and_source_free(self) -> None:
        """A reviewed WebDriver cleanup failure must return only its stable exception type."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_session_cleanup_contract"
        )
        cleanup = namespace["_delete_webdriver_session_bounded"]
        original_request = cleanup.__globals__["_json_request"]
        calls: list[tuple[int, str, str, dict[str, object]]] = []

        def successful_request(
            driver_port: int,
            method: str,
            path: str,
            payload: dict[str, object],
        ) -> dict[str, object]:
            calls.append((driver_port, method, path, payload))
            return {"value": None}

        cleanup.__globals__["_json_request"] = successful_request
        try:
            self.assertIsNone(cleanup(9515, "session-1"))
        finally:
            cleanup.__globals__["_json_request"] = original_request
        self.assertEqual(calls, [(9515, "DELETE", "/session/session-1", {})])

        for exception in (
            OSError("raw-io-detail"),
            ValueError("raw-value-detail"),
            RuntimeError("raw-runtime-detail"),
            json.JSONDecodeError("raw-json-detail", "x", 0),
        ):
            with self.subTest(exception_type=type(exception).__name__):
                def failing_request(*_args: object, **_kwargs: object) -> dict[str, object]:
                    raise exception

                cleanup.__globals__["_json_request"] = failing_request
                try:
                    failure_type = cleanup(9515, "session-1")
                finally:
                    cleanup.__globals__["_json_request"] = original_request
                self.assertEqual(failure_type, type(exception).__name__)
                self.assertNotIn("raw-", failure_type)

    def test_forced_close_pass_does_not_suppress_session_cleanup_failure(self) -> None:
        """The forced-close failure envelope must consume typed session cleanup evidence."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_forced_close_browser_pass(")
        end = runner.index("\ndef _run_agent_task_forced_close_trial(", start)
        browser_pass = runner[start:end]

        self.assertNotIn("contextlib.suppress(Exception)", browser_pass)
        self.assertIn("session_cleanup_failure_type", browser_pass)
        cleanup_call = browser_pass.index("_delete_webdriver_session_bounded(")
        self.assertIn("driver_port, session_id", browser_pass[cleanup_call:cleanup_call + 160])
        self.assertIn('"session_cleanup_failure_type"', browser_pass)
        self.assertIn('"WebDriverSessionCleanupError"', browser_pass)

    def test_trial_preserves_session_cleanup_failure_separately_from_driver_cleanup(self) -> None:
        """Browser, session-delete, and driver-process failures must remain distinguishable."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_session_cleanup_trial_contract"
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
                "session_cleanup_failure_type": "OSError",
                "cleanup_failure_type": "TimeoutExpired",
                "driver_process_terminated": False,
                "driver_kill_fallback_used": True,
                "browser_process_terminated": True,
                "chromium_process_set_terminated": True,
            }

        trial.__globals__["_run_agent_task_forced_close_browser_pass"] = fake_browser_pass
        result = trial(
            pathlib.Path("/unused/chrome"),
            pathlib.Path("/unused/chromedriver"),
            "http://127.0.0.1/fixture",
            4,
        )

        self.assertIs(result["passed"], False)
        self.assertEqual(result["failure_type"], "RuntimeError")
        self.assertEqual(result["session_cleanup_failure_type"], "OSError")
        self.assertEqual(result["cleanup_failure_type"], "TimeoutExpired")
        self.assertIs(result["driver_process_terminated"], False)
        self.assertIs(result["driver_kill_fallback_used"], True)
        self.assertIs(result["browser_process_terminated"], True)
        self.assertIs(result["chromium_process_set_terminated"], True)
        self.assertIs(result["profile_cleaned"], True)


if __name__ == "__main__":
    unittest.main()
