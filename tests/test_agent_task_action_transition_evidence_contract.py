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

        validate("idle", "")
        hostile = "buyer-secret-marker-must-not-reach-ci"
        for state, text in (("submitted", ""), ("idle", hostile)):
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

    def test_surface_completeness_requires_transition_baseline_evidence(self) -> None:
        """A post-condition without a pre-action baseline must not satisfy the buyer gate."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_transition_surface")
        complete = namespace["_agent_task_surfaces_complete"]
        trial = {
            "trial_number": 1,
            "passed": True,
            "post_condition": True,
            "input_echo_verified": True,
            "url_unchanged": True,
            "input_semantics_verified": True,
            "submit_semantics_verified": True,
            "profile_cleaned": True,
        }
        self.assertFalse(complete([trial]))
        trial["pre_action_baseline_verified"] = True
        self.assertTrue(complete([trial]))


if __name__ == "__main__":
    unittest.main()
