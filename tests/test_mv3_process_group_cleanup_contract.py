"""Regression contract for session-cleanup causality during process-group teardown."""

from __future__ import annotations

import pathlib
import runpy
import signal
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3ProcessGroupCleanupContractTests(unittest.TestCase):
    """Preserve the reviewed session failure when process-group signaling also fails."""

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

    def test_session_cleanup_error_survives_process_group_signal_failures(self) -> None:
        """Process-group teardown failure stays secondary to reviewed session cleanup."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_group_cleanup_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__
        driver = unittest.mock.Mock()
        driver.pid = 4242
        session_error = RuntimeError("session delete failed")

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
                raise session_error
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        with tempfile.TemporaryDirectory(prefix="originweave-group-cleanup-") as profile_dir:
            with (
                unittest.mock.patch.object(
                    globals_["os"],
                    "killpg",
                    side_effect=PermissionError("process-group signal denied"),
                ) as kill_process_group,
                unittest.mock.patch.dict(
                    globals_,
                    {
                        "_start_chromedriver": lambda _binary: (driver, 43123),
                        "_wait_for_driver": lambda _port: None,
                        "_json_request": fake_json_request,
                        "_wait_for_extension_evidence": (
                            lambda _port, _session, _expected: self._surfaces()
                        ),
                        "_exercise_real_click": lambda _port, _session: "clicked",
                    },
                ),
            ):
                with self.assertRaises(namespace["WebDriverSessionCleanupError"]) as raised:
                    run_browser_pass(
                        pathlib.Path("/controlled/chrome"),
                        pathlib.Path("/controlled/chromedriver"),
                        "http://127.0.0.1:8080/page.html",
                        profile_dir,
                        "initialized",
                    )

        self.assertIs(raised.exception.__cause__, session_error)
        self.assertEqual(
            kill_process_group.call_args_list,
            [
                unittest.mock.call(driver.pid, signal.SIGTERM),
                unittest.mock.call(driver.pid, signal.SIGKILL),
            ],
        )
        self.assertIn(
            "ChromeDriver process teardown also failed: PermissionError",
            getattr(raised.exception, "__notes__", []),
        )


if __name__ == "__main__":
    unittest.main()
