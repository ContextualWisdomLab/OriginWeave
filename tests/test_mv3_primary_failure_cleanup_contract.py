"""Regression contract for preserving a primary browser-pass failure through cleanup."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _FakeDriver:
    """Model a ChromeDriver process that tears down successfully."""

    def __init__(self) -> None:
        self.terminated = False
        self.wait_calls = 0

    def terminate(self) -> None:
        """Record graceful termination."""

        self.terminated = True

    def kill(self) -> None:
        """Fail if the hard-kill fallback is unexpectedly required."""

        raise AssertionError("hard-kill fallback was not expected")

    def wait(self, timeout: float) -> int:
        """Model an immediately reaped process."""

        if timeout <= 0:
            raise AssertionError("timeout must remain positive")
        self.wait_calls += 1
        return 0


class ManifestV3PrimaryFailureCleanupTests(unittest.TestCase):
    """Cleanup failures must not replace the causal browser-pass failure."""

    def test_primary_browser_failure_survives_reviewed_session_cleanup_failure(self) -> None:
        """A later reviewed cleanup error must remain secondary to the primary failure."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_primary_cleanup_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__
        fake_driver = _FakeDriver()
        primary_error = RuntimeError("controlled primary browser-pass failure")
        cleanup_error = OSError("controlled session cleanup failure")

        def fake_json_request(
            _driver_port: int,
            method: str,
            path: str,
            _payload=None,
            *,
            timeout: float = 5.0,
        ):
            if timeout <= 0:
                raise AssertionError("timeout must remain positive")
            if method == "POST" and path == "/session":
                return {
                    "value": {
                        "sessionId": "session-1",
                        "capabilities": {
                            "browserVersion": namespace["PINNED_CHROME_VERSION"]
                        },
                    }
                }
            if method == "POST" and path == "/session/session-1/url":
                raise primary_error
            if method == "DELETE" and path == "/session/session-1":
                raise cleanup_error
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        with tempfile.TemporaryDirectory(prefix="originweave-primary-cleanup-") as profile_dir:
            with (
                unittest.mock.patch.object(
                    globals_["subprocess"], "Popen", return_value=fake_driver
                ),
                unittest.mock.patch.dict(
                    globals_,
                    {
                        "_free_loopback_port": lambda: 43123,
                        "_wait_for_driver": lambda _port: None,
                        "_json_request": fake_json_request,
                    },
                ),
            ):
                with self.assertRaises(RuntimeError) as raised:
                    run_browser_pass(
                        pathlib.Path("/controlled/chrome"),
                        pathlib.Path("/controlled/chromedriver"),
                        "http://127.0.0.1:8080/page.html",
                        profile_dir,
                        pathlib.Path(profile_dir) / "extension",
                        "initialized",
                        namespace["INITIAL_EXTENSION_VERSION"],
                        "initialized",
                    )

        self.assertIs(raised.exception, primary_error)
        self.assertTrue(fake_driver.terminated)
        self.assertEqual(fake_driver.wait_calls, 1)


if __name__ == "__main__":
    unittest.main()
