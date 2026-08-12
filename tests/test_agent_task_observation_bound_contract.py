"""Contract for bounded semantic-observation evidence in the controlled Agent Task."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskObservationBoundContractTests(unittest.TestCase):
    """Require the pinned-browser Agent Task to fail closed on oversized observations."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.namespace = runpy.run_path(
            str(RUNNER), run_name="agent_task_observation_bound_contract"
        )

    def test_semantic_observation_has_an_explicit_byte_limit(self) -> None:
        """The runner must expose one finite semantic-observation byte ceiling."""

        self.assertIn("MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES", self.namespace)
        maximum = self.namespace["MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES"]
        self.assertIsInstance(maximum, int)
        self.assertGreater(maximum, 0)
        self.assertLessEqual(maximum, 64 * 1024)

    def test_observation_measurement_accepts_exact_limit_and_rejects_overflow(self) -> None:
        """Canonical UTF-8 evidence at the ceiling is valid; one byte over fails closed."""

        self.assertIn("_measure_agent_task_semantic_observation_bytes", self.namespace)
        helper = self.namespace["_measure_agent_task_semantic_observation_bytes"]
        maximum = self.namespace["MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES"]

        # Canonical compact JSON for {"x":"..."} uses exactly eight structural bytes.
        exact = {"x": "a" * (maximum - 8)}
        oversized = {"x": "a" * (maximum - 7)}
        self.assertEqual(helper(exact), maximum)
        with self.assertRaises(ValueError):
            helper(oversized)
        with self.assertRaises(ValueError):
            helper({})
        with self.assertRaises(TypeError):
            helper("not-an-observation")

    def test_semantic_locator_text_is_utf8_bounded_before_comparison(self) -> None:
        """One hostile computed role/name must not consume the whole WebDriver response budget."""

        self.assertIn("MAX_AGENT_TASK_SEMANTIC_TEXT_BYTES", self.namespace)
        self.assertIn("_validate_agent_task_semantic_text", self.namespace)
        maximum = self.namespace["MAX_AGENT_TASK_SEMANTIC_TEXT_BYTES"]
        helper = self.namespace["_validate_agent_task_semantic_text"]

        self.assertIsInstance(maximum, int)
        self.assertGreater(maximum, 0)
        self.assertLessEqual(
            maximum, self.namespace["MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES"]
        )
        exact = "é" * (maximum // 2)
        oversized = exact + "é"
        self.assertEqual(helper(exact, "accessible name"), exact)
        with self.assertRaises(ValueError):
            helper(oversized, "accessible name")
        with self.assertRaises(TypeError):
            helper(7, "accessible name")

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn(
            '_validate_agent_task_semantic_text(role, "role")',
            runner,
        )
        self.assertIn(
            '_validate_agent_task_semantic_text(label, "accessible name")',
            runner,
        )

    def test_real_agent_task_path_uses_the_bounded_measurement_helper(self) -> None:
        """The real controlled browser pass must not bypass the bounded helper."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn(
            "semantic_observation_bytes = _measure_agent_task_semantic_observation_bytes(",
            runner,
        )


if __name__ == "__main__":
    unittest.main()
