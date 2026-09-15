"""Repository contract for navigation-to-download authority liveness."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-browser-session/src/browser_session.rs"

BASE_NAVIGATION_MARKERS = (
    "pub struct NavigationSettlementAuthority",
    "pub fn record_observed_navigation(",
    "pub fn reestablish_presentation_authority(",
)


class BrowserSessionNavigationDownloadContractTests(unittest.TestCase):
    """Keep navigation-to-download liveness inside aggregate-owned authority."""

    def test_download_start_transition_is_witness_bound(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        if not any(marker in source for marker in BASE_NAVIGATION_MARKERS):
            self.skipTest("Browser Session navigation production API is not implemented yet")

        self.assertIn(
            "pub fn record_observed_navigation_download_started(",
            source,
            "a production navigation slice must close navigation-to-download without stranding presentation authority",
        )
        signature = re.search(
            r"pub fn record_observed_navigation_download_started\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("NavigationSettlementAuthority", params)
        self.assertNotIn("BrowserSessionIncarnation", params)
        self.assertNotIn("BrowsingContextId", params)
        self.assertNotIn("BrowserContextEpoch", params)


if __name__ == "__main__":
    unittest.main()
