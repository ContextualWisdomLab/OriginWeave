"""Repository contracts for Browser Session recovery-only adapter custody."""

from __future__ import annotations

import pathlib
import re
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

        request_struct = recovery_source.split(
            "pub struct RecoveryContextOperationRequest<O> {", 1
        )[1].split("\n}", 1)[0]
        self.assertNotRegex(request_struct, r"(?m)^\s*pub(?:\([^)]*\))?\s+")

        inherent_impls = re.findall(
            r"impl(?:<[^\n{]+>)?\s+RecoveryContextOperationRequest(?:<[^\n{]+>)?\s*\{",
            recovery_source,
        )
        self.assertEqual(len(inherent_impls), 1)
        request_impl = recovery_source.split(
            "impl<O> RecoveryContextOperationRequest", 1
        )[1].split("pub trait RecoveryContextOperationPort", 1)[0]
        public_methods = re.findall(
            r"(?m)^\s*pub(?:\s+const)?\s+fn\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)",
            request_impl,
        )
        self.assertGreater(len(public_methods), 0)
        for method_name, parameters in public_methods:
            self.assertIn(
                "&self",
                parameters,
                f"{method_name} must remain an accessor, not a public construction path",
            )

        for constructor_pattern in (
            r"impl(?:<[^\n{]+>)?\s+(?:(?:core|std)::default::)?Default\s+for\s+RecoveryContextOperationRequest",
            r"impl(?:<[^\n{]+>)?\s+(?:(?:core|std)::convert::)?From<[^\n{]+>\s+for\s+RecoveryContextOperationRequest",
            r"impl(?:<[^\n{]+>)?\s+(?:(?:core|std)::convert::)?TryFrom<[^\n{]+>\s+for\s+RecoveryContextOperationRequest",
        ):
            self.assertNotRegex(recovery_source, constructor_pattern)

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
            "expected_recovery_evidence",
            "expected_create_attempt_recovery_evidence",
            "generic recovery adapter success is not itself destruction or reconciliation proof",
            "recovery operation dispatch must not erase unresolved ownership evidence",
            "recovery operation dispatch must not erase create-attempt provenance",
        ):
            self.assertIn(token, hostile)

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
