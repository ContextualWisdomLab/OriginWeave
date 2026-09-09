"""Contract for preserving Chromium's process sandbox in Agent Task evidence."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskChromiumSandboxContractTests(unittest.TestCase):
    """Keep the governed-browser Agent Task evidence on a sandboxed Chrome process."""

    def test_agent_task_browser_pass_does_not_disable_chromium_sandbox(self) -> None:
        """Security evidence must not launch the Agent Task browser with ``--no-sandbox``."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_sandbox_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])

        self.assertNotIn('"--no-sandbox"', browser_pass_source)

    def test_agent_task_browser_pass_does_not_own_chromedriver_diagnostics(self) -> None:
        """Keep ChromeDriver process diagnostics in their canonical owner lane."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_sandbox_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])

        self.assertNotIn('"--verbose"', browser_pass_source)
        self.assertNotIn('"--log-path=', browser_pass_source)
        self.assertNotIn("_classify_chromedriver_startup_diagnostic", browser_pass_source)

if __name__ == "__main__":
    unittest.main()
