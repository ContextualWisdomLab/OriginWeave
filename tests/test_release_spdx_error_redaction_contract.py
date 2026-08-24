"""Regression tests for payload-safe SPDX parser failure diagnostics."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"
CONTEXT = "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"


class ReleaseSpdxErrorRedactionContractTests(unittest.TestCase):
    """Ensure raised validation errors retain no external document payload object."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.namespace = runpy.run_path(str(VALIDATOR), run_name="release_spdx_error_redaction")
        cls.validate = staticmethod(cls.namespace["validate_spdx_3_0_1_jsonld_bytes"])
        cls.error_type = cls.namespace["SpdxJsonLdEnvelopeError"]

    def _assert_payload_free_error(self, payload: bytes, expected_code: str) -> None:
        with self.assertRaises(self.error_type) as captured:
            self.validate(payload)
        error = captured.exception
        self.assertEqual(error.code, expected_code)
        self.assertIsNone(error.__cause__)
        self.assertIsNone(error.__context__)

    def test_invalid_utf8_does_not_survive_in_exception_chain(self) -> None:
        marker = b"buyer-secret-marker-must-not-survive-validation"
        self._assert_payload_free_error(marker + b"\xff", "invalid_utf8")

    def test_malformed_json_does_not_survive_in_exception_chain(self) -> None:
        marker = "buyer-secret-marker-must-not-survive-validation"
        payload = (
            '{"@context":"'
            + CONTEXT
            + '","@graph":[{"type":"SpdxDocument","private":"'
            + marker
            + '",}]}'
        ).encode("utf-8")
        self._assert_payload_free_error(payload, "invalid_json")


if __name__ == "__main__":
    unittest.main()
