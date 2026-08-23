"""Fail-closed contract tests for bounded SPDX 3.0.1 JSON-LD envelope verification."""

from __future__ import annotations

import json
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"
CONTEXT = "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"


class ReleaseSpdxJsonLdEnvelopeContractTests(unittest.TestCase):
    """Verify exact serialization identity without promoting partial checks to SPDX conformance."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.namespace = runpy.run_path(str(VALIDATOR), run_name="release_spdx_envelope_contract")
        cls.validate = staticmethod(cls.namespace["validate_spdx_3_0_1_jsonld_bytes"])
        cls.error_type = cls.namespace["SpdxJsonLdEnvelopeError"]
        cls.max_bytes = cls.namespace["MAX_SPDX_JSONLD_BYTES"]
        cls.max_graph_objects = cls.namespace["MAX_SPDX_GRAPH_OBJECTS"]

    @staticmethod
    def _payload(graph: list[object], *, context: object = CONTEXT) -> bytes:
        return json.dumps(
            {"@context": context, "@graph": graph},
            separators=(",", ":"),
            ensure_ascii=True,
        ).encode("utf-8")

    def _assert_error_code(self, payload: bytes, expected_code: str) -> Exception:
        with self.assertRaises(self.error_type) as captured:
            self.validate(payload)
        self.assertEqual(captured.exception.code, expected_code)
        return captured.exception

    def test_exact_context_and_single_document_are_admitted(self) -> None:
        summary = self.validate(
            self._payload(
                [
                    {"type": "CreationInfo", "@id": "_:creation", "specVersion": "3.0.1"},
                    {
                        "type": "SpdxDocument",
                        "spdxId": "https://example.invalid/spdx/document",
                        "creationInfo": "_:creation",
                    },
                ]
            )
        )

        self.assertEqual(summary["context"], CONTEXT)
        self.assertEqual(summary["graph_object_count"], 2)
        self.assertEqual(summary["spdx_document_count"], 1)

    def test_context_array_is_rejected_until_schema_aware_validation(self) -> None:
        self._assert_error_code(
            self._payload(
                [{"type": "SpdxDocument"}],
                context=[CONTEXT, {"buyer": "https://example.invalid/spdx/buyer#"}],
            ),
            "invalid_context",
        )

    def test_inline_context_cannot_import_or_rebind_spdx_semantics(self) -> None:
        hostile_contexts = [
            [CONTEXT, {"@import": "https://example.invalid/moving-context.jsonld"}],
            [CONTEXT, {"@base": "https://example.invalid/base/"}],
            [CONTEXT, {"@vocab": "https://example.invalid/vocab#"}],
            [CONTEXT, {"type": "https://example.invalid/NotSpdxDocument"}],
            [CONTEXT, {"spdxId": "https://example.invalid/not-id"}],
        ]

        for context in hostile_contexts:
            with self.subTest(context=context):
                self._assert_error_code(
                    self._payload([{"type": "SpdxDocument"}], context=context),
                    "invalid_context",
                )

    def test_context_extensions_cannot_add_ambient_remote_authority(self) -> None:
        self._assert_error_code(
            self._payload(
                [{"type": "SpdxDocument"}],
                context=[CONTEXT, "https://example.invalid/moving-context.jsonld"],
            ),
            "invalid_context",
        )
        self._assert_error_code(
            self._payload([{"type": "SpdxDocument"}], context=[CONTEXT, None]),
            "invalid_context",
        )

    def test_context_must_match_exact_spdx_3_0_1_identity(self) -> None:
        self._assert_error_code(
            self._payload(
                [{"type": "SpdxDocument"}],
                context="https://spdx.org/rdf/3.1/spdx-context.jsonld",
            ),
            "invalid_context",
        )

    def test_top_level_object_must_contain_only_context_and_graph(self) -> None:
        payload = json.dumps(
            {"@context": CONTEXT, "@graph": [{"type": "SpdxDocument"}], "extra": True},
            separators=(",", ":"),
        ).encode("utf-8")
        self._assert_error_code(payload, "invalid_top_level")

    def test_duplicate_json_keys_fail_closed(self) -> None:
        payload = (
            '{"@context":"'
            + CONTEXT
            + '","@context":"'
            + CONTEXT
            + '","@graph":[{"type":"SpdxDocument"}]}'
        ).encode("utf-8")
        self._assert_error_code(payload, "duplicate_key")

    def test_graph_must_be_bounded_object_array(self) -> None:
        self._assert_error_code(
            self._payload([{"type": "SpdxDocument"}, "not-an-object"]),
            "invalid_graph_object",
        )
        oversized_graph = [{"type": "Package"}] * (self.max_graph_objects + 1)
        self._assert_error_code(self._payload(oversized_graph), "too_many_graph_objects")

    def test_exactly_one_spdx_document_is_required(self) -> None:
        self._assert_error_code(self._payload([{"type": "Package"}]), "invalid_document_count")
        self._assert_error_code(
            self._payload(
                [
                    {"type": "SpdxDocument"},
                    {"type": "SpdxDocument"},
                ]
            ),
            "invalid_document_count",
        )

    def test_invalid_utf8_and_nonfinite_json_fail_closed(self) -> None:
        self._assert_error_code(b"\xff", "invalid_utf8")
        payload = (
            '{"@context":"'
            + CONTEXT
            + '","@graph":[{"type":"SpdxDocument","score":NaN}]}'
        ).encode("utf-8")
        self._assert_error_code(payload, "invalid_json")

    def test_empty_and_oversized_payloads_fail_before_semantic_use(self) -> None:
        self._assert_error_code(b"", "invalid_size")
        self._assert_error_code(b" " * (self.max_bytes + 1), "invalid_size")

    def test_external_bytes_are_never_reflected_in_errors(self) -> None:
        marker = "buyer-secret-marker-must-not-reach-release-diagnostics"
        payload = self._payload(
            [
                {
                    "type": "SpdxDocument",
                    "unexpected": marker,
                }
            ],
            context=marker,
        )
        error = self._assert_error_code(payload, "invalid_context")
        self.assertNotIn(marker, str(error))


if __name__ == "__main__":
    unittest.main()
