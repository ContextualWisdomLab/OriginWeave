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

    def test_windows_secure_open_contract_names_non_reparse_and_identity_primitives(self):
        """Windows parity must name the concrete non-reparse and handle-identity primitives."""
        text = MIGRATION.read_text(encoding="utf-8")
        required = (
            "FILE_FLAG_OPEN_REPARSE_POINT",
            "GetFileInformationByHandleEx",
            "FILE_ID_INFO",
            "VolumeSerialNumber",
            "FileId",
        )
        for token in required:
            self.assertIn(token, text)

    def test_migration_preserves_explicit_json_nesting_budget(self):
        """The Rust-owner plan must retain the product-owned pre-materialization depth limit."""
        text = MIGRATION.read_text(encoding="utf-8")
        required = (
            "MAX_SPDX_JSON_NESTING_DEPTH",
            "256",
            "too_deep_json_structure",
            "current nesting depth",
        )
        for token in required:
            self.assertIn(token, text)


if __name__ == "__main__":
    unittest.main()
