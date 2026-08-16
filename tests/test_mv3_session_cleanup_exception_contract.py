"""Regression contract for fail-closed WebDriver session cleanup on update trials."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _UnexpectedCleanupFailure(Exception):
    """Model an unreviewed programming/integration failure during session deletion."""


class _FakeDriver:
    """Record process cleanup without launching ChromeDriver."""

    def __init__(self) -> None:
        self.terminated = False
        self.killed = False

    def terminate(self) -> None:
        """Record the graceful process-termination fallback."""

        self.terminated = True

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

    def test_unreviewed_session_cleanup_exception_is_not_silently_suppressed(self) -> None:
        """A new exception class must propagate while ChromeDriver is still terminated."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_cleanup_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__
        fake_driver = _FakeDriver()

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
                raise _UnexpectedCleanupFailure("must not be normalized")
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        surfaces = {
            "workerStartCount": "1",
            "storagePersistence": "initialized",
            "extensionVersion": namespace["INITIAL_EXTENSION_VERSION"],
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

        with tempfile.TemporaryDirectory(prefix="originweave-cleanup-contract-") as trial_root:
            trial_dir = pathlib.Path(trial_root)
            profile_dir = trial_dir / "profile"
            extension_dir = trial_dir / "extension"
            extension_dir.mkdir()
            with (
                mock.patch.object(globals_["subprocess"], "Popen", return_value=fake_driver),
                mock.patch.dict(
                    globals_,
                    {
                        "_free_loopback_port": lambda: 43123,
                        "_wait_for_driver": lambda _port: None,
                        "_json_request": fake_json_request,
                        "_wait_for_extension_evidence": (
                            lambda _port, _session, _persistence, _version, _migration: surfaces
                        ),
                        "_exercise_real_click": lambda _port, _session: "clicked",
                    },
                ),
            ):
                with self.assertRaises(_UnexpectedCleanupFailure):
                    run_browser_pass(
                        pathlib.Path("/controlled/chrome"),
                        pathlib.Path("/controlled/chromedriver"),
                        "http://127.0.0.1:8080/page.html",
                        profile_dir,
                        extension_dir,
                        "initialized",
                        namespace["INITIAL_EXTENSION_VERSION"],
                        "initialized",
                    )

        self.assertTrue(fake_driver.terminated)
        self.assertFalse(fake_driver.killed)


if __name__ == "__main__":
    unittest.main()
