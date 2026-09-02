import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
MIGRATION = ROOT / "docs/traceability/release-sbom-owner-migration.md"


class ReleaseSbomOwnerMigrationContractTests(unittest.TestCase):
    """Keep the pre-GA release/SBOM owner repair explicit and non-duplicative."""

    def test_release_sbom_migration_keeps_one_canonical_owner(self):
        """The repair plan must converge SBOM policy and verification on one Release owner."""
        self.assertTrue(MIGRATION.is_file())
        text = MIGRATION.read_text(encoding="utf-8")
        required = (
            "`originweave-release`",
            "`ReleaseManifest`",
            "`originweave-core`",
            "`scripts/release/validate_spdx_jsonld.py`",
            "thin conformance/fixture harness",
            "Rust-owned",
            "no compatibility shim",
            "no duplicated release API",
            "#221",
            "#240",
        )
        for token in required:
            self.assertIn(token, text)
        self.assertIn("Proposed", text)
        self.assertIn("not shipped truth", text)


if __name__ == "__main__":
    unittest.main()
