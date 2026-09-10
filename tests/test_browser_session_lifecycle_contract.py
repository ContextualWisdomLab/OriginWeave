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
        self.assertIn("pub struct BoundBrowserSession", source)
        self.assertIn("pub trait DisposableContextPort", source)
        self.assertIn("pub struct DisposableIsolationId", source)
        self.assertIn("pub struct DisposableContextHandle", source)
        self.assertIn("pub struct BrowserSessionIncarnation", source)
        self.assertIn("pub struct PresentationMutationAuthority", source)
        self.assertIn("pub struct DisposableContextCreateRequest", source)
        self.assertIn("pub struct DisposableContextDestroyRequest", source)
        self.assertIn("pub enum BrowserSessionRecoveryEvidence", source)
        self.assertIn("BrowserSessionState::RecoveryRequired", source)
        self.assertIn("pub enum DisposableContextCreateError", source)
        self.assertIn("pub enum DisposableContextDestroyError", source)
        self.assertNotIn("pub enum DisposableContextPortError", source)
        self.assertNotIn("DisposableContextPortId", source)
        self.assertNotIn("fn port_id(&self)", source)
        self.assertNotIn("pub fn create_disposable_context<P: DisposableContextPort>", source)
        self.assertIn("pub fn bind_lifecycle_port<P: DisposableContextPort>", source)
        self.assertIn("CreateFailedClean", source)
        self.assertIn("CreateFailedUncertain", source)
        self.assertIn("PartialCreationIsolation", source)
        self.assertIn("DuplicateAdapterHandle", source)
        self.assertIn("UnprovenDestruction", source)
        self.assertIn("create_disposable_context_with_port", source)
        self.assertIn("advance_context_epoch", source)
        self.assertIn("record_transport_loss", source)
        self.assertIn("transport_is_lost", source)
        self.assertIn("recovery_evidence", source)
        self.assertIn("user-context", source)
        self.assertIn("Reconstructing cleanup authority", source)
        self.assertIn("sequential_incarnation_reuse_rejects_stale_authority", source)

        authority_impl = source.split("impl PresentationMutationAuthority", 1)[1].split(
            "enum OwnedContextState", 1
        )[0]
        self.assertNotIn("pub fn new", authority_impl)
        self.assertNotIn("pub const fn new", authority_impl)

        create_request_impl = source.split("impl DisposableContextCreateRequest", 1)[1].split(
            "pub struct DisposableContextDestroyRequest", 1
        )[0]
        destroy_request_impl = source.split("impl DisposableContextDestroyRequest", 1)[1].split(
            "pub trait DisposableContextPort", 1
        )[0]
        self.assertNotIn("pub fn new", create_request_impl)
        self.assertNotIn("pub const fn new", create_request_impl)
        self.assertNotIn("pub fn new", destroy_request_impl)
        self.assertNotIn("pub const fn new", destroy_request_impl)

    def test_hostile_recovery_and_reincarnation_fixtures_remain_external(self) -> None:
        """Recovery, binding, and sequential reuse invariants must execute outside crate internals."""

        destroy_hostile = (
            CRATE / "tests/destroy_failure_requires_recovery.rs"
        ).read_text(encoding="utf-8")
        reincarnation_hostile = (
            CRATE / "tests/sequential_incarnation_reuse.rs"
        ).read_text(encoding="utf-8")
        preflight_hostile = (
            CRATE / "tests/lifecycle_port_preflight_side_effect.rs"
        ).read_text(encoding="utf-8")
        substitution_hostile = (
            CRATE / "tests/lifecycle_port_same_id_spoof.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "destroy_failure_requires_recovery_before_any_new_authority",
            destroy_hostile,
        )
        self.assertIn("BrowserSessionRecoveryEvidence::UnprovenDestruction", destroy_hostile)
        self.assertIn("assert!(bound.record_transport_loss());", destroy_hostile)
        self.assertIn("assert!(!bound.record_transport_loss());", destroy_hostile)
        self.assertIn(
            "stale_authority_cannot_cross_sequential_session_incarnations",
            reincarnation_hostile,
        )
        self.assertIn(
            "assert_ne!(\n        bound_a.browser_session().incarnation(),",
            reincarnation_hostile,
        )
        self.assertIn(
            "assert!(bound_b.lifecycle_port().destroy_incarnations.is_empty());",
            reincarnation_hostile,
        )
        self.assertIn(
            "lifecycle_binding_invokes_no_adapter_callback_before_authorized_create",
            preflight_hostile,
        )
        self.assertIn("identity_callbacks", preflight_hostile)
        self.assertIn(
            "distinct_adapter_cannot_be_substituted_for_create_after_binding",
            substitution_hostile,
        )
        self.assertIn(
            "distinct_adapter_cannot_be_substituted_for_destroy_after_binding",
            substitution_hostile,
        )

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
        self.assertIn("RecoveryRequired", adr)
        self.assertIn("BrowserSessionIncarnation", adr)
        self.assertIn("BrowserSessionRecoveryEvidence", adr)
        self.assertIn("DisposableContextCreateRequest", adr)
        self.assertIn("DisposableContextDestroyRequest", adr)
        self.assertIn("BoundBrowserSession", adr)
        self.assertIn("linear lifecycle-port binding", adr)
        self.assertIn("DisposableContextCreateError", adr)
        self.assertIn("DisposableContextDestroyError", adr)
        self.assertIn("CreateFailedClean", adr)
        self.assertIn("CreateFailedUncertain", adr)
        self.assertIn("transport liveness", adr)
        self.assertIn("sequential", adr)
        self.assertIn("unproven destruction", adr)
        self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", trace)
        self.assertIn("BoundBrowserSession", trace)
        self.assertIn("RecoveryRequired", trace)
        self.assertIn("BrowserSessionIncarnation", trace)
        self.assertIn("lossless recovery evidence", trace)
        self.assertIn("transport liveness", trace)
        self.assertIn("sequential ABA", trace)
        self.assertIn("command ACK", trace)
        self.assertIn("PresentationMutationAuthority", uml)
        self.assertIn("BoundBrowserSession", uml)
        self.assertIn("BrowserSessionIncarnation", uml)
        self.assertIn("RecoveryRequired", uml)
        self.assertIn("transport_lost", uml)
        self.assertIn("DisposableContextDestroyError / cleanup unproven", uml)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)


if __name__ == "__main__":
    unittest.main()
