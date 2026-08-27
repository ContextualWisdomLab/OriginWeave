"""Documentation regression for the bounded HTTP framing type contract."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DESIGN = ROOT / "docs/superpowers/specs/2026-08-07-http11-semantics-design.md"


class HttpFramingDocumentationContractTests(unittest.TestCase):
    """Keep the approved design aligned with the implemented framing type."""

    def test_content_length_contract_is_platform_sized(self) -> None:
        """The design must not claim a u64 API when production exposes usize."""

        design = DESIGN.read_text(encoding="utf-8")
        self.assertIn("ContentLength(usize)", design)
        self.assertNotIn("ContentLength(u64)", design)
        self.assertIn("fits `usize` and the encoded-content budget", design)


if __name__ == "__main__":
    unittest.main()
