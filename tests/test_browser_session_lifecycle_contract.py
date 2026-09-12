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
        required_symbols = (
            "pub struct BrowserSession",
            "pub struct BoundBrowserSession",
            "pub trait DisposableContextPort",
            "pub struct DisposableIsolationId",
            "pub struct DisposableContextHandle",
            "pub struct BrowserSessionIncarnation",
            "pub struct PresentationMutationAuthority",
            "pub struct DisposableContextCreateRequest",
            "pub struct DisposableContextCreateCompletion",
            "pub enum DisposableContextCreateDisposition",
            "pub enum DisposableContextCreateCompletionError",
            "pub struct DisposableContextDestroyRequest",
            "pub enum BrowserSessionRecoveryEvidence",
            "pub struct AuthorizedContextOperationRequest",
            "pub enum AuthorizedContextOperationError",
            "pub trait AuthorizedContextOperationPort",
            "pub enum DisposableContextCreateError",
            "pub enum DisposableContextDestroyError",
            "pub fn abandoned_bound_session_count",
            "pub fn finish",
            "pub fn execute_authorized_context_operation",
        )
        for symbol in required_symbols:
            self.assertIn(symbol, source)

        self.assertIn("BrowserSessionState::RecoveryRequired", source)
        self.assertNotIn("pub enum DisposableContextPortError", source)
        self.assertNotIn("DisposableContextPortId", source)
        self.assertNotIn("fn port_id(&self)", source)
        self.assertNotIn("pub const fn lifecycle_port", source)
        self.assertNotIn("pub fn lifecycle_port", source)
        self.assertNotIn("pub fn create_disposable_context<P: DisposableContextPort>", source)
        self.assertIn("pub fn bind_lifecycle_port<P: DisposableContextPort>", source)
        self.assertIn("attempt_epoch: BrowserContextEpoch", source)
        self.assertIn("fn complete_disposable_context_creation(", source)
        self.assertIn("DisposableContextCreateDisposition::Accepted", source)
        self.assertIn("DisposableContextCreateDisposition::Rejected", source)
        self.assertIn("CreateFailedClean", source)
        self.assertIn("CreateFailedUncertain", source)
        self.assertIn("PartialCreationIsolation", source)
        self.assertIn("DuplicateAdapterHandle", source)
        self.assertIn("UnsettledAdapterHandle", source)
        self.assertIn("UnprovenDestruction", source)
        self.assertIn("RecoveryRequiredOwnedHandle", source)
        self.assertIn("TransportLossOwnedHandle", source)
        self.assertIn("create_disposable_context_with_port", source)
        self.assertIn("advance_context_epoch", source)
        self.assertIn("record_transport_loss", source)
        self.assertIn("transport_is_lost", source)
        self.assertIn("recovery_evidence", source)
        self.assertIn("AuthorizedContextOperationError::BrowserSession", source)
        self.assertIn("AuthorizedContextOperationError::Adapter", source)
        self.assertIn("#[must_use =", source)
        self.assertIn("impl<P> Drop for BoundBrowserSession<P>", source)
        self.assertIn("<redacted>", source)
        self.assertIn("user-context", source)
        self.assertIn("Reconstructing cleanup authority", source)
        self.assertIn("sequential_incarnation_reuse_rejects_stale_authority", source)
        self.assertIn("pub fn finish(&mut self)", source)
        self.assertNotIn("pub fn finish(mut self)", source)

        authority_impl = source.split("impl PresentationMutationAuthority", 1)[1].split(
            "enum OwnedContextState", 1
        )[0]
        self.assertNotIn("pub fn new", authority_impl)
        self.assertNotIn("pub const fn new", authority_impl)

        create_request_impl = source.split("impl DisposableContextCreateRequest", 1)[1].split(
            "pub enum DisposableContextCreateDisposition", 1
        )[0]
        completion_impl = source.split("impl DisposableContextCreateCompletion", 1)[1].split(
            "pub enum DisposableContextCreateCompletionError", 1
        )[0]
        destroy_request_impl = source.split("impl DisposableContextDestroyRequest", 1)[1].split(
            "pub trait DisposableContextPort", 1
        )[0]
        operation_request_impl = source.split("impl<O> AuthorizedContextOperationRequest", 1)[1].split(
            "pub enum AuthorizedContextOperationError", 1
        )[0]
        for request_impl in (
            create_request_impl,
            completion_impl,
            destroy_request_impl,
            operation_request_impl,
        ):
            self.assertNotIn("pub fn new", request_impl)
            self.assertNotIn("pub const fn new", request_impl)

    def test_hostile_recovery_binding_and_operation_fixtures_remain_external(self) -> None:
        """Recovery, binding, transactions, operations, and abandonment execute externally."""

        destroy_hostile = (CRATE / "tests/destroy_failure_requires_recovery.rs").read_text(
            encoding="utf-8"
        )
        reincarnation_hostile = (CRATE / "tests/sequential_incarnation_reuse.rs").read_text(
            encoding="utf-8"
        )
        preflight_hostile = (CRATE / "tests/lifecycle_port_preflight_side_effect.rs").read_text(
            encoding="utf-8"
        )
        substitution_hostile = (CRATE / "tests/lifecycle_port_same_id_spoof.rs").read_text(
            encoding="utf-8"
        )
        transaction_hostile = (CRATE / "tests/creation_transaction_completion.rs").read_text(
            encoding="utf-8"
        )
        debug_hostile = (CRATE / "tests/bound_session_debug_redaction.rs").read_text(
            encoding="utf-8"
        )
        transport_hostile = (CRATE / "tests/transport_loss_recovery_evidence.rs").read_text(
            encoding="utf-8"
        )
        recovery_hostile = (CRATE / "tests/recovery_required_sibling_evidence.rs").read_text(
            encoding="utf-8"
        )
        operation_hostile = (CRATE / "tests/authorized_context_operation.rs").read_text(
            encoding="utf-8"
        )
        abandonment_hostile = (CRATE / "tests/bound_session_abandonment.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("destroy_failure_requires_recovery_before_any_new_authority", destroy_hostile)
        self.assertIn("BrowserSessionRecoveryEvidence::UnprovenDestruction", destroy_hostile)
        self.assertIn("assert!(bound.record_transport_loss());", destroy_hostile)
        self.assertIn("assert!(!bound.record_transport_loss());", destroy_hostile)
        self.assertNotIn("lifecycle_port()", destroy_hostile)

        self.assertIn("stale_authority_cannot_cross_sequential_session_incarnations", reincarnation_hostile)
        self.assertIn("assert_ne!(\n        bound_a.browser_session().incarnation(),", reincarnation_hostile)
        self.assertIn("assert!(destroy_b.borrow().is_empty());", reincarnation_hostile)
        self.assertNotIn("lifecycle_port()", reincarnation_hostile)

        self.assertIn("lifecycle_binding_invokes_no_adapter_callback_before_authorized_create", preflight_hostile)
        self.assertIn("identity_callbacks", preflight_hostile)
        self.assertNotIn("bound.lifecycle_port()", preflight_hostile)

        self.assertIn("distinct_adapter_cannot_be_substituted_for_create_after_binding", substitution_hostile)
        self.assertIn("distinct_adapter_cannot_be_substituted_for_destroy_after_binding", substitution_hostile)
        self.assertNotIn("bound.lifecycle_port()", substitution_hostile)

        self.assertIn("accepted_and_rejected_create_candidates_are_correlated_by_exact_attempt", transaction_hostile)
        self.assertIn("request.attempt_epoch().value()", transaction_hostile)
        self.assertIn("DisposableContextCreateDisposition::Accepted", transaction_hostile)
        self.assertIn("DisposableContextCreateDisposition::Rejected", transaction_hostile)
        self.assertIn("assert!(ledger.pending.is_empty());", transaction_hostile)

        self.assertIn("bound_session_debug_never_executes_or_exposes_adapter_debug", debug_hostile)
        self.assertIn("adapter-secret-sentinel", debug_hostile)
        self.assertIn("debug_callbacks.get(),\n        0", debug_hostile)

        self.assertIn("transport_loss_preserves_exact_owned_handle_as_non_authorizing_recovery_evidence", transport_hostile)
        self.assertIn("transport-user-context-501", transport_hostile)
        self.assertIn("recovery_evidence().len(),\n        1", transport_hostile)

        self.assertIn("recovery_required_projects_exact_handles_for_indirectly_uncertain_siblings", recovery_hostile)
        self.assertIn("BrowserSessionRecoveryEvidence::RecoveryRequiredOwnedHandle", recovery_hostile)
        self.assertIn("indirectly invalidated sibling", recovery_hostile)

        self.assertIn("authorized_operation_uses_exact_bound_port_and_rejects_stale_authority_before_io", operation_hostile)
        self.assertIn("AuthorizedContextOperationError::BrowserSession", operation_hostile)
        self.assertIn("AuthorizedContextOperationError::Adapter", operation_hostile)
        self.assertIn("stale authority must fail before the bound adapter", operation_hostile)

        self.assertIn("dropping_unresolved_bound_session_is_observable_without_implicit_browser_io", abandonment_hostile)
        self.assertIn("abandoned_bound_session_count", abandonment_hostile)
        self.assertIn("Drop must never pretend synchronous browser cleanup succeeded", abandonment_hostile)
        self.assertIn("failed_finish_retains_same_bound_owner_for_cleanup_and_retry", abandonment_hostile)
        self.assertIn("same bound lifecycle owner must remain available for cleanup", abandonment_hostile)
        self.assertIn("proven_destruction_can_finish_without_abandonment_path", abandonment_hostile)
        self.assertIn("bound.finish()", abandonment_hostile)

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
        for token in (
            "Status: Proposed",
            "WD-webdriver-bidi-20260909",
            "RecoveryRequired",
            "RecoveryRequiredOwnedHandle",
            "BrowserSessionIncarnation",
            "BrowserSessionRecoveryEvidence",
            "DisposableContextCreateRequest",
            "DisposableContextCreateCompletion",
            "DisposableContextDestroyRequest",
            "BoundBrowserSession",
            "linear lifecycle-port binding",
            "no public raw port accessor",
            "per-create transaction",
            "DisposableContextCreateError",
            "DisposableContextDestroyError",
            "CreateFailedClean",
            "CreateFailedUncertain",
            "transport liveness",
            "sequential",
            "unproven destruction",
            "AuthorizedContextOperationPort",
            "TransportLossOwnedHandle",
            "abandoned_bound_session_count",
            "failed `finish()`",
            "Drop",
            "finish()",
        ):
            self.assertIn(token, adr)

        for token in (
            "IMPLEMENTED_ON_ACTIVE_PR",
            "BoundBrowserSession",
            "DisposableContextCreateCompletion",
            "per-create transaction",
            "no public raw port accessor",
            "RecoveryRequired",
            "RecoveryRequiredOwnedHandle",
            "BrowserSessionIncarnation",
            "lossless recovery evidence",
            "transport liveness",
            "Sequential ABA",
            "command ACK",
            "AuthorizedContextOperationPort",
            "TransportLossOwnedHandle",
            "abandoned_bound_session_count",
            "durable crash/process-restart recovery",
        ):
            self.assertIn(token, trace)

        for token in (
            "PresentationMutationAuthority",
            "BoundBrowserSession",
            "DisposableContextCreateCompletion",
            "BrowserSessionIncarnation",
            "RecoveryRequired",
            "RecoveryRequiredOwnedHandle",
            "transport_lost",
            "DisposableContextDestroyError / cleanup unproven",
            "AuthorizedContextOperationRequest",
            "AuthorizedContextOperationError::BrowserSession",
            "TransportLossOwnedHandle",
            "abandoned_bound_session_count",
            "finish()",
        ):
            self.assertIn(token, uml)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", trace)


if __name__ == "__main__":
    unittest.main()
