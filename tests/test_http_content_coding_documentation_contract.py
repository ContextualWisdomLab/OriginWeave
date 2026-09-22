"""Keep HTTP content-coding documentation aligned with executable decoder semantics."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DESIGN = ROOT / "docs/superpowers/specs/2026-08-07-http11-semantics-design.md"
CONTENT = ROOT / "crates/originweave-http/src/content.rs"


class HttpContentCodingDocumentationContractTests(unittest.TestCase):
    """Prevent reviewed HTTP decoder behavior from drifting behind the design record."""

    def test_design_records_current_gzip_and_deflate_compatibility(self) -> None:
        """The approved design must describe the decoder behavior buyers actually receive."""

        design = DESIGN.read_text(encoding="utf-8")
        content = CONTENT.read_text(encoding="utf-8")

        self.assertIn("MultiGzDecoder", content)
        self.assertIn("DeflateRawCompatibility", content)
        self.assertIn("concatenated RFC 1952 members", design)
        self.assertIn("raw RFC 1951 DEFLATE compatibility fallback", design)
        self.assertNotIn("Multiple codings, raw deflate fallback", design)


if __name__ == "__main__":
    unittest.main()
