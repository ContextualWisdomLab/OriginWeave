"""Regression contract for manifest-backed SPDX validation through the release CLI."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import pathlib
import runpy
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "release" / "validate_spdx_jsonld.py"
CONTEXT = "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"


class ReleaseSpdxCliDigestContractTests(unittest.TestCase):
    """Require release CLI callers to opt into exact manifest-backed digest evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        namespace = runpy.run_path(str(VALIDATOR), run_name="spdx_cli_digest_contract")
        cls.main = staticmethod(namespace["main"])

    @staticmethod
    def _payload() -> bytes:
        return json.dumps(
            {
                "@context": CONTEXT,
                "@graph": [{"type": "SpdxDocument", "spdxId": "release-cli"}],
            },
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")

    @staticmethod
    def _digest(payload: bytes) -> str:
        return "sha256:" + hashlib.sha256(payload).hexdigest()

    def test_cli_can_bind_direct_file_to_manifest_backed_digest(self) -> None:
        payload = self._payload()
        expected_digest = self._digest(payload)

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-cli-") as directory:
            document = pathlib.Path(directory) / "release.spdx.jsonld"
            document.write_bytes(payload)
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                status = self.main(
                    [str(document), "--expected-sha256", expected_digest]
                )

        self.assertEqual(status, 0)
        summary = json.loads(output.getvalue())
        self.assertEqual(summary["artifact_sha256"], expected_digest)
        self.assertEqual(summary["spdx_document_count"], 1)

    def test_cli_digest_mismatch_fails_closed_without_reflecting_identity_or_path(self) -> None:
        payload = self._payload()
        wrong_digest = "sha256:" + "0" * 64

        with tempfile.TemporaryDirectory(prefix="originweave-spdx-cli-secret-") as directory:
            document = pathlib.Path(directory) / "customer-secret-release.spdx.jsonld"
            document.write_bytes(payload)
            error_output = io.StringIO()
            with contextlib.redirect_stderr(error_output):
                status = self.main([str(document), "--expected-sha256", wrong_digest])

        diagnostic = error_output.getvalue()
        self.assertEqual(status, 2)
        self.assertIn("digest_mismatch", diagnostic)
        self.assertNotIn(wrong_digest, diagnostic)
        self.assertNotIn("customer-secret-release.spdx.jsonld", diagnostic)


if __name__ == "__main__":
    unittest.main()
