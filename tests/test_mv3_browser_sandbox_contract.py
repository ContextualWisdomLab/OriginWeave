"""Regression contract for preserving Chromium sandboxing in browser evidence."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
DOCTORING = ROOT / "docs" / "doctoring" / "chromium-sandbox-evidence-boundary.md"


class BrowserSandboxContractTests(unittest.TestCase):
    """Require every real-Chromium compatibility path to keep the process sandbox enabled."""

    def test_real_browser_passes_do_not_disable_chromium_sandbox(self) -> None:
        """Neither MV3 nor Agent Task evidence may launch Chrome with ``--no-sandbox``."""

        namespace = runpy.run_path(str(RUNNER), run_name="browser_sandbox_contract")
        for function_name in ("_run_browser_pass", "_run_agent_task_browser_pass"):
            with self.subTest(function_name=function_name):
                source = inspect.getsource(namespace[function_name])
                self.assertNotIn('"--no-sandbox"', source)

    def test_doctoring_describes_the_repaired_sandbox_boundary(self) -> None:
        """Doctoring must not describe the repaired ordinary launch as still unsandboxed."""

        text = DOCTORING.read_text(encoding="utf-8")
        self.assertNotIn(
            "ordinary Manifest V3 compatibility path still passes `--no-sandbox`",
            text,
        )
        self.assertIn(
            "both ordinary Manifest V3 and Agent Task real-browser paths omit `--no-sandbox`",
            text,
        )


if __name__ == "__main__":
    unittest.main()
