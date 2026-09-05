"""Regression contract for preserving Chromium sandboxing in real-browser evidence."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class BrowserSandboxContractTests(unittest.TestCase):
    """Require every real-Chromium evidence path to keep process sandboxing enabled."""

    def test_real_browser_passes_do_not_disable_chromium_sandbox(self) -> None:
        """No real-browser evidence lane may launch Chrome with ``--no-sandbox``."""

        namespace = runpy.run_path(str(RUNNER), run_name="browser_sandbox_contract")
        for function_name in (
            "_run_browser_pass",
            "_run_agent_task_browser_pass",
            "_run_agent_task_forced_close_browser_pass",
            "_run_agent_task_browser_crash_browser_pass",
        ):
            with self.subTest(function_name=function_name):
                source = inspect.getsource(namespace[function_name])
                self.assertNotIn('"--no-sandbox"', source)


if __name__ == "__main__":
    unittest.main()
