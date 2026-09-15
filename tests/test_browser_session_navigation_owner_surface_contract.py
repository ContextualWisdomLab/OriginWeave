"""Owner-side repository contracts for Browser Session navigation authority."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "crates/originweave-browser-session/src/browser_session.rs"


class BrowserSessionNavigationOwnerSurfaceContractTests(unittest.TestCase):
    """Pin the navigation surface in the production owner, independent of stacked acceptance PRs."""

    def setUp(self) -> None:
        """Read the Browser Session owner source once for each contract assertion."""

        self.source = SOURCE_PATH.read_text(encoding="utf-8")

    def assert_source_pattern(self, pattern: str) -> None:
        """Require a structural source token without depending on rustfmt whitespace."""

        self.assertRegex(self.source, re.compile(pattern, re.MULTILINE | re.DOTALL))

    def test_navigation_capabilities_are_owner_issued_and_not_raw_id_constructible(self) -> None:
        """Navigation settlement must remain an opaque Browser Session capability."""

        for pattern in (
            r"pub\s+struct\s+NavigationSettlementAuthority\s*\{",
            r"pub\s+enum\s+NavigationTerminationOutcome\s*\{",
            r"pub\s+fn\s+record_observed_navigation\s*\(",
            r"pub\s+fn\s+record_observed_navigation_committed\s*\(",
            r"pub\s+fn\s+record_observed_navigation_settled\s*\(",
            r"pub\s+fn\s+record_observed_navigation_terminated\s*\(",
            r"pub\s+fn\s+record_observed_navigation_download_started\s*\(",
            r"pub\s+fn\s+reestablish_presentation_authority\s*\(",
            r"pub\s+fn\s+destroy_owned_disposable_context\s*\(",
        ):
            self.assert_source_pattern(pattern)

        witness_match = re.search(
            r"pub\s+struct\s+NavigationSettlementAuthority\s*\{(?P<body>.*?)\n\}",
            self.source,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(witness_match)
        witness_body = witness_match.group("body") if witness_match else ""
        self.assertNotRegex(witness_body, r"(?m)^\s*pub(?:\([^)]*\))?\s+")
        self.assertRegex(witness_body, r"navigation_generation\s*:\s*u64")
        self.assertRegex(witness_body, r"context_epoch\s*:\s*BrowserContextEpoch")
        self.assertNotRegex(
            self.source,
            r"impl\s+NavigationSettlementAuthority\b",
            "The settlement witness is intentionally opaque and has no inherent mint/read surface.",
        )

    def test_navigation_state_machine_keeps_liveness_separate_from_presentation_epoch(self) -> None:
        """Navigation generations close independently before explicit presentation re-establishment."""

        for pattern in (
            r"PresentationNavigationState\s*::\s*Pending",
            r"PresentationNavigationState\s*::\s*Eligible",
            r"reserve_navigation_generation\s*\(",
            r"mark_observed_navigation_committed\s*\(",
            r"close_observed_navigation\s*\(",
            r"reestablish_presentation_authority_for_context\s*\(",
            r"reserve_epoch\s*\(\s*&mut\s+self\.next_epoch\s*\)",
        ):
            self.assert_source_pattern(pattern)

        bound_start = re.search(
            r"impl\s*<\s*P\s*:\s*DisposableContextPort\s*>\s*BoundBrowserSession\s*<\s*P\s*>\s*\{",
            self.source,
        )
        operation_start = re.search(
            r"impl\s*<\s*P\s*:\s*AuthorizedContextOperationPort\s*>\s*BoundBrowserSession\s*<\s*P\s*>\s*\{",
            self.source,
        )
        self.assertIsNotNone(bound_start)
        self.assertIsNotNone(operation_start)
        self.assertLess(bound_start.start(), operation_start.start())
        bound_surface = self.source[bound_start.start() : operation_start.start()]
        compact_surface = "".join(bound_surface.split())
        self.assertIn(
            "self.session.begin_observed_navigation(incarnation,browsing_context,context_epoch)",
            compact_surface,
        )
        self.assertGreaterEqual(compact_surface.count("self.session.close_observed_navigation(authority)"), 3)
        self.assertRegex(bound_surface, r"NavigationTerminationOutcome\s*::\s*Aborted")
        self.assertRegex(bound_surface, r"NavigationTerminationOutcome\s*::\s*Failed")


if __name__ == "__main__":
    unittest.main()
