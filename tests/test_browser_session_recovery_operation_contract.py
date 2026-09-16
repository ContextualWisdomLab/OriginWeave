"""Repository contracts for Browser Session recovery-only adapter custody."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/originweave-browser-session"


class BrowserSessionRecoveryOperationContractTests(unittest.TestCase):
    """Keep recovery I/O purpose-bounded to the exact consumed adapter."""

    def test_recovery_dispatch_stays_inside_native_owner_module(self) -> None:
        """Do not reopen raw adapter access to bridge recovery custody."""

        lib_source = (CRATE / "src/lib.rs").read_text(encoding="utf-8")
        browser_source = (CRATE / "src/browser_session.rs").read_text(encoding="utf-8")
        recovery_source = (CRATE / "src/recovery.rs").read_text(encoding="utf-8")

        self.assertIn("mod browser_session;", lib_source)
        self.assertNotIn('include!("browser_session.rs")', lib_source)
        self.assertIn("pub(crate) fn dispatch_recovery_operation", browser_source)
        self.assertNotIn("pub fn dispatch_recovery_operation", browser_source)

        for symbol in (
            "pub struct RecoveryContextOperationRequest",
            "pub trait RecoveryContextOperationPort",
            "pub enum RecoveryContextOperationError",
            "pub fn execute_recovery_context_operation",
        ):
            self.assertIn(symbol, recovery_source)

        request_impl = recovery_source.split(
            "impl<O> RecoveryContextOperationRequest", 1
        )[1].split("pub trait RecoveryContextOperationPort", 1)[0]
        self.assertNotIn("pub fn new", request_impl)
        self.assertNotIn("pub const fn new", request_impl)
        self.assertNotIn("pub fn browser_session(&self)", recovery_source)
        self.assertNotIn("pub fn port", recovery_source)
        self.assertNotIn("pub const fn port", recovery_source)

    def test_hostile_fixture_preserves_uncertainty_after_adapter_result(self) -> None:
        """Adapter success or failure must not silently become reconciliation proof."""

        hostile = (CRATE / "tests/recovery_same_adapter_operation.rs").read_text(
            encoding="utf-8"
        )
        for token in (
            "RecoveryContextOperationPort",
            "execute_recovery_context_operation",
            "request.browser_session()",
            "request.incarnation()",
            "request.state()",
            "request.recovery_evidence()",
            "request.create_attempt_recovery_evidence()",
            "RecoveryContextOperationError::Adapter",
        ):
            self.assertIn(token, hostile)
        self.assertIn("state_before", hostile)
        self.assertIn("evidence_before", hostile)

    def test_architecture_docs_describe_current_recovery_surface(self) -> None:
        """ADR, traceability, and UML must not describe the pre-operation wrapper."""

        adr = (
            ROOT / "docs/adr/0116-browser-session-recovery-custody-and-hot-ownership.md"
        ).read_text(encoding="utf-8")
        trace = (
            ROOT / "docs/traceability/browser-session-lifecycle-authority.md"
        ).read_text(encoding="utf-8")
        uml = (ROOT / "docs/uml/browser-session-lifecycle-authority.md").read_text(
            encoding="utf-8"
        )

        for document in (adr, trace, uml):
            self.assertIn("RecoveryContextOperationPort", document)
            self.assertIn("RecoveryContextOperationRequest", document)
            self.assertIn("same", document.lower())
        self.assertIn("pub(crate)", adr)
        self.assertIn("pub(crate)", trace)
        self.assertIn("success/failure does not clear Browser Session uncertainty", uml)
        self.assertIn("#316", adr)
        self.assertIn("#316", trace)
        self.assertIn("#316", uml)


if __name__ == "__main__":
    unittest.main()
