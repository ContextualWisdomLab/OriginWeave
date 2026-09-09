"""Regression tests for canonical Git raw-diff similarity score spelling."""

from __future__ import annotations

import unittest
from pathlib import Path

from scripts.ci.classify_ci_change_scope import classify_changes, parse_nul_raw_changes


def _raw_record(
    status: str,
    *,
    source_oid: str | None = None,
    destination_oid: str | None = None,
) -> bytes:
    """Build a docs-to-doc rename/copy record with caller-controlled identity evidence."""

    source_identity = source_oid or "1" * 40
    destination_identity = destination_oid or "2" * 40
    return (
        f":100644 100644 {source_identity} {destination_identity} {status}\0"
        "docs/old.md\0docs/new.md\0"
    ).encode()


class CiChangeScopeSimilarityScoreTests(unittest.TestCase):
    """Only score spellings and identities emitted by Git may authorize lightweight CI."""

    def test_canonical_three_digit_similarity_score_is_accepted(self) -> None:
        """Git raw output zero-pads scored statuses to three decimal digits."""

        changes = parse_nul_raw_changes(_raw_record("R074"))
        self.assertEqual(classify_changes(changes), (True, False))

    def test_noncanonical_similarity_score_widths_are_rejected(self) -> None:
        """Malformed producer text must fail closed instead of authorizing docs-only CI."""

        for status in ("R74", "R0074", "C5", "M5", "M0000"):
            with self.subTest(status=status):
                with self.assertRaisesRegex(ValueError, "status"):
                    parse_nul_raw_changes(_raw_record(status))

    def test_out_of_range_three_digit_score_is_rejected(self) -> None:
        """Three digits are necessary but a percentage above 100 is still invalid."""

        with self.assertRaisesRegex(ValueError, "status"):
            parse_nul_raw_changes(_raw_record("R101"))

    def test_perfect_similarity_requires_identical_blob_identity(self) -> None:
        """R100/C100 cannot describe different blob contents in exact two-tree evidence."""

        for status in ("R100", "C100"):
            with self.subTest(status=status):
                with self.assertRaisesRegex(ValueError, "similarity"):
                    parse_nul_raw_changes(_raw_record(status))

    def test_perfect_similarity_accepts_identical_blob_identity(self) -> None:
        """A content-identical rename or copy retains the same blob object identity."""

        identity = "1" * 40
        for status in ("R100", "C100"):
            with self.subTest(status=status):
                changes = parse_nul_raw_changes(
                    _raw_record(
                        status,
                        source_oid=identity,
                        destination_oid=identity,
                    )
                )
                self.assertEqual(classify_changes(changes), (True, False))

    def test_nonperfect_similarity_rejects_identical_blob_identity(self) -> None:
        """Identical blobs are 100% similar and cannot carry a lower R/C score."""

        identity = "1" * 40
        for status in ("R074", "C074"):
            with self.subTest(status=status):
                with self.assertRaisesRegex(ValueError, "similarity"):
                    parse_nul_raw_changes(
                        _raw_record(
                            status,
                            source_oid=identity,
                            destination_oid=identity,
                        )
                    )

    def test_nonperfect_similarity_accepts_distinct_blob_identity(self) -> None:
        """A non-perfect rename/copy similarity score requires distinct blob contents."""

        for status in ("R074", "C074"):
            with self.subTest(status=status):
                changes = parse_nul_raw_changes(_raw_record(status))
                self.assertEqual(classify_changes(changes), (True, False))

    def test_changelog_records_similarity_object_identity_contract(self) -> None:
        """The release record must name the security invariant added by this slice."""

        changelog = (Path(__file__).resolve().parents[1] / "CHANGELOG.md").read_text()
        self.assertIn("Bind Git rename/copy similarity to blob identity", changelog)


if __name__ == "__main__":
    unittest.main()
