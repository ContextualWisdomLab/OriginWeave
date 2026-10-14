"""Regression contracts for standards references that bind OriginWeave design claims."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCTORING = ROOT / "docs" / "doctoring.md"


class DoctoringReferenceContractTests(unittest.TestCase):
    """Keep cited primary-standard authorship aligned with the canonical source."""

    def test_rfc_5280_reference_uses_canonical_author_initials(self) -> None:
        """RFC 5280 must credit Sharon Boeyen as S. Boeyen, matching RFC Editor metadata."""
        text = DOCTORING.read_text(encoding="utf-8")
        expected = (
            "Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W. "
            "(2008). *Internet X.509 public key infrastructure certificate and certificate "
            "revocation list (CRL) profile* (RFC 5280). Internet Engineering Task Force. "
            "https://doi.org/10.17487/RFC5280"
        )
        self.assertIn(expected, text)

    def test_native_messaging_decision_trace_is_pinned(self) -> None:
        """Native messaging framing evidence must remain traceable and immutable."""
        text = DOCTORING.read_text(encoding="utf-8")
        compatibility = (
            ROOT / "docs" / "doctoring" / "mv3-compatibility.md"
        ).read_text(encoding="utf-8")
        for expected in (
            "### Native messaging framing and authority",
            "Chrome's native-messaging protocol uses a UTF-8 JSON payload",
            "64 MiB browser-to-host ceiling",
            "Chromium Authors. (2026). *native_message_process_host.cc*",
            "160af61f9d1316fd1f1dc41e9503cc1f1926d31f",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, text)
        expected_source_url = (
            "https://chromium.googlesource.com/chromium/src/+/"
            "160af61f9d1316fd1f1dc41e9503cc1f1926d31f/"
            "chrome/browser/extensions/api/messaging/native_message_process_host.cc"
        )
        expected_blob = "9d205a90d70b0c1c9f0b3b1c5f296528f6b21755"
        mutable_source_url = (
            "https://chromium.googlesource.com/chromium/src/+/refs/heads/main/"
            "chrome/browser/extensions/api/messaging/native_message_process_host.cc"
        )
        for document in (text, compatibility):
            with self.subTest(document=document[:24]):
                self.assertIn(expected_source_url, document)
                self.assertIn(expected_blob, document)
                self.assertNotIn(mutable_source_url, document)


if __name__ == "__main__":
    unittest.main()
