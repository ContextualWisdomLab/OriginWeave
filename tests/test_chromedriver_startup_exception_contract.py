"""Fail-closed exception contract for bounded ChromeDriver startup probing."""

from __future__ import annotations

import http.client
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ChromeDriverStartupExceptionContractTests(unittest.TestCase):
    """Keep recoverable startup transport faults separate from terminal failures."""

    def test_runner_startup_retries_transient_incomplete_response(self) -> None:
        """A truncated startup response may be retried within the existing deadline."""

        namespace = runpy.run_path(str(RUNNER), run_name="chromedriver_incomplete_startup_response")
        wait_for_driver = namespace["_wait_for_driver"]
        attempts = [0]

        def truncated_then_ready(*_args: object, **_kwargs: object) -> dict[str, object]:
            attempts[0] += 1
            if attempts[0] == 1:
                raise http.client.IncompleteRead(b'{"value":', 20)
            return {"value": {"ready": True}}

        wait_for_driver.__globals__["_json_request"] = truncated_then_ready
        wait_for_driver(9515)
        self.assertEqual(attempts[0], 2)

    def test_runner_startup_does_not_retry_terminal_runtime_failure(self) -> None:
        """A terminal WebDriver/runtime failure must fail closed before a later success."""

        namespace = runpy.run_path(str(RUNNER), run_name="chromedriver_terminal_startup_failure")
        wait_for_driver = namespace["_wait_for_driver"]
        attempts = [0]

        def terminal_then_ready(*_args: object, **_kwargs: object) -> dict[str, object]:
            attempts[0] += 1
            if attempts[0] == 1:
                raise RuntimeError("WebDriver HTTP 403: forbidden")
            return {"value": {"ready": True}}

        wait_for_driver.__globals__["_json_request"] = terminal_then_ready
        with self.assertRaisesRegex(RuntimeError, "HTTP 403"):
            wait_for_driver(9515)
        self.assertEqual(attempts[0], 1)


if __name__ == "__main__":
    unittest.main()
