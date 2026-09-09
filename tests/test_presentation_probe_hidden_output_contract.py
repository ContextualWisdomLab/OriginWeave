"""Contract for hidden presentation-probe observations in pinned Chromium."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"


class PresentationProbeHiddenOutputContractTests(unittest.TestCase):
    """Keep hidden evidence values observable without script execution."""

    def test_hidden_probe_outputs_use_non_rendered_webdriver_property_reads(self) -> None:
        """Hidden fixture values must use textContent, not rendered Get Element Text."""

        namespace = runpy.run_path(str(RUNNER), run_name="presentation_hidden_output_contract")
        reader = inspect.getsource(namespace["_read_presentation_probe"])
        fixture = FIXTURE.read_text(encoding="utf-8")

        for element_id in (
            "presentation-viewport",
            "presentation-device-pixel-ratio",
            "presentation-timezone",
        ):
            with self.subTest(element_id=element_id):
                self.assertIn(f'id="{element_id}" hidden', fixture)

        self.assertIn('"/property/textContent"', reader)
        self.assertNotIn('"/text"', reader)
        self.assertNotIn('"/execute/sync"', reader)


if __name__ == "__main__":
    unittest.main()
