"""Contract for causal Agent Task action-transition evidence in pinned Chrome."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskActionTransitionEvidenceContractTests(unittest.TestCase):
    """Require a false pre-action baseline before accepting a post-action success state."""

    def test_pre_action_baseline_validator_is_closed_and_non_echoing(self) -> None:
        """A pre-fired fixture must fail without echoing page-controlled state or text."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_transition_contract")
        self.assertIn("_validate_agent_task_pre_action_state", namespace)
        validate = namespace["_validate_agent_task_pre_action_state"]

        validate("idle", "idle")
        hostile = "buyer-secret-marker-must-not-reach-ci"
        for state, text in (("submitted", "idle"), ("idle", hostile)):
            with self.subTest(state=state, text=text), self.assertRaisesRegex(
                RuntimeError,
                r"^Agent Task pre-action baseline was already satisfied$",
            ) as raised:
                validate(state, text)
            self.assertNotIn(hostile, str(raised.exception))

    def test_pre_action_observation_happens_before_native_click(self) -> None:
        """Evidence must prove baseline→native action→post-condition ordering."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_transition_order")
        source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        baseline = source.index("_validate_agent_task_pre_action_state")
        native_click = source.index('"/click"')
        post_condition = source.index("_validate_agent_task_submitted_state")
        self.assertLess(baseline, native_click)
        self.assertLess(native_click, post_condition)
        self.assertIn('"pre_action_baseline_verified": True', source)

    def test_trial_evidence_propagates_the_transition_baseline(self) -> None:
        """The per-trial evidence must retain the observed causal-baseline witness."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_transition_surface")
        browser_pass = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        trial = inspect.getsource(namespace["_run_agent_task_trial"])
        self.assertIn('"pre_action_baseline_verified": True', browser_pass)
        self.assertIn(
            '"pre_action_baseline_verified": result["pre_action_baseline_verified"]',
            trial,
        )


if __name__ == "__main__":
    unittest.main()
