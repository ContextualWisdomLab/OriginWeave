"""Owner-side repository contracts for Browser Session navigation authority."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "crates/originweave-browser-session/src/browser_session.rs"


class BrowserSessionNavigationOwnerSurfaceContractTests(unittest.TestCase):
    """Pin the navigation surface in the production owner, independent of stacked acceptance PRs."""

    def setUp(self) -> None:
        """Read the Browser Session owner source once for each contract assertion."""

        self.source = SOURCE_PATH.read_text(encoding="utf-8")

    def test_navigation_capabilities_are_owner_issued_and_not_raw_id_constructible(self) -> None:
        """Navigation settlement must remain an opaque Browser Session capability."""

        for symbol in (
            "pub struct NavigationSettlementAuthority",
            "pub enum NavigationTerminationOutcome",
            "pub fn record_observed_navigation(",
            "pub fn record_observed_navigation_committed(",
            "pub fn record_observed_navigation_settled(",
            "pub fn record_observed_navigation_terminated(",
            "pub fn record_observed_navigation_download_started(",
            "pub fn reestablish_presentation_authority(",
            "pub fn destroy_owned_disposable_context(",
        ):
            self.assertIn(symbol, self.source)

        witness_declaration = self.source.split(
            "pub struct NavigationSettlementAuthority", 1
        )[1].split("pub enum NavigationTerminationOutcome", 1)[0]
        self.assertNotIn("pub fn new", witness_declaration)
        self.assertNotIn("pub const fn new", witness_declaration)
        self.assertIn("navigation_generation: u64", witness_declaration)
        self.assertIn("context_epoch: BrowserContextEpoch", witness_declaration)

    def test_navigation_state_machine_keeps_liveness_separate_from_presentation_epoch(self) -> None:
        """Navigation generations close independently before explicit presentation re-establishment."""

        for token in (
            "PresentationNavigationState::Pending",
            "PresentationNavigationState::Eligible",
            "reserve_navigation_generation",
            "mark_observed_navigation_committed",
            "close_observed_navigation",
            "reestablish_presentation_authority_for_context",
            "reserve_epoch(&mut self.next_epoch)",
        ):
            self.assertIn(token, self.source)

        bound_surface = self.source.split(
            "impl<P: DisposableContextPort> BoundBrowserSession<P>", 1
        )[1].split("impl<P: AuthorizedContextOperationPort> BoundBrowserSession<P>", 1)[0]
        compact_surface = "".join(bound_surface.split())
        self.assertIn(
            "self.session.begin_observed_navigation(incarnation,browsing_context,context_epoch)",
            compact_surface,
        )
        self.assertGreaterEqual(compact_surface.count("self.session.close_observed_navigation(authority)"), 3)
        self.assertIn("NavigationTerminationOutcome::Aborted", bound_surface)
        self.assertIn("NavigationTerminationOutcome::Failed", bound_surface)


if __name__ == "__main__":
    unittest.main()
