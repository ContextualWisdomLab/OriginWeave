"""Security regressions for Git raw-diff mode/object identity coupling."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import parse_nul_raw_changes


class CiChangeScopeObjectIdTests(unittest.TestCase):
    """Reject raw records that could not come from the intended two-tree Git diff."""

    def test_addition_rejects_nonzero_source_object_id(self) -> None:
        """An absent addition preimage must carry Git's all-zero object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                b":000000 100644 1111111 2222222 A\0README.md\0"
            )

    def test_deletion_rejects_nonzero_destination_object_id(self) -> None:
        """An absent deletion postimage must carry Git's all-zero object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                b":100644 000000 1111111 2222222 D\0docs/old.md\0"
            )

    def test_materialized_side_rejects_zero_object_id(self) -> None:
        """A present two-tree side must not use the absence sentinel object identity."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                b":100644 100644 0000000 2222222 M\0docs/PRD.md\0"
            )

    def test_same_mode_modification_rejects_identical_object_ids(self) -> None:
        """A same-mode two-tree modification must identify two different blobs."""

        with self.assertRaisesRegex(ValueError, "object id"):
            parse_nul_raw_changes(
                b":100644 100644 1111111 1111111 M\0docs/PRD.md\0"
            )


if __name__ == "__main__":
    unittest.main()
