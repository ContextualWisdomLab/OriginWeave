"""Contract for the controlled Agent Task semantic-observation evidence schema."""

from __future__ import annotations

import ast
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskSemanticObservationSchemaContractTests(unittest.TestCase):
    """Keep unreviewed page content outside the controlled semantic evidence object."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.tree = ast.parse(RUNNER.read_text(encoding="utf-8"), filename=str(RUNNER))

    @staticmethod
    def _literal_dict_keys(node: ast.Dict) -> tuple[str, ...]:
        """Return exact string-literal dictionary keys or fail the contract."""

        keys: list[str] = []
        for key in node.keys:
            if not isinstance(key, ast.Constant) or not isinstance(key.value, str):
                raise AssertionError("semantic observation keys must be string literals")
            keys.append(key.value)
        return tuple(keys)

    def _semantic_observation_assignment(self) -> ast.Dict:
        """Find the one executable controlled-observation construction."""

        assignments = [
            node
            for node in ast.walk(self.tree)
            if isinstance(node, ast.Assign)
            and any(
                isinstance(target, ast.Name) and target.id == "semantic_observation"
                for target in node.targets
            )
        ]
        self.assertEqual(len(assignments), 1)
        value = assignments[0].value
        self.assertIsInstance(value, ast.Dict)
        return value

    def test_controlled_observation_has_only_reviewed_role_name_fields(self) -> None:
        """Raw page text or instruction-like fields cannot drift into emitted evidence."""

        observation = self._semantic_observation_assignment()
        self.assertEqual(self._literal_dict_keys(observation), ("input", "submit"))
        semantic_nodes = dict(zip(self._literal_dict_keys(observation), observation.values))

        expected_values = {
            "input": {"role": "input_role", "name": "input_name"},
            "submit": {"role": "submit_role", "name": "submit_name"},
        }
        for semantic_key, expected_fields in expected_values.items():
            semantic_node = semantic_nodes[semantic_key]
            self.assertIsInstance(semantic_node, ast.Dict)
            self.assertEqual(self._literal_dict_keys(semantic_node), ("role", "name"))
            actual_fields = dict(
                zip(self._literal_dict_keys(semantic_node), semantic_node.values)
            )
            for field_name, expected_variable in expected_fields.items():
                value = actual_fields[field_name]
                self.assertIsInstance(value, ast.Name)
                self.assertEqual(value.id, expected_variable)

    def test_exact_observation_flows_through_bounded_measurement(self) -> None:
        """The reviewed object must be the exact object sent to the byte-bound helper."""

        calls = [
            node
            for node in ast.walk(self.tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "_measure_agent_task_semantic_observation_bytes"
        ]
        self.assertEqual(len(calls), 1)
        self.assertEqual(len(calls[0].args), 1)
        argument = calls[0].args[0]
        self.assertIsInstance(argument, ast.Name)
        self.assertEqual(argument.id, "semantic_observation")
        self.assertFalse(calls[0].keywords)


if __name__ == "__main__":
    unittest.main()
