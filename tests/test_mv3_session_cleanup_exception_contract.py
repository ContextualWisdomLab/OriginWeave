"""Regression contract for fail-closed WebDriver session cleanup."""

from __future__ import annotations

import pathlib
import runpy
import subprocess
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _UnexpectedCleanupFailure(Exception):
    """Model an unreviewed programming/integration failure during session deletion."""


class _FakeDriver:
    """Record process cleanup without launching ChromeDriver."""

    def __init__(
        self,
        *,
        terminate_error: OSError | None = None,
        kill_error: OSError | None = None,
        wait_timeout_once: bool = False,
    ) -> None:
        self.terminated = False
        self.killed = False
        self.terminate_error = terminate_error
        self.kill_error = kill_error
        self.wait_timeout_once = wait_timeout_once
        self.wait_calls = 0

    def terminate(self) -> None:
        """Record the graceful process-termination fallback."""

        self.terminated = True
        if self.terminate_error is not None:
            raise self.terminate_error

    def kill(self) -> None:
        """Record the bounded hard-kill fallback when requested."""

        self.killed = True
        if self.kill_error is not None:
            raise self.kill_error

    def wait(self, timeout: float) -> int:
        """Model either an immediately reaped process or one bounded timeout."""

        if timeout <= 0:
            raise AssertionError("timeout must remain positive")
        self.wait_calls += 1
        if self.wait_timeout_once and self.wait_calls == 1:
            raise subprocess.TimeoutExpired("controlled-chromedriver", timeout)
        return 0


class ManifestV3SessionCleanupExceptionTests(unittest.TestCase):
    """Unexpected cleanup failures must remain visible after process teardown."""

    @staticmethod
    def _surfaces() -> dict[str, str]:
        """Return one fully passing controlled compatibility surface set."""

        return {
            "workerStartCount": "1",
            "storagePersistence": "initialized",
            "extensionVersion": "1.0.0",
            "storageMigration": "initialized",
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
            with unittest.mock.patch.dict(
                globals_,
                {
                    "_start_chromedriver": lambda _binary: (fake_driver, 43123),
                    "_wait_for_driver": lambda _port: None,
                    "_json_request": fake_json_request,
                    "_wait_for_extension_evidence": (
                        lambda _port, _session, _persistence, _version, _migration: (
                            self._surfaces()
                        )
                    ),
                    "_exercise_real_click": lambda _port, _session: "clicked",
                },
            ):
                try:
                    run_browser_pass(
                        pathlib.Path("/controlled/chrome"),
                        pathlib.Path("/controlled/chromedriver"),
                        "http://127.0.0.1:8080/page.html",
                        pathlib.Path(profile_dir),
                        pathlib.Path(profile_dir) / "extension",
                        "initialized",
                        namespace["INITIAL_EXTENSION_VERSION"],
                        "initialized",
                    )
                except Exception as error:  # noqa: BLE001 - return exact boundary error.
                    return namespace, error
        raise AssertionError("cleanup failure unexpectedly became success")

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

    def test_successful_kill_after_wait_timeout_is_normal_cleanup(self) -> None:
        """A bounded wait timeout must remain a successful fallback when kill reaps the process."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_teardown_contract")
        fake_driver = _FakeDriver(wait_timeout_once=True)

        error = namespace["_teardown_driver_process"](fake_driver)

        self.assertIsNone(error)
        self.assertTrue(fake_driver.terminated)
        self.assertTrue(fake_driver.killed)
        self.assertEqual(fake_driver.wait_calls, 2)

    def test_successful_kill_after_terminate_error_is_normal_cleanup(self) -> None:
        """A recoverable terminate error must not fail cleanup after bounded kill succeeds."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_teardown_contract")
        fake_driver = _FakeDriver(terminate_error=OSError("terminate failed"))

        error = namespace["_teardown_driver_process"](fake_driver)

        self.assertIsNone(error)
        self.assertTrue(fake_driver.terminated)
        self.assertTrue(fake_driver.killed)
        self.assertEqual(fake_driver.wait_calls, 1)

    def test_failed_kill_fallback_is_recorded_on_the_primary_teardown_error(self) -> None:
        """A secondary fallback failure must not disappear while the first error stays causal."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_teardown_contract")
        terminate_error = OSError("terminate failed")
        fake_driver = _FakeDriver(
            terminate_error=terminate_error,
            kill_error=PermissionError("kill denied"),
        )

        error = namespace["_teardown_driver_process"](fake_driver)

        self.assertIs(error, terminate_error)
        self.assertTrue(fake_driver.killed)
        self.assertIn(
            "bounded ChromeDriver kill fallback also failed: PermissionError",
            getattr(error, "__notes__", []),
        )


if __name__ == "__main__":
    unittest.main()
