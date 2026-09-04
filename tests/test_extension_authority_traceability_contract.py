"""Regression contract for extension-to-Agent security traceability."""

from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
TRACEABILITY = (
    REPOSITORY_ROOT / "docs" / "traceability" / "extension-authority-security.md"
)


class ExtensionAuthorityTraceabilityContractTests(unittest.TestCase):
    """Keep compatibility, Agent authority, and secret authority as separate claims."""

    def test_extension_security_dossier_preserves_maturity_boundaries(self) -> None:
        """Active security proofs must never be promoted to protected-main shipped truth."""
        text = TRACEABILITY.read_text(encoding="utf-8")
        semantic_text = text.replace("**", "")

        self.assertIn("DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL", semantic_text)
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", text)
        self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", text)
        self.assertIn("PR #62", text)
        self.assertIn("PR #63", text)
        self.assertIn("Proposed ADR 0013", text)
        self.assertIn("SecretBrokerRequired", text)
        self.assertIn("UnexpectedSecretMaterial", text)
        self.assertIn("R3 approval", text)
        self.assertIn("does not close issue #27 or issue #10", semantic_text)

    def test_extension_proposal_authority_is_explicitly_non_transitive(self) -> None:
        """The dossier must forbid proposal permission from becoming broader Agent authority."""
        text = TRACEABILITY.read_text(encoding="utf-8")

        for boundary in (
            "-/> Agent capability",
            "-/> Agent readable/writable origin",
            "-/> trusted instruction source",
            "-/> secret-delivery authority",
            "-/> approval",
            "-/> protected-value resolution",
        ):
            with self.subTest(boundary=boundary):
                self.assertIn(boundary, text)


if __name__ == "__main__":
    unittest.main()
