"""Tests for the exact LLVM coverage gate."""

from __future__ import annotations

import contextlib
import io
import json
import pathlib
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "ci"))

import verify_coverage  # noqa: E402


def payload(count: int = 3, covered: int = 3) -> dict[str, object]:
    """Create a minimal LLVM summary payload for tests."""

    return {
        "data": [
            {
                "totals": {
                    metric: {"count": count, "covered": covered}
                    for metric in verify_coverage.REQUIRED_METRICS
                }
            }
        ]
    }


class CoverageVerifierTests(unittest.TestCase):
    """Exercise every fail-closed coverage-verification path."""

    def test_exact_coverage_has_no_uncovered_metrics(self) -> None:
        """Equal covered and total counts satisfy the contract."""

        self.assertEqual(verify_coverage.uncovered_metrics(payload()), {})

    def test_partial_coverage_reports_each_incomplete_metric(self) -> None:
        """Every incomplete metric must retain its covered and total counts."""

        self.assertEqual(
            verify_coverage.uncovered_metrics(payload(3, 2)),
            {metric: (2, 3) for metric in verify_coverage.REQUIRED_METRICS},
        )

    def test_uncovered_region_locations_report_only_real_zero_count_region_entries(self) -> None:
        """LLVM segment diagnostics identify uncovered production coordinates precisely."""

        candidate = payload(3, 2)
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/example.rs",
                "segments": [
                    [10, 2, 0, True, True, False],
                    [10, 8, 0, True, False, False],
                    [11, 1, 0, True, True, True],
                    [12, 3, 4, True, True, False],
                    [13, 5, 0, False, True, False],
                    [10, 2, 0, True, True, False],
                ],
            },
            {
                "filename": "src/second.rs",
                "segments": [[3, 7, 0, True, True, False]],
            },
        ]
        self.assertEqual(
            verify_coverage.uncovered_region_locations(candidate),
            ["src/example.rs:10:2", "src/second.rs:3:7"],
        )

    def test_uncovered_region_locations_fall_back_to_function_regions(self) -> None:
        """LLVM function-region detail locates misses absent from segment entries."""

        candidate = payload(3, 2)
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {"filename": "src/example.rs", "segments": []}
        ]
        candidate["data"][0]["functions"] = [  # type: ignore[index]
            {
                "name": "example",
                "filenames": ["src/example.rs"],
                "regions": [
                    [42, 9, 42, 15, 0, 0, 0, 0],
                    [43, 1, 43, 8, 2, 0, 0, 0],
                    [44, 3, 44, 7, 0, 0, 0, 3],
                ],
            }
        ]
        self.assertEqual(
            verify_coverage.uncovered_region_locations(candidate),
            ["src/example.rs:42:9"],
        )

    def test_function_region_fallback_is_scoped_to_deficient_files(self) -> None:
        """Noisy zero-count instantiations from fully covered files are excluded."""

        candidate = payload(3, 2)
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/complete.rs",
                "segments": [],
                "summary": {"regions": {"count": 4, "covered": 4}},
            },
            {
                "filename": "src/partial.rs",
                "segments": [],
                "summary": {"regions": {"count": 7, "covered": 6}},
            },
        ]
        candidate["data"][0]["functions"] = [  # type: ignore[index]
            {
                "name": "complete_instantiation",
                "filenames": ["src/complete.rs"],
                "regions": [[10, 2, 10, 8, 0, 0, 0, 0]],
            },
            {
                "name": "partial_instantiation",
                "filenames": ["src/partial.rs"],
                "regions": [[42, 9, 42, 15, 0, 0, 0, 0]],
            },
        ]
        self.assertEqual(
            verify_coverage.uncovered_region_locations(candidate),
            ["src/partial.rs:42:9"],
        )

    def test_function_region_fallback_merges_identical_instantiations(self) -> None:
        """A covered monomorphization satisfies the same aggregate source region."""

        candidate = payload(3, 2)
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/partial.rs",
                "segments": [],
                "summary": {"regions": {"count": 2, "covered": 1}},
            }
        ]
        candidate["data"][0]["functions"] = [  # type: ignore[index]
            {
                "name": "generic_zero",
                "filenames": ["src/partial.rs"],
                "regions": [[42, 9, 42, 15, 0, 0, 0, 0]],
            },
            {
                "name": "generic_covered",
                "filenames": ["src/partial.rs"],
                "regions": [[42, 9, 42, 15, 3, 0, 0, 0]],
            },
            {
                "name": "actually_uncovered",
                "filenames": ["src/partial.rs"],
                "regions": [[50, 2, 50, 8, 0, 0, 0, 0]],
            },
        ]
        self.assertEqual(
            verify_coverage.uncovered_region_locations(candidate),
            ["src/partial.rs:50:2"],
        )

    def test_uncovered_file_region_summaries_report_only_deficient_files(self) -> None:
        """File summaries identify which source contributes aggregate region debt."""

        candidate = payload(3, 2)
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/complete.rs",
                "summary": {"regions": {"count": 4, "covered": 4}},
            },
            {
                "filename": "src/partial.rs",
                "summary": {"regions": {"count": 7, "covered": 6}},
            },
            {"filename": "src/no-summary.rs"},
        ]
        self.assertEqual(
            verify_coverage.uncovered_file_region_summaries(candidate),
            ["src/partial.rs=6/7"],
        )

    def test_uncovered_region_locations_are_best_effort_for_missing_file_detail(self) -> None:
        """Summary-only or malformed file detail never weakens aggregate enforcement."""

        self.assertEqual(verify_coverage.uncovered_region_locations(payload(3, 2)), [])
        candidate = payload(3, 2)
        candidate["data"][0]["files"] = "not-a-list"  # type: ignore[index]
        self.assertEqual(verify_coverage.uncovered_region_locations(candidate), [])

    def test_malformed_payloads_fail_closed(self) -> None:
        """Missing, ambiguous, and impossible summaries are rejected."""

        malformed = [
            {},
            {"data": "not-a-list"},
            {"data": []},
            {"data": [{}, {}]},
            {"data": ["not-an-object"]},
            {"data": [{}]},
            {"data": [{"totals": "not-an-object"}]},
            {"data": [{"totals": {}}]},
        ]
        for candidate in malformed:
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                verify_coverage.uncovered_metrics(candidate)

    def test_non_integer_and_impossible_counts_fail_closed(self) -> None:
        """Boolean, negative, and over-covered counters are invalid."""

        for count, covered in [(True, 1), (1, False), (-1, 0), (1, -1), (1, 2)]:
            candidate = payload()
            totals = candidate["data"][0]["totals"]  # type: ignore[index]
            totals["branches"] = {"count": count, "covered": covered}  # type: ignore[index]
            with self.subTest(count=count, covered=covered), self.assertRaises(ValueError):
                verify_coverage.uncovered_metrics(candidate)

    def test_verify_file_accepts_exact_and_rejects_partial_coverage(self) -> None:
        """File verification distinguishes complete and partial reports."""

        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "coverage.json"
            path.write_text(json.dumps(payload()), encoding="utf-8")
            verify_coverage.verify_file(path)

            path.write_text(json.dumps(payload(3, 2)), encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "below 100%"):
                verify_coverage.verify_file(path)

    def test_verify_file_includes_precise_uncovered_region_coordinates(self) -> None:
        """A region-only failure points directly at uncovered source coordinates."""

        candidate = payload()
        totals = candidate["data"][0]["totals"]  # type: ignore[index]
        totals["regions"] = {"count": 2, "covered": 1}  # type: ignore[index]
        candidate["data"][0]["files"] = [  # type: ignore[index]
            {
                "filename": "src/example.rs",
                "segments": [[42, 9, 0, True, True, False]],
            }
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "coverage.json"
            path.write_text(json.dumps(candidate), encoding="utf-8")
            with self.assertRaisesRegex(
                RuntimeError,
                r"regions=1/2; uncovered regions: src/example\.rs:42:9",
            ):
                verify_coverage.verify_file(path)

    def test_main_reports_usage_success_and_input_failures(self) -> None:
        """The command-line interface returns stable process statuses."""

        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            self.assertEqual(verify_coverage.main([]), 2)
        self.assertIn("usage", stderr.getvalue())

        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "coverage.json"
            path.write_text(json.dumps(payload()), encoding="utf-8")
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                self.assertEqual(verify_coverage.main([str(path)]), 0)
            self.assertIn("100% covered", stdout.getvalue())

            path.write_text("not-json", encoding="utf-8")
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                self.assertEqual(verify_coverage.main([str(path)]), 1)
            self.assertTrue(stderr.getvalue())

            missing = pathlib.Path(directory) / "missing.json"
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                self.assertEqual(verify_coverage.main([str(missing)]), 1)
            self.assertTrue(stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
