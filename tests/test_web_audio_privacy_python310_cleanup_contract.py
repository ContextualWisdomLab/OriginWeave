"""Regression contracts for bounded Web Audio runner recovery."""

from __future__ import annotations

import http.client
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_web_audio_privacy.py"


class WebAudioPrivacyRecoveryContractTests(unittest.TestCase):
    """Preserve causal failures and retry only bounded local readiness faults."""

    def _namespace(self) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name="web_audio_recovery_contract")

    def test_cleanup_failure_does_not_require_exception_add_note(self) -> None:
        """Cleanup recovery must remain correct on Python 3.10."""

        namespace = self._namespace()
        cleanup = namespace["_cleanup_browser_session_preserving_primary"]
        globals_dict = cleanup.__globals__
        original_cleanup = globals_dict["_cleanup_browser_session"]

        class Python310StylePrimaryError(RuntimeError):
            """Model a BaseException implementation without Python 3.11 add_note."""

            add_note = None

        def fail_cleanup(_driver_port: int, _session_id: str) -> None:
            raise OSError("simulated bounded cleanup failure")

        globals_dict["_cleanup_browser_session"] = fail_cleanup
        try:
            primary = Python310StylePrimaryError("primary browser failure")
            with self.assertRaises(Python310StylePrimaryError) as raised:
                cleanup(9515, "safe-session", primary)
            self.assertIs(raised.exception, primary)
            self.assertIsInstance(raised.exception.__context__, OSError)
        finally:
            globals_dict["_cleanup_browser_session"] = original_cleanup

    def test_driver_readiness_retries_transient_http_disconnect(self) -> None:
        """A recoverable loopback protocol disconnect must use the bounded startup retry."""

        namespace = self._namespace()
        wait_for_driver = namespace["_wait_for_driver"]
        globals_dict = wait_for_driver.__globals__
        original_request = globals_dict["_json_request"]
        attempts = 0

        def transient_request(
            _driver_port: int,
            _method: str,
            _path: str,
            _payload: dict[str, object] | None = None,
            *,
            timeout: float,
        ) -> dict[str, object]:
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise http.client.RemoteDisconnected("simulated startup disconnect")
            return {"value": {"ready": True}}

        globals_dict["_json_request"] = transient_request
        try:
            wait_for_driver(9515)
            self.assertEqual(attempts, 2)
        finally:
            globals_dict["_json_request"] = original_request

    def test_driver_readiness_retries_transient_bad_status_line(self) -> None:
        """A malformed startup response must use the same bounded readiness retry."""

        namespace = self._namespace()
        wait_for_driver = namespace["_wait_for_driver"]
        globals_dict = wait_for_driver.__globals__
        original_request = globals_dict["_json_request"]
        attempts = 0

        def transient_request(
            _driver_port: int,
            _method: str,
            _path: str,
            _payload: dict[str, object] | None = None,
            *,
            timeout: float,
        ) -> dict[str, object]:
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise http.client.BadStatusLine("simulated startup protocol noise")
            return {"value": {"ready": True}}

        globals_dict["_json_request"] = transient_request
        try:
            wait_for_driver(9515)
            self.assertEqual(attempts, 2)
        finally:
            globals_dict["_json_request"] = original_request

    def test_trial_protocol_failure_becomes_bounded_failed_evidence(self) -> None:
        """A malformed in-trial response must not abort the repeatability portfolio."""

        namespace = self._namespace()
        run_trial_with_evidence = namespace["_run_trial_with_evidence"]
        globals_dict = run_trial_with_evidence.__globals__
        original_run_trial = globals_dict["_run_trial"]

        def fail_trial(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            _trial_number: int,
        ) -> dict[str, object]:
            raise http.client.BadStatusLine("simulated in-trial protocol noise")

        globals_dict["_run_trial"] = fail_trial
        try:
            evidence = run_trial_with_evidence(
                pathlib.Path("/reviewed/chrome"),
                pathlib.Path("/reviewed/chromedriver"),
                "http://127.0.0.1:9515/page.html",
                2,
            )
            self.assertEqual(
                evidence,
                {
                    "trial_number": 2,
                    "passed": False,
                    "error_type": "BadStatusLine",
                },
            )
        finally:
            globals_dict["_run_trial"] = original_run_trial


if __name__ == "__main__":
    unittest.main()
