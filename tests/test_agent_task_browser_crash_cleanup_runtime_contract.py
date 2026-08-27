"""Fail-closed contract for unexpected browser-crash cleanup runtime failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskBrowserCrashCleanupRuntimeContractTests(unittest.TestCase):
    """Keep unknown WebDriver/runtime cleanup failures visible after a browser crash."""

    def test_unknown_runtime_error_is_not_suppressed(self) -> None:
        """Only reviewed post-crash transport loss may be converted to cleanup success."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="agent_task_browser_crash_cleanup_runtime_contract"
        )
        cleanup_session = namespace["_cleanup_crashed_browser_session"]

        def unexpected_runtime_failure(*_args: object, **_kwargs: object) -> object:
            raise RuntimeError("unexpected WebDriver protocol failure")

        cleanup_session.__globals__["_json_request"] = unexpected_runtime_failure
        with self.assertRaisesRegex(RuntimeError, "unexpected WebDriver protocol failure"):
            cleanup_session(9222, "session-1")


if __name__ == "__main__":
    unittest.main()
