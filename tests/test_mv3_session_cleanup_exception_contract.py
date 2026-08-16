"""Regression contract for fail-closed WebDriver session cleanup."""

from __future__ import annotations

import pathlib
import runpy
import signal
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _UnexpectedCleanupFailure(Exception):
    """Model an unreviewed programming/integration failure during session deletion."""


class _FakeDriver:
    """Model an isolated ChromeDriver process-group leader without launching it."""

    def __init__(self) -> None:
        self.pid = 4242
        self.wait_timeouts: list[float] = []

    def wait(self, timeout: float) -> int:
        """Model an immediately reaped process while recording the bounded timeout."""

        if timeout <= 0:
            raise AssertionError("timeout must remain positive")
        self.wait_timeouts.append(timeout)
        return 0


class ManifestV3SessionCleanupExceptionTests(unittest.TestCase):
    """Unexpected cleanup failures must remain visible after process-group teardown."""

    def test_unreviewed_session_cleanup_exception_is_not_silently_suppressed(self) -> None:
        """A new exception class must propagate after bounded process-group cleanup."""

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

        with tempfile.TemporaryDirectory(prefix="originweave-cleanup-contract-") as profile_dir:
            with (
                unittest.mock.patch.object(
                    globals_["subprocess"], "Popen", return_value=fake_driver
                ) as popen,
                unittest.mock.patch.object(
                    globals_["os"], "killpg"
                ) as kill_process_group,
                unittest.mock.patch.dict(
                    globals_,
                    {
                        "_free_loopback_port": lambda: 43123,
                        "_wait_for_driver": lambda _port: None,
                        "_json_request": fake_json_request,
                        "_wait_for_extension_evidence": (
                            lambda _port, _session, _expected: surfaces
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
                        "initialized",
                    )

        _, popen_kwargs = popen.call_args
        self.assertIs(popen_kwargs.get("start_new_session"), True)
        kill_process_group.assert_called_once_with(fake_driver.pid, signal.SIGTERM)
        self.assertEqual(fake_driver.wait_timeouts, [5])


if __name__ == "__main__":
    unittest.main()
