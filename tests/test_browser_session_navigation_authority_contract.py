"""Repository contract for navigation-invalidated presentation authority."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-browser-session/src/lib.rs"


class BrowserSessionNavigationAuthorityContractTests(unittest.TestCase):
    """Keep fresh presentation authority on the exact mutable bound owner."""

    def test_read_projection_cannot_mint_presentation_authority(self) -> None:
        """A raw browsing-context id on the read model must not mint authority."""

        source = SOURCE.read_text(encoding="utf-8")
        browser_session_impl = source.split("impl BrowserSession {", 1)[1].split(
            "impl<P: DisposableContextPort> BoundBrowserSession<P>", 1
        )[0]

        self.assertNotIn("pub fn presentation_authority(", browser_session_impl)
        self.assertIn("pub fn record_observed_navigation(", source)
        self.assertIn("pub fn reestablish_presentation_authority(", source)


if __name__ == "__main__":
    unittest.main()
