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
        self.assertIn("pub fn record_observed_navigation_terminated(", source)
        self.assertIn("pub fn reestablish_presentation_authority(", source)

    def test_navigation_observation_is_bound_to_exact_session_context_generation(self) -> None:
        """A delayed event must not alias across aggregate incarnations, contexts, or epochs."""

        source = SOURCE.read_text(encoding="utf-8")
        signature = re.search(
            r"pub fn record_observed_navigation\s*\((?P<params>.*?)\)\s*->(?P<return_type>[^\{]+)\{",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("BrowserSessionIncarnation", params)
        self.assertIn("BrowsingContextId", params)
        self.assertIn("BrowserContextEpoch", params)
        self.assertIn(
            "NavigationSettlementAuthority",
            signature.group("return_type"),
            "an admitted navigation start must issue an opaque aggregate-bound settlement authority",
        )

    def test_navigation_settlement_requires_aggregate_issued_authority(self) -> None:
        """Raw provenance must never become authority to unlock presentation re-establishment."""

        source = SOURCE.read_text(encoding="utf-8")
        signature = re.search(
            r"pub fn record_observed_navigation_settled\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("NavigationSettlementAuthority", params)
        self.assertNotIn(
            "BrowserSessionIncarnation",
            params,
            "session incarnation is provenance for minting the settlement authority, not settlement authority itself",
        )
        self.assertNotIn(
            "BrowsingContextId",
            params,
            "raw browser addressability must not unlock a pending navigation",
        )
        self.assertNotIn(
            "BrowserContextEpoch",
            params,
            "a reconstructible epoch is correlation evidence, not a settlement capability",
        )

    def test_navigation_abort_and_failure_have_explicit_terminal_transition(self) -> None:
        """Negative terminal events must close pending state without pretending they committed."""

        source = SOURCE.read_text(encoding="utf-8")
        signature = re.search(
            r"pub fn record_observed_navigation_terminated\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )
        declaration = re.search(
            r"pub enum NavigationTerminationOutcome\s*\{(?P<body>.*?)\}",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("NavigationSettlementAuthority", params)
        self.assertIn("NavigationTerminationOutcome", params)
        self.assertNotIn("BrowserSessionIncarnation", params)
        self.assertNotIn("BrowsingContextId", params)
        self.assertNotIn("BrowserContextEpoch", params)

        self.assertIsNotNone(declaration)
        body = declaration.group("body")
        self.assertRegex(body, r"\bAborted\b")
        self.assertRegex(body, r"\bFailed\b")

    def test_navigation_settlement_authority_is_not_publicly_constructible(self) -> None:
        """Only Browser Session may mint the witness that unlocks settlement."""

        source = SOURCE.read_text(encoding="utf-8")
        declaration = re.search(
            r"pub struct NavigationSettlementAuthority\s*\{(?P<body>.*?)\}",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(declaration)
        self.assertNotRegex(
            declaration.group("body"),
            r"\bpub(?:\([^)]*\))?\s+\w+\s*:",
            "settlement-authority state must remain private to the Browser Session crate",
        )
        self.assertNotIn(
            "impl NavigationSettlementAuthority {\n    pub fn new(",
            source,
            "raw callers must not reconstruct settlement authority through a public constructor",
        )


if __name__ == "__main__":
    unittest.main()
