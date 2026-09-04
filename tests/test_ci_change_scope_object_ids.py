"""Security regressions for Git raw-diff mode/object identity coupling."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import parse_nul_raw_changes

ZERO = "0" * 40
ONE = "1" * 40
TWO = "2" * 40


class CiChangeScopeObjectIdTests(unittest.TestCase):
    """Reject raw records that could not come from the intended two-tree Git diff."""

    def test_abbreviated_object_ids_are_rejected(self) -> None:
        """Lightweight authority requires complete SHA-1 or SHA-256 identities."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                b":100644 100644 1111111 2222222 M\0docs/PRD.md\0"
            )

    def test_complete_sha256_object_ids_are_accepted(self) -> None:
        """Repositories using SHA-256 retain complete-object compatibility."""

        changes = parse_nul_raw_changes(
            (
                f":100644 100644 {'1' * 64} {'2' * 64} M\0"
                "docs/PRD.md\0"
            ).encode()
        )
        self.assertEqual(len(changes), 1)

    def test_addition_rejects_nonzero_source_object_id(self) -> None:
        """An absent addition preimage must carry Git's all-zero object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                f":000000 100644 {ONE} {TWO} A\0README.md\0".encode()
            )

    def test_deletion_rejects_nonzero_destination_object_id(self) -> None:
        """An absent deletion postimage must carry Git's all-zero object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                f":100644 000000 {ONE} {TWO} D\0docs/old.md\0".encode()
            )

    def test_materialized_side_rejects_zero_object_id(self) -> None:
        """A present two-tree side must not use the absence sentinel object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                f":100644 100644 {ZERO} {TWO} M\0docs/PRD.md\0".encode()
            )

    def test_same_mode_modification_rejects_identical_object_ids(self) -> None:
        """A same-mode two-tree modification must identify two different blobs."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                f":100644 100644 {ONE} {ONE} M\0docs/PRD.md\0".encode()
            )


if __name__ == "__main__":
    unittest.main()
