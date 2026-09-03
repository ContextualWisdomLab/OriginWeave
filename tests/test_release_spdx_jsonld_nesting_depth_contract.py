"""Regression contract for bounded JSON nesting before SPDX tree materialization."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"


class ReleaseSpdxJsonLdNestingDepthContractTests(unittest.TestCase):
    """Require a product-owned nesting budget before the generic JSON parser."""

    def test_nesting_beyond_product_budget_fails_in_preflight(self) -> None:
        """Deep valid JSON must fail before parser recursion becomes the resource boundary."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_nesting_depth_contract")
        max_depth = namespace["MAX_SPDX_JSON_NESTING_DEPTH"]
        preflight = namespace["_reject_excessive_json_structure"]
        envelope_error = namespace["SpdxJsonLdEnvelopeError"]

        text = "[" * (max_depth + 1) + "0" + "]" * (max_depth + 1)
        with self.assertRaises(envelope_error) as captured:
            preflight(text)

        self.assertEqual(captured.exception.code, "too_deep_json_structure")

    def test_structural_characters_inside_strings_do_not_consume_depth_budget(self) -> None:
        """Bracket-looking string content must remain inert for the nesting budget."""

        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_nesting_string_contract")
        max_depth = namespace["MAX_SPDX_JSON_NESTING_DEPTH"]
        preflight = namespace["_reject_excessive_json_structure"]

        preflight('"' + ("[" * (max_depth + 1)) + '"')


if __name__ == "__main__":
    unittest.main()
