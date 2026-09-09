"""Contract for causal Agent Task action-transition evidence in pinned Chrome."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskActionTransitionEvidenceContractTests(unittest.TestCase):
    """Require false baselines before accepting an action-caused success state."""

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

    def test_typed_input_validator_is_closed_and_non_echoing(self) -> None:
        """Typed-value evidence must fail closed without echoing page-controlled input."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_typed_value_contract")
        self.assertIn("_validate_agent_task_typed_value", namespace)
        validate = namespace["_validate_agent_task_typed_value"]

        validate(namespace["AGENT_TASK_INPUT_VALUE"])
        hostile = "buyer-secret-marker-must-not-reach-ci"
        with self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task typed input verification failed$",
        ) as raised:
            validate(hostile)
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

    def test_typed_input_value_is_observed_before_pre_click_baseline(self) -> None:
        """Native send-keys acknowledgement must not substitute for observed input state."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_typed_value_order")
        source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        typing = source.index('"/value"')
        value_observation = source.find('"/property/value"', typing)
        typed_validation = source.find("_validate_agent_task_typed_value", typing)
        pre_click_baseline = source.find("_validate_agent_task_pre_action_state", typing)
        native_click = source.index('"/click"')
        self.assertNotEqual(value_observation, -1)
        self.assertNotEqual(typed_validation, -1)
        self.assertLess(typing, value_observation)
        self.assertLess(value_observation, typed_validation)
        self.assertLess(typed_validation, pre_click_baseline)
        self.assertLess(pre_click_baseline, native_click)
        self.assertIn('"input_value_verified": True', source)

    def test_pre_click_baseline_happens_after_typing_and_before_click(self) -> None:
        """Typing must not be able to pre-satisfy the submit post-condition."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_pre_click_order")
        source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        typing = source.index('"/value"')
        pre_click_baseline = source.find("_validate_agent_task_pre_action_state", typing)
        native_click = source.index('"/click"')
        self.assertNotEqual(pre_click_baseline, -1)
        self.assertLess(typing, pre_click_baseline)
        self.assertLess(pre_click_baseline, native_click)
        self.assertIn('"pre_click_baseline_verified": True', source)

    def test_url_stability_is_observed_through_the_accepted_post_condition(self) -> None:
        """A delayed navigation must not escape the accepted action-outcome boundary."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_final_url_order")
        source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        native_click = source.index('"/click"')
        post_condition = source.index("_validate_agent_task_submitted_state", native_click)
        exact_echo = source.index("if text != AGENT_TASK_INPUT_VALUE", post_condition)
        final_url_read = source.find('"/url"', exact_echo)
        success_evidence = source.index("return {", exact_echo)
        self.assertNotEqual(final_url_read, -1)
        self.assertLess(exact_echo, final_url_read)
        self.assertLess(final_url_read, success_evidence)
        self.assertIn("Agent Task URL changed before accepted outcome", source)

    def test_surface_completeness_requires_transition_baseline_evidence(self) -> None:
        """Post-condition evidence must include typed-value and both baseline witnesses."""

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
        self.assertFalse(complete([trial]))
        trial["input_value_verified"] = True
        self.assertFalse(complete([trial]))
        trial["pre_click_baseline_verified"] = True
        self.assertTrue(complete([trial]))


if __name__ == "__main__":
    unittest.main()
