"""Regression contract for the canonical MV3 supported-capability evidence matrix."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCTORING = ROOT / "docs" / "doctoring" / "mv3-compatibility.md"
MATURITY = ROOT / "docs" / "evidence" / "2026-08-10-active-pr-maturity.md"


class ManifestV3SupportedCapabilityMatrixContractTests(unittest.TestCase):
    """Keep compatibility claims executable, maturity-scoped, and authority-safe."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.doctoring = DOCTORING.read_text(encoding="utf-8")
        cls.maturity = MATURITY.read_text(encoding="utf-8")

    def test_matrix_separates_protected_active_planned_and_out_of_scope(self) -> None:
        """The matrix must never collapse active evidence into protected-main support."""

        for marker in (
            "## Supported-capability evidence matrix",
            "**PROTECTED_MAIN**",
            "**ACTIVE_PR #43**",
            "**ACTIVE_PR #56**",
            "**ACTIVE_PR #59**",
            "**ACTIVE_PR #60**",
            "**PLANNED**",
            "**PLANNED / SECURITY-GATED**",
            "**OUT_OF_SCOPE FOR COMPATIBILITY CLAIM**",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.doctoring)

    def test_update_migration_is_not_documented_as_restart_only(self) -> None:
        """Update compatibility requires a version transition plus migrated state."""

        for marker in (
            "Restart persistence and extension update migration are separate compatibility claims",
            "`1.0.0` to `1.0.1`",
            "schema marker to migrate from version 1 to version 2",
            "checked-in fixture is not rewritten",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.doctoring)

        row = next(
            line for line in self.maturity.splitlines() if line.startswith("| #60 |")
        )
        self.assertIn("**IMPLEMENTED_ON_ACTIVE_PR**", row)
        self.assertIn("e696e19c9eaf3dedb104a5de4bdbd7970abf90d4", row)
        self.assertIn("CI run `31433968874`", row)
        self.assertIn("Manifest V3 Compatibility run `31433968931`", row)
        self.assertNotIn("IMPLEMENTED_ON_PROTECTED_MAIN", row)

    def test_compatibility_never_grants_agent_authority(self) -> None:
        """Chrome API success must remain separate from OriginWeave Agent grants."""

        for marker in (
            "Chrome API permission does not become Agent capability",
            "no Agent bookmark capability",
            "no Agent history capability",
            "does not claim Chrome Web Store/enterprise update semantics or Agent authority",
        ):
            with self.subTest(marker=marker):
                self.assertTrue(marker in self.doctoring or marker in self.maturity)


if __name__ == "__main__":
    unittest.main()
