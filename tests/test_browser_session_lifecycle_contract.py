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
        """Raw driver identifiers must never become caller-mintable authority tokens."""

        source = (CRATE / "src/lib.rs").read_text(encoding="utf-8")
        self.assertIn("pub struct BrowserSession", source)
        self.assertIn("pub trait DisposableContextPort", source)
        self.assertIn("pub struct DisposableIsolationId", source)
        self.assertIn("pub struct DisposableContextHandle", source)
        self.assertIn("pub struct BrowserSessionIncarnation", source)
        self.assertIn("pub struct PresentationMutationAuthority", source)
        self.assertIn("pub enum BrowserSessionRecoveryEvidence", source)
        self.assertIn("BrowserSessionState::RecoveryRequired", source)
        self.assertIn("pub enum DisposableContextCreateError", source)
        self.assertIn("pub enum DisposableContextDestroyError", source)
        self.assertNotIn("pub enum DisposableContextPortError", source)
        self.assertIn("CreateFailedClean", source)
        self.assertIn("CreateFailedUncertain", source)
        self.assertIn("PartialCreationIsolation", source)
        self.assertIn("DuplicateAdapterHandle", source)
        self.assertIn("UnprovenDestruction", source)
        self.assertIn("create_disposable_context", source)
        self.assertIn("advance_context_epoch", source)
        self.assertIn("record_transport_loss", source)
        self.assertIn("transport_is_lost", source)
        self.assertIn("recovery_evidence", source)
        self.assertIn("user-context identifier", source)
        self.assertIn("Reconstructing cleanup authority", source)
        self.assertIn("sequential_incarnation_reuse_rejects_stale_authority", source)

        authority_impl = source.split("impl PresentationMutationAuthority", 1)[1].split(
            "enum OwnedContextState", 1
        )[0]
        self.assertNotIn("pub fn new", authority_impl)
        self.assertNotIn("pub const fn new", authority_impl)

    def test_hostile_recovery_and_reincarnation_fixtures_remain_external(self) -> None:
        """Recovery and sequential reuse invariants must be executable outside crate internals."""

        destroy_hostile = (
            CRATE / "tests/destroy_failure_requires_recovery.rs"
        ).read_text(encoding="utf-8")
        reincarnation_hostile = (
            CRATE / "tests/sequential_incarnation_reuse.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "destroy_failure_requires_recovery_before_any_new_authority",
            destroy_hostile,
        )
        self.assertIn("BrowserSessionRecoveryEvidence::UnprovenDestruction", destroy_hostile)
        self.assertIn("assert!(session.record_transport_loss());", destroy_hostile)
        self.assertIn("assert!(!session.record_transport_loss());", destroy_hostile)
        self.assertIn(
            "stale_authority_cannot_cross_sequential_session_incarnations",
            reincarnation_hostile,
        )
        self.assertIn("assert_ne!(session_a.incarnation(), session_b.incarnation());", reincarnation_hostile)
        self.assertIn("assert!(port_b.destroy_incarnations.is_empty());", reincarnation_hostile)

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
        self.assertIn("WD-webdriver-bidi-20260818", adr)
        self.assertIn("RecoveryRequired", adr)
        self.assertIn("BrowserSessionIncarnation", adr)
        self.assertIn("BrowserSessionRecoveryEvidence", adr)
        self.assertIn("DisposableContextCreateError", adr)
        self.assertIn("DisposableContextDestroyError", adr)
        self.assertIn("CreateFailedClean", adr)
        self.assertIn("CreateFailedUncertain", adr)
        self.assertIn("transport liveness", adr)
        self.assertIn("sequential", adr)
        self.assertIn("unproven destruction", adr)
        self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", trace)
        self.assertIn("RecoveryRequired", trace)
        self.assertIn("BrowserSessionIncarnation", trace)
        self.assertIn("lossless recovery evidence", trace)
        self.assertIn("transport liveness", trace)
        self.assertIn("sequential ABA", trace)
        self.assertIn("command ACK", trace)
        self.assertIn("PresentationMutationAuthority", uml)
        self.assertIn("BrowserSessionIncarnation", uml)
        self.assertIn("RecoveryRequired", uml)
        self.assertIn("transport_lost", uml)
        self.assertIn("DisposableContextDestroyError / cleanup unproven", uml)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)


if __name__ == "__main__":
    unittest.main()
