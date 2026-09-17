"""Repository contracts for proof-bearing Browser Session recovery settlement."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/originweave-browser-session"
RECOVERY = CRATE / "src/recovery.rs"
BROWSER_SESSION = CRATE / "src/browser_session.rs"
HOSTILE = CRATE / "tests/recovery_exact_fact_settlement.rs"
ADR = ROOT / "docs/adr/0116-browser-session-recovery-custody-and-hot-ownership.md"
TRACE = ROOT / "docs/traceability/browser-session-lifecycle-authority.md"
UML = ROOT / "docs/uml/browser-session-lifecycle-authority.md"
DOCTORING = ROOT / "docs/doctoring/browser-session-recovery-settlement.md"


def _struct_body(source: str, declaration: str) -> str:
    """Return one simple Rust struct body used by the authority-surface contract."""

    return source.split(declaration, 1)[1].split("\n}", 1)[0]


class BrowserSessionRecoverySettlementContractTests(unittest.TestCase):
    """Keep recovery settlement exact-fact-bound and non-caller-constructible."""

    def test_settlement_surface_is_explicit_and_opaque(self) -> None:
        """New construction paths must not bypass Browser Session-issued fact custody."""

        recovery_source = RECOVERY.read_text(encoding="utf-8")
        browser_source = BROWSER_SESSION.read_text(encoding="utf-8")

        for symbol in (
            "pub struct RecoveryFact",
            "pub struct RecoverySettlementRequest",
            "pub trait RecoverySettlementPort",
            "pub enum RecoverySettlementError",
            "pub fn recovery_fact",
            "pub fn create_attempt_recovery_fact",
            "pub fn settle_recovery_fact",
        ):
            self.assertIn(symbol, recovery_source)

        for declaration in (
            "pub struct RecoveryFact {",
            "pub struct RecoverySettlementRequest<P> {",
        ):
            body = _struct_body(recovery_source, declaration)
            self.assertNotRegex(
                body,
                r"(?m)^\s*pub(?:\([^)]*\))?\s+",
                f"{declaration} fields must stay private",
            )

        self.assertNotRegex(
            recovery_source,
            r"#\[derive\([^\]]*\bDefault\b[^\]]*\)\]\s*pub struct RecoveryFact",
        )
        for constructor_pattern in (
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::default::)?Default\s+for\s+RecoveryFact",
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::convert::)?From<[^{}]+?>\s+for\s+RecoveryFact",
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::convert::)?TryFrom<[^{}]+?>\s+for\s+RecoveryFact",
        ):
            self.assertNotRegex(recovery_source, constructor_pattern)

        self.assertIn("pub(crate) fn dispatch_recovery_operation", browser_source)
        self.assertNotIn("pub fn dispatch_recovery_operation", browser_source)
        self.assertNotIn("pub fn port", recovery_source)
        self.assertNotIn("pub const fn port", recovery_source)

    def test_settlement_order_pins_pre_io_validation_and_post_proof_commit(self) -> None:
        """Fact validation must precede proof I/O, which must precede aggregate mutation."""

        source = RECOVERY.read_text(encoding="utf-8")
        selector = source.split("fn select_recovery_fact", 1)[1].split("\n    }\n}", 1)[0]
        method = source.split("pub fn settle_recovery_fact", 1)[1].split("\n    }\n}", 1)[0]

        selector_authority = selector.index("RecoveryFactValidationError::AuthorityMismatch")
        selector_revision = selector.index("fact.revision != self.revision")
        selector_exact_fact = selector.index(".get(fact.index)")
        selection = method.index(".select_recovery_fact(fact)")
        verifier = method.index("verify_recovery_settlement")
        retirement = method.index("settle_recovery_evidence_at")
        revision_commit = method.index("self.revision = next_revision")

        self.assertLess(selector_authority, selector_revision)
        self.assertLess(selector_revision, selector_exact_fact)
        self.assertLess(selection, verifier)
        self.assertLess(verifier, retirement)
        self.assertLess(retirement, revision_commit)
        self.assertIn("RecoveryFactLedger::CreateAttempt", selector)
        self.assertIn("RecoveryFactLedger::CreateAttempt", method)
        self.assertIn("settle_create_attempt_recovery_evidence_at", method)

    def test_hostile_fixture_pins_replay_sibling_foreign_and_dual_ledger_cases(self) -> None:
        """The external fixture must keep realistic settlement abuse cases executable."""

        hostile = HOSTILE.read_text(encoding="utf-8")
        for token in (
            "RecoverySettlementPort",
            "RecoverySettlementRequest",
            "settle_recovery_fact",
            "RecoverySettlementError::StaleFact",
            "RecoverySettlementError::AuthorityMismatch",
            "stale replay must fail before adapter proof verification",
            "settling one fact invalidates previously issued sibling handles",
            "foreign session/incarnation fact must fail before adapter proof verification",
            "create_attempt_recovery_fact",
            "BrowserSessionState::Ended",
            "must never restore ordinary browser authority",
        ):
            self.assertIn(token, hostile)

    def test_architecture_docs_name_the_current_settlement_boundary(self) -> None:
        """ADR/trace/UML/doctoring must describe proof-bearing settlement, not only dispatch."""

        documents = [
            ADR.read_text(encoding="utf-8"),
            TRACE.read_text(encoding="utf-8"),
            UML.read_text(encoding="utf-8"),
            DOCTORING.read_text(encoding="utf-8"),
        ]
        for document in documents:
            for token in (
                "RecoveryFact",
                "RecoverySettlementPort",
                "RecoverySettlementRequest",
                "settle_recovery_fact",
                "#316",
            ):
                self.assertIn(token, document)
            self.assertIn("revision", document.lower())
            self.assertIn("proof", document.lower())


if __name__ == "__main__":
    unittest.main()
