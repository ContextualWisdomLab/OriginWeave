"""Regression tests for canonical Git raw-diff similarity score spelling."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import classify_changes, parse_nul_raw_changes


def _raw_record(status: str) -> bytes:
    """Build a docs-to-doc rename record with a caller-controlled score suffix."""

    return (
        f":100644 100644 1111111 2222222 {status}\0"
        "docs/old.md\0docs/new.md\0"
    ).encode()


class CiChangeScopeSimilarityScoreTests(unittest.TestCase):
    """Only score spellings emitted by Git may influence lightweight CI authority."""

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


if __name__ == "__main__":
    unittest.main()
