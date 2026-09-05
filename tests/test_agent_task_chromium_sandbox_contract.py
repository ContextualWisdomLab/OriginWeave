"""Contract for preserving Chromium's process sandbox in Agent Task evidence."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
WORKFLOW = ROOT / ".github" / "workflows" / "mv3-compatibility.yml"


class AgentTaskChromiumSandboxContractTests(unittest.TestCase):
    """Keep the governed-browser Agent Task evidence on a sandboxed Chrome process."""

    def test_agent_task_browser_pass_does_not_disable_chromium_sandbox(self) -> None:
        """Security evidence must not launch the Agent Task browser with ``--no-sandbox``."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_sandbox_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])

        self.assertNotIn('"--no-sandbox"', browser_pass_source)

    def test_pinned_chrome_installs_its_linux_sandbox_helper(self) -> None:
        """The downloaded browser must retain a usable layer-one sandbox in CI."""

        workflow = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("sudo chown root:root", workflow)
        self.assertIn("sudo chmod 4755", workflow)
        self.assertIn("chrome_sandbox", workflow)


if __name__ == "__main__":
    unittest.main()
