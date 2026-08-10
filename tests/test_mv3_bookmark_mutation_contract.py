"""Fail-first contract for real Manifest V3 bookmark mutation compatibility."""

from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"


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


if __name__ == "__main__":
    unittest.main()
