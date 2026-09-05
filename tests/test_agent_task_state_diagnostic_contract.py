"""Regression contract for fail-closed, non-reflective Agent Task state diagnostics."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskStateDiagnosticContractTests(unittest.TestCase):
    """Keep page-controlled state values out of runner diagnostics."""

    def test_post_condition_failure_does_not_reflect_page_controlled_state(self) -> None:
        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_state_diagnostic_contract")
        validate = namespace["_validate_agent_task_submitted_state"]
        hostile_state = "rejected<script>buyer-secret-marker</script>"

        with self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task state post-condition failed$",
        ) as captured:
            validate(hostile_state)

        self.assertNotIn(hostile_state, str(captured.exception))
        validate("submitted")


if __name__ == "__main__":
    unittest.main()
