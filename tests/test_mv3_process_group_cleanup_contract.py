"""Regression contracts for bounded process-group cleanup."""

from __future__ import annotations

import os
import pathlib
import runpy
import signal
import subprocess
import sys
import tempfile
import time
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3ProcessGroupCleanupContractTests(unittest.TestCase):
    """Prove cleanup causality and descendant process-group termination."""

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

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "killpg"), "requires POSIX process groups")
    def test_teardown_reaps_descendant_that_ignores_sigterm_after_leader_exits(self) -> None:
        """A fast-exiting leader cannot make a SIGTERM-resistant descendant look reaped."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_real_group_cleanup_contract")
        teardown_driver_process = namespace["_teardown_driver_process"]
        child_program = (
            "import signal,time\n"
            "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
            "print('ready', flush=True)\n"
            "time.sleep(60)\n"
        )
        leader_program = (
            "import subprocess,sys,time\n"
            f"child = subprocess.Popen([sys.executable, '-c', {child_program!r}], "
            "stdout=subprocess.PIPE, text=True)\n"
            "assert child.stdout is not None\n"
            "assert child.stdout.readline().strip() == 'ready'\n"
            "print(child.pid, flush=True)\n"
            "time.sleep(60)\n"
        )
        driver = subprocess.Popen(
            [sys.executable, "-c", leader_program],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            start_new_session=True,
        )
        self.assertIsNotNone(driver.stdout)
        assert driver.stdout is not None
        child_pid = int(driver.stdout.readline().strip())
        self.assertGreater(child_pid, 0)
        process_group_id = driver.pid

        try:
            self.assertIsNone(teardown_driver_process(driver))
            deadline = time.monotonic() + 1.0
            while time.monotonic() < deadline:
                try:
                    os.killpg(process_group_id, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.05)
            else:
                self.fail("process-group teardown left a SIGTERM-resistant descendant alive")
        finally:
            try:
                os.killpg(process_group_id, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                driver.wait(timeout=5)
            except subprocess.TimeoutExpired:
                driver.kill()
                driver.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
