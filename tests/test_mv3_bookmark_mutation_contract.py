"""Fail-first contract for real Manifest V3 bookmark mutation compatibility."""

from __future__ import annotations

import importlib.util
import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


def _load_runner_module():
    """Load the compatibility runner without invoking its command-line entry point."""

    spec = importlib.util.spec_from_file_location("originweave_mv3_runner", RUNNER)
    if spec is None or spec.loader is None:
        raise AssertionError("unable to load the MV3 compatibility runner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ManifestV3BookmarkMutationContractTests(unittest.TestCase):
    """Require one bounded create/read/delete bookmark lifecycle in real Chromium."""

    def test_fixture_declares_bookmarks_permission(self) -> None:
        """The controlled extension must explicitly request bookmark authority."""

        manifest = json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))
        self.assertIn("bookmarks", manifest["permissions"])

    def test_service_worker_executes_bounded_bookmark_mutation_lifecycle(self) -> None:
        """Compatibility evidence must require create/read/delete, not only tree reads."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            "exerciseBookmarkMutation",
            "chrome.bookmarks.create",
            "chrome.bookmarks.get",
            "chrome.bookmarks.remove",
            '"OriginWeave MV3 compatibility bookmark"',
            "bookmarkMutationReady",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)

    def test_bookmark_mutation_is_bound_to_controlled_fixture_url_and_cleanup(self) -> None:
        """The fixture must not mutate bookmarks for an arbitrary sender or leave residue."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            'parsed.protocol !== "http:"',
            'parsed.hostname !== "127.0.0.1"',
            'parsed.pathname !== "/page.html"',
            "finally",
            "chrome.bookmarks.remove",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertNotIn("_error.message", worker)
        self.assertNotIn("String(_error)", worker)

    def test_bookmark_failures_emit_only_bounded_stage_diagnostics(self) -> None:
        """Every bookmark stage must return a reviewed token without raw browser values."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            "bookmark-source-rejected",
            "bookmark-create-rejected",
            "bookmark-get-missing",
            "bookmark-id-mismatch",
            "bookmark-title-mismatch",
            "bookmark-url-mismatch",
            "bookmark-remove-rejected",
            "bookmark-complete-ready",
            "bookmarksDiagnostic",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertNotIn("_error.message", worker)
        self.assertNotIn("String(_error)", worker)

    def test_runner_preserves_only_reviewed_bookmark_diagnostic_tokens(self) -> None:
        """Trial evidence must reduce arbitrary bookmark diagnostics to `unexpected`."""

        runner = _load_runner_module()
        approved = {
            "bookmark-source-rejected",
            "bookmark-create-rejected",
            "bookmark-get-missing",
            "bookmark-id-mismatch",
            "bookmark-title-mismatch",
            "bookmark-url-mismatch",
            "bookmark-remove-rejected",
            "bookmark-complete-ready",
            "bookmark-not-evaluated",
        }
        self.assertIn("bookmarksDiagnostic", runner.SURFACE_EVIDENCE_KEYS)
        self.assertEqual(runner.BOOKMARK_DIAGNOSTIC_VALUES, frozenset(approved))
        for token in approved:
            with self.subTest(token=token):
                self.assertEqual(
                    runner._safe_surface_value("bookmarksDiagnostic", token), token
                )

        for raw in (
            "OriginWeave MV3 compatibility bookmark",
            "Error: secret bookmark failure",
        ):
            with self.subTest(raw=raw):
                error = runner.CompatibilitySurfaceError(
                    {"bookmarks": "missing", "bookmarksDiagnostic": raw}
                )
                evidence = runner._failure_evidence(error)
                self.assertEqual(
                    evidence["observed"]["bookmarksDiagnostic"], "unexpected"
                )
                self.assertNotIn(raw, repr(evidence))


if __name__ == "__main__":
    unittest.main()
