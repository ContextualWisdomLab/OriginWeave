"""Documentation contract for HTTP Content-Length persistence semantics."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"
DOCTORING = ROOT / "docs/doctoring/http-content-length-persistence.md"
REGRESSION = ROOT / "crates/originweave-http/tests/persistent_content_length.rs"


class HttpContentLengthPersistenceDocumentationContractTests(unittest.TestCase):
    """Keep the shipped claim aligned with the realistic framing regression."""

    def test_content_length_persistence_repair_is_release_noted(self) -> None:
        """The Unreleased record must state the exact persistence boundary."""

        changelog = CHANGELOG.read_text(encoding="utf-8")
        self.assertIn(
            "Corrected bounded HTTP `Content-Length` handling so a complete response returns "
            "at the declared message boundary without waiting for transport EOF, while "
            "already-buffered surplus remains fail closed.",
            changelog,
        )

    def test_persistence_claim_retains_primary_evidence_and_realistic_regression(self) -> None:
        """The release note must remain traceable to RFC 9112 and the TLS loopback case."""

        doctoring = DOCTORING.read_text(encoding="utf-8")
        regression = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("RFC 9112 §6.2", doctoring)
        self.assertIn("RFC 9112 §6.3", doctoring)
        self.assertIn("HTTP_TIMEOUT: Duration = Duration::from_millis(100)", regression)
        self.assertIn("KEEP_ALIVE_HOLD: Duration = Duration::from_millis(350)", regression)
        self.assertIn("BodyFraming::ContentLength(5)", regression)


if __name__ == "__main__":
    unittest.main()
