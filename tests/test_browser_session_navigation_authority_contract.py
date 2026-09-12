"""Repository contract for navigation-invalidated presentation authority."""

from __future__ import annotations

import pathlib
import re
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
        self.assertIn("pub fn record_observed_navigation_settled(", source)
        self.assertIn("pub fn reestablish_presentation_authority(", source)

    def test_navigation_observation_is_bound_to_exact_session_context_generation(self) -> None:
        """A delayed event must not alias across aggregate incarnations, contexts, or epochs."""

        source = SOURCE.read_text(encoding="utf-8")
        signature = re.search(
            r"pub fn record_observed_navigation\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("BrowserSessionIncarnation", params)
        self.assertIn("BrowsingContextId", params)
        self.assertIn("BrowserContextEpoch", params)

    def test_navigation_settlement_is_bound_to_the_same_domain_generation(self) -> None:
        """Only a matching browser-settled generation may reopen re-establishment eligibility."""

        source = SOURCE.read_text(encoding="utf-8")
        signature = re.search(
            r"pub fn record_observed_navigation_settled\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("BrowserSessionIncarnation", params)
        self.assertIn("BrowsingContextId", params)
        self.assertIn("BrowserContextEpoch", params)


if __name__ == "__main__":
    unittest.main()
