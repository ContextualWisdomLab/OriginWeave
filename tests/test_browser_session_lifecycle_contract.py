"""Repository contracts for Browser Session presentation authority."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/originweave-browser-session"


class BrowserSessionLifecycleContractTests(unittest.TestCase):
    """Keep presentation mutation authority in an explicit Browser Session domain."""

    def test_browser_session_is_an_independent_workspace_boundary(self) -> None:
        """Browser Session authority must not be hidden in a driver adapter."""

        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn(
            "crates/originweave-browser-session",
            workspace["workspace"]["members"],
        )
        package = tomllib.loads((CRATE / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(
            package["dependencies"],
            {"originweave-core": {"path": "../originweave-core"}},
        )

    def test_domain_source_mints_authority_only_from_owned_lifecycle(self) -> None:
        """A raw driver identifier must never become a caller-mintable authority token."""

        source = (CRATE / "src/lib.rs").read_text(encoding="utf-8")
        self.assertIn("pub struct BrowserSession", source)
        self.assertIn("pub trait DisposableContextPort", source)
        self.assertIn("pub struct PresentationMutationAuthority", source)
        self.assertIn("create_disposable_context", source)
        self.assertIn("advance_context_epoch", source)
        self.assertIn("record_transport_loss", source)

        authority_impl = source.split("impl PresentationMutationAuthority", 1)[1].split(
            "enum OwnedContextState", 1
        )[0]
        self.assertNotIn("pub fn new", authority_impl)
        self.assertNotIn("pub const fn new", authority_impl)

    def test_architecture_decision_and_traceability_are_explicit(self) -> None:
        """Disposable ownership must remain a Proposed, standards-traced active-PR claim."""

        adr = (ROOT / "docs/adr/0114-browser-session-disposable-context-authority.md").read_text(
            encoding="utf-8"
        )
        trace = (ROOT / "docs/traceability/browser-session-lifecycle-authority.md").read_text(
            encoding="utf-8"
        )
        uml = (ROOT / "docs/uml/browser-session-lifecycle-authority.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("Status: Proposed", adr)
        self.assertIn("WD-webdriver-bidi-20260909", adr)
        self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", trace)
        self.assertIn("command ACK", trace)
        self.assertIn("PresentationMutationAuthority", uml)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)


if __name__ == "__main__":
    unittest.main()
