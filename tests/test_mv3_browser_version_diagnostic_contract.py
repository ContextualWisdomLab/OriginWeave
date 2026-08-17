"""Regression contract for classified Chrome capability version diagnostics."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _FakeDriver:
    """Model bounded ChromeDriver process cleanup without launching a process."""

    def __init__(self) -> None:
        self.terminated = False

    def terminate(self) -> None:
        """Record graceful teardown."""

        self.terminated = True

    def kill(self) -> None:
        """Fail if the normal teardown unexpectedly needs hard-kill fallback."""

        raise AssertionError("unexpected ChromeDriver hard-kill fallback")

    def wait(self, timeout: float) -> int:
        """Model an immediately reaped ChromeDriver process."""

        if timeout <= 0:
            raise AssertionError("timeout must remain positive")
        return 0


class ManifestV3BrowserVersionDiagnosticTests(unittest.TestCase):
    """Keep browser-reported capability text out of runner diagnostics."""

    def test_browser_version_mismatch_does_not_retain_raw_capability_text(self) -> None:
        """An unexpected browser version must fail closed with a classified safe message."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_browser_version_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__
        fake_driver = _FakeDriver()
        raw_version = "151.0 secret-token /home/runner/private https://example.invalid"

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
                        "capabilities": {"browserVersion": raw_version},
                    }
                }
            if method == "DELETE" and path.endswith("/session/session-1"):
                return {"value": None}
            raise AssertionError(f"unexpected WebDriver request: {method} {path}")

        with tempfile.TemporaryDirectory(
            prefix="originweave-browser-version-contract-"
        ) as profile_dir:
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
                        pathlib.Path(profile_dir),
                        pathlib.Path(profile_dir) / "extension",
                        "initialized",
                        namespace["INITIAL_EXTENSION_VERSION"],
                        "initialized",
                    )

        rendered = str(raised.exception)
        self.assertEqual(
            rendered,
            f"unexpected Chrome version; expected {namespace['PINNED_CHROME_VERSION']}",
        )
        self.assertNotIn("secret-token", rendered)
        self.assertNotIn("/home/runner/private", rendered)
        self.assertNotIn("example.invalid", rendered)
        self.assertIsNone(raised.exception.__cause__)
        self.assertTrue(fake_driver.terminated)


if __name__ == "__main__":
    unittest.main()
