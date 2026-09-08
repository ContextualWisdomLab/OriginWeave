"""Contract for honest extension-isolation evidence in controlled Agent Task trials."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskExtensionIsolationEvidenceContractTests(unittest.TestCase):
    """Separate requested Chrome launch isolation from browser-observed success evidence."""

    def test_launch_request_is_not_reported_as_observed_extension_isolation(self) -> None:
        """A command-line request must not become a verified Agent Task success surface."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_extension_evidence_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        surface_source = inspect.getsource(namespace["_agent_task_surfaces_complete"])

        self.assertIn('"--disable-extensions"', browser_pass_source)
        self.assertIn('"extensions_disabled_requested": True', browser_pass_source)
        self.assertNotIn('"extensions_disabled": True', browser_pass_source)
        self.assertNotIn('trial.get("extensions_disabled") is True', surface_source)


if __name__ == "__main__":
    unittest.main()
