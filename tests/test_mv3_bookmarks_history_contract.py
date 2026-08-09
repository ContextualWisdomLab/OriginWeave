"""Fail-first contract for real Manifest V3 bookmarks and history compatibility."""

from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3BookmarksHistoryContractTests(unittest.TestCase):
    """Require two additional read-only Chrome API surfaces in the real browser lane."""

    def test_fixture_declares_bookmarks_and_history_permissions(self) -> None:
        """The controlled fixture must request the APIs it exercises."""

        manifest = json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))
        for permission in ("bookmarks", "history"):
            with self.subTest(permission=permission):
                self.assertIn(permission, manifest["permissions"])

    def test_service_worker_exercises_read_only_bookmarks_and_history_apis(self) -> None:
        """Compatibility evidence must come from executing the real extension APIs."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in ("chrome.bookmarks.getTree", "chrome.history.search"):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)

    def test_content_script_and_runner_require_both_surfaces(self) -> None:
        """Each browser pass and repeatability trial must prove both APIs succeeded."""

        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")
        for expected in ("originweaveBookmarks", "originweaveHistory"):
            with self.subTest(expected=expected):
                self.assertIn(expected, content)
        for surface in ("bookmarks", "history"):
            with self.subTest(surface=surface):
                self.assertIn(f'"{surface}"', runner)


if __name__ == "__main__":
    unittest.main()
