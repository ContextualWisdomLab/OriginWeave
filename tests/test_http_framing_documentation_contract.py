"""Documentation regression for the bounded HTTP framing type contract."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DESIGN = ROOT / "docs/superpowers/specs/2026-08-07-http11-semantics-design.md"
ERRATA = ROOT / "docs/superpowers/specs/2026-08-07-http11-semantics-errata.md"
FRAMING = ROOT / "crates/originweave-http/src/framing.rs"


class HttpFramingDocumentationContractTests(unittest.TestCase):
    """Keep the approved design aligned with the implemented framing type."""

    def test_content_length_contract_is_platform_sized(self) -> None:
        """A stale u64 design line must be explicitly superseded by normative errata."""

        source = FRAMING.read_text(encoding="utf-8")
        design = DESIGN.read_text(encoding="utf-8")
        self.assertIn("ContentLength(usize)", source)

        if "ContentLength(usize)" not in design:
            self.assertIn("ContentLength(u64)", design)
            errata = ERRATA.read_text(encoding="utf-8")
            self.assertIn("Active normative correction", errata)
            self.assertIn("ContentLength(usize)", errata)
            self.assertIn("superseded", errata)
            self.assertIn("fit `usize` and the encoded-content budget", errata)


if __name__ == "__main__":
    unittest.main()
