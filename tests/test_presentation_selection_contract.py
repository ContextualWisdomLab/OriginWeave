"""Guard presentation selection against unsupported randomized defaults."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class PresentationSelectionContractTests(unittest.TestCase):
    """Require evidence-backed cohorts before the kernel chooses a profile."""

    def test_kernel_does_not_offer_seeded_population_selection(self) -> None:
        """A seed must not invent population weights or observable identities."""
        source = (
            ROOT / "crates/originweave-fingerprint/src/lib.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("pub struct PresentationSeed", source)
        self.assertNotIn("pub fn derive(seed:", source)

        adr = (
            ROOT / "docs/adr/0110-privacy-preserving-presentation-identity.md"
        ).read_text(encoding="utf-8")
        self.assertIn("default profile selection remains unavailable", adr.lower())


if __name__ == "__main__":
    unittest.main()
