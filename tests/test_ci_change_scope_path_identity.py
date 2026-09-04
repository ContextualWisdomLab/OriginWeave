"""Security regressions for Git raw-diff rename/copy path identity."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import classify_changes, parse_nul_raw_changes


def _raw_record(status: str, source_path: str, destination_path: str) -> bytes:
    """Build one canonical scored rename/copy record with caller-controlled paths."""

    return (
        f":100644 100644 1111111 2222222 {status}\0"
        f"{source_path}\0{destination_path}\0"
    ).encode()


class CiChangeScopePathIdentityTests(unittest.TestCase):
    """Reject rename/copy evidence that cannot describe two distinct repository entries."""

    def test_distinct_docs_rename_remains_lightweight_eligible(self) -> None:
        """A normal docs-to-doc rename preserves both distinct path identities."""

        changes = parse_nul_raw_changes(
            _raw_record("R100", "docs/old.md", "docs/new.md")
        )
        self.assertEqual(classify_changes(changes), (True, False))

    def test_rename_or_copy_with_identical_paths_is_rejected(self) -> None:
        """Malformed R/C records must not authorize the prose-only CI lane."""

        for status in ("R100", "C100"):
            with self.subTest(status=status):
                with self.assertRaisesRegex(ValueError, "path"):
                    parse_nul_raw_changes(
                        _raw_record(status, "docs/PRD.md", "docs/PRD.md")
                    )


if __name__ == "__main__":
    unittest.main()
