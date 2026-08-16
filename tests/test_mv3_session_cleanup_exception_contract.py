"""Regression contract for fail-closed WebDriver session cleanup."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _UnexpectedCleanupFailure(Exception):
    """Model an unreviewed programming/integration failure during session deletion."""


class _FakeDriver:
    """Record process cleanup without launching ChromeDriver."""

    def __init__(self, *, terminate_error: OSError | None = None) -> None:
        self.terminated = False
        self.killed = False
        self.terminate_error = terminate_error

    def terminate(self) -> None:
        """Record the graceful process-termination fallback."""

        self.terminated = True
        if self.terminate_error is not None:
            raise self.terminate_error

    def kill(self) -> None:
        """Record the bounded hard-kill fallback when requested."""

        self.killed = True

    def wait(self, timeout: float) -> int:
        """Model an immediately reaped process."""

        if timeout <= 0:
            raise AssertionError("timeout must remain positive")
        return 0


class ManifestV3SessionCleanupExceptionTests(unittest.TestCase):
    """Unexpected cleanup failures must remain visible after process teardown."""

    @staticmethod
    def _surfaces() -> dict[str, str]:
        """Return one fully passing controlled compatibility surface set."""

        return {
            "workerStartCount": "1",
            "storagePersistence": "initialized",
            "workerReply": "pong",
            "content": "ready",
            "storage": "ready",
            "dnr": "blocked",
            "tabs": "ready",
            "windows": "ready",
            "scripting": "ready",
            "scriptingExecuted": "ready",
            "commands": "ready",
            "sidePanel": "ready",
            "bookmarks": "ready",
            "history": "ready",
            "downloads": "ready",
        }

    def _run_with_cleanup_failure(
        self,
        cleanup_failure: Exception,
        fake_driver: _FakeDriver,
    ) -> tuple[object, object]:
        """Run the production browser-pass boundary with controlled cleanup failures."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_cleanup_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__

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
            if method == "POST" and path.endswith("/url"):
                return {"value": None}
            if method == "DELETE" and path.endswith("/session/session-1"):
                raise cleanup_failure
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        with tempfile.TemporaryDirectory(prefix="originweave-cleanup-contract-") as profile_dir:
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
                        "_wait_for_extension_evidence": (
                            lambda _port, _session, _expected: self._surfaces()
                        ),
                        "_exercise_real_click": lambda _port, _session: "clicked",
                    },
                ),
            ):
                try:
                    run_browser_pass(
                        pathlib.Path("/controlled/chrome"),
                        pathlib.Path("/controlled/chromedriver"),
                        "http://127.0.0.1:8080/page.html",
                        profile_dir,
                        "initialized",
                    )
                except Exception as error:  # noqa: BLE001 - the test returns the exact boundary error.
                    return namespace, error
        self.fail("cleanup failure unexpectedly became success")

    def test_unreviewed_session_cleanup_exception_is_not_silently_suppressed(self) -> None:
        """A new exception class must propagate while ChromeDriver is still terminated."""

        fake_driver = _FakeDriver()
        expected = _UnexpectedCleanupFailure("must not be normalized")
        _namespace, error = self._run_with_cleanup_failure(expected, fake_driver)

        self.assertIs(error, expected)
        self.assertTrue(fake_driver.terminated)
        self.assertFalse(fake_driver.killed)

    def test_reviewed_session_cleanup_error_survives_teardown_failure(self) -> None:
        """The causal session failure must not be replaced by a later terminate error."""

        fake_driver = _FakeDriver(terminate_error=OSError("terminate failed"))
        session_error = RuntimeError("session delete failed")
        namespace, error = self._run_with_cleanup_failure(session_error, fake_driver)

        self.assertIsInstance(error, namespace["WebDriverSessionCleanupError"])
        self.assertIs(error.__cause__, session_error)
        self.assertTrue(fake_driver.terminated)
        self.assertTrue(fake_driver.killed)


if __name__ == "__main__":
    unittest.main()
