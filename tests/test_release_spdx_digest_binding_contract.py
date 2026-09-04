"""Regression contract binding validated SPDX bytes to the declared release artifact digest."""

from __future__ import annotations

import hashlib
import json
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"
CONTEXT = "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"


class ReleaseSpdxDigestBindingContractTests(unittest.TestCase):
    """Prevent a valid but substituted SPDX document from satisfying release evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_digest_binding_contract")
        cls.validate_bound = staticmethod(namespace["validate_release_spdx_3_0_1_jsonld_bytes"])
        cls.error_type = namespace["SpdxJsonLdEnvelopeError"]

    @staticmethod
    def _payload(marker: str = "release-a") -> bytes:
        return json.dumps(
            {
                "@context": CONTEXT,
                "@graph": [{"type": "SpdxDocument", "spdxId": marker}],
            },
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")

    @staticmethod
    def _digest(payload: bytes) -> str:
        return "sha256:" + hashlib.sha256(payload).hexdigest()

    def test_exact_candidate_bytes_match_the_manifest_backed_sbom_digest(self) -> None:
        payload = self._payload()
        expected_digest = self._digest(payload)

        summary = self.validate_bound(payload, expected_digest)

        self.assertEqual(summary["artifact_sha256"], expected_digest)
        self.assertEqual(summary["spdx_document_count"], 1)

    def test_valid_but_substituted_document_fails_closed_on_digest_mismatch(self) -> None:
        declared = self._payload("declared-release")
        substituted = self._payload("substituted-release-secret-marker")

        with self.assertRaises(self.error_type) as captured:
            self.validate_bound(substituted, self._digest(declared))

        self.assertEqual(captured.exception.code, "digest_mismatch")
        self.assertNotIn("substituted-release-secret-marker", str(captured.exception))

    def test_expected_digest_must_use_canonical_lowercase_sha256_identity(self) -> None:
        payload = self._payload()
        valid = self._digest(payload)
        invalid_digests = (
            valid.removeprefix("sha256:"),
            "SHA256:" + valid.removeprefix("sha256:"),
            "sha256:" + valid.removeprefix("sha256:").upper(),
            "sha256:" + "0" * 63,
        )

        for digest in invalid_digests:
            with self.subTest(digest=digest):
                with self.assertRaises(self.error_type) as captured:
                    self.validate_bound(payload, digest)
                self.assertEqual(captured.exception.code, "invalid_expected_digest")


if __name__ == "__main__":
    unittest.main()
