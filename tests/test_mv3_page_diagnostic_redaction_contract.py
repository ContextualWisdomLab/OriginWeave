"""Regression tests for credential-safe page-derived browser evidence failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
HOSTILE_PAGE_VALUE = "buyer-secret-marker-must-not-reach-ci"


class Mv3PageDiagnosticRedactionContractTests(unittest.TestCase):
    """Page-controlled observations may decide failure but must not become CI text."""

    def test_real_click_failure_does_not_echo_page_text(self) -> None:
        """A mismatched DOM post-condition must keep page text out of diagnostics."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_click_diagnostic_contract")
        exercise = namespace["_exercise_real_click"]
        element_ids = iter(("button-element", "output-element"))
        request_count = 0

        def find_element(*_args: object, **_kwargs: object) -> str:
            return next(element_ids)

        def json_request(*_args: object, **_kwargs: object) -> dict[str, object]:
            nonlocal request_count
            request_count += 1
            if request_count == 1:
                return {"value": None}
            return {"value": HOSTILE_PAGE_VALUE}

        exercise.__globals__["_find_element"] = find_element
        exercise.__globals__["_json_request"] = json_request

        with self.assertRaises(RuntimeError) as captured:
            exercise(9515, "session-1")

        self.assertEqual(str(captured.exception), "real click post-condition failed")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_mv3_convergence_failure_does_not_echo_page_dataset(self) -> None:
        """Untrusted extension/page dataset values must not be serialized into CI errors."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_surface_diagnostic_contract")
        wait_for_evidence = namespace["_wait_for_extension_evidence"]
        wait_for_evidence.__globals__["FIXTURE_TIMEOUT_SECONDS"] = 0.5
        wait_for_evidence.__globals__["_execute"] = lambda *_args, **_kwargs: {
            "content": HOSTILE_PAGE_VALUE,
            "workerStartCount": "1",
        }

        time_module = wait_for_evidence.__globals__["time"]
        with patch.object(time_module, "monotonic", side_effect=(0.0, 0.0, 1.0)), patch.object(
            time_module,
            "sleep",
            return_value=None,
        ), self.assertRaises(RuntimeError) as captured:
            wait_for_evidence(9515, "session-1", "initialized")

        self.assertEqual(str(captured.exception), "MV3 fixture did not converge")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_agent_task_initial_url_mismatch_does_not_echo_observed_url(self) -> None:
        """A browser-observed URL mismatch must not serialize page-controlled URL data."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_url_diagnostic_contract")
        browser_pass = namespace["_run_agent_task_browser_pass"]
        request_count = 0
        hostile_url = f"https://example.invalid/?value={HOSTILE_PAGE_VALUE}"

        class FakeDriver:
            def terminate(self) -> None:
                return None

            def wait(self, *, timeout: float) -> int:
                del timeout
                return 0

        def json_request(*_args: object, **_kwargs: object) -> dict[str, object]:
            nonlocal request_count
            request_count += 1
            if request_count == 1:
                return {
                    "value": {
                        "sessionId": "session-1",
                        "capabilities": {"browserVersion": namespace["PINNED_CHROME_VERSION"]},
                    }
                }
            if request_count == 2:
                return {"value": None}
            return {"value": hostile_url}

        browser_pass.__globals__["_wait_for_driver"] = lambda *_args, **_kwargs: None
        browser_pass.__globals__["_json_request"] = json_request
        browser_pass.__globals__["_cleanup_browser_session_preserving_primary"] = (
            lambda *_args, **_kwargs: None
        )
        subprocess_module = browser_pass.__globals__["subprocess"]

        with patch.object(subprocess_module, "Popen", return_value=FakeDriver()), self.assertRaises(
            RuntimeError
        ) as captured:
            browser_pass(
                pathlib.Path("/controlled/chrome"),
                pathlib.Path("/controlled/chromedriver"),
                "http://127.0.0.1:8080/index.html",
                "/controlled/profile",
            )

        self.assertEqual(str(captured.exception), "Agent Task initial URL mismatch")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_webdriver_http_failure_does_not_echo_remote_body(self) -> None:
        """A non-success HTTP response may select failure but must not become CI payload."""

        namespace = runpy.run_path(str(RUNNER), run_name="webdriver_http_diagnostic_contract")
        json_request = namespace["_json_request"]

        class FakeResponse:
            status = 500

            def read(self, _limit: int) -> bytes:
                return f'{{"value":{{"message":"{HOSTILE_PAGE_VALUE}"}}}}'.encode()

        class FakeConnection:
            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> FakeResponse:
                return FakeResponse()

            def close(self) -> None:
                return None

        http_client = json_request.__globals__["http"].client
        with patch.object(http_client, "HTTPConnection", return_value=FakeConnection()), self.assertRaises(
            RuntimeError
        ) as captured:
            json_request(9515, "GET", "/status")

        self.assertEqual(str(captured.exception), "WebDriver HTTP request failed with status 500")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_webdriver_protocol_failure_does_not_echo_remote_error_text(self) -> None:
        """A W3C error response must not retain the remote error code or message."""

        namespace = runpy.run_path(str(RUNNER), run_name="webdriver_protocol_diagnostic_contract")
        json_request = namespace["_json_request"]

        class FakeResponse:
            status = 200

            def read(self, _limit: int) -> bytes:
                return (
                    '{"value":{"error":"javascript error","message":"'
                    + HOSTILE_PAGE_VALUE
                    + '"}}'
                ).encode()

        class FakeConnection:
            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> FakeResponse:
                return FakeResponse()

            def close(self) -> None:
                return None

        http_client = json_request.__globals__["http"].client
        with patch.object(http_client, "HTTPConnection", return_value=FakeConnection()), self.assertRaises(
            RuntimeError
        ) as captured:
            json_request(9515, "POST", "/session", {})

        self.assertEqual(str(captured.exception), "WebDriver command failed")
        self.assertNotIn("javascript error", str(captured.exception))
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_driver_readiness_timeout_does_not_echo_last_exception(self) -> None:
        """Startup timeout must not serialize the last remote diagnostic into CI text."""

        namespace = runpy.run_path(str(RUNNER), run_name="webdriver_startup_diagnostic_contract")
        wait_for_driver = namespace["_wait_for_driver"]
        wait_for_driver.__globals__["STARTUP_TIMEOUT_SECONDS"] = 0.5
        wait_for_driver.__globals__["_json_request"] = (
            lambda *_args, **_kwargs: (_ for _ in ()).throw(RuntimeError(HOSTILE_PAGE_VALUE))
        )
        time_module = wait_for_driver.__globals__["time"]

        with patch.object(time_module, "monotonic", side_effect=(0.0, 0.0, 1.0)), patch.object(
            time_module,
            "sleep",
            return_value=None,
        ), self.assertRaises(RuntimeError) as captured:
            wait_for_driver(9515)

        self.assertEqual(str(captured.exception), "ChromeDriver did not become ready")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))


if __name__ == "__main__":
    unittest.main()
