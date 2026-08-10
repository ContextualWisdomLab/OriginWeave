"""Fail-first contract for bounded Manifest V3 history mutation compatibility."""

from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"


class ManifestV3HistoryMutationContractTests(unittest.TestCase):
    """Require one controlled add/read/delete history lifecycle in real Chromium."""

    def test_fixture_declares_history_permission(self) -> None:
        """The controlled extension must explicitly request history authority."""

        manifest = json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))
        self.assertIn("history", manifest["permissions"])

    def test_service_worker_executes_bounded_history_mutation_lifecycle(self) -> None:
        """Compatibility evidence must require add/read/delete, not search alone."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            "exerciseHistoryMutation",
            "chrome.history.addUrl",
            "chrome.history.search",
            "chrome.history.deleteUrl",
            "historyMutationReady",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)

    def test_history_mutation_is_bound_to_controlled_fixture_url_and_cleanup(self) -> None:
        """The fixture must not mutate arbitrary history and must remove test state."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            'parsed.protocol !== "http:"',
            'parsed.hostname !== "127.0.0.1"',
            'parsed.pathname !== "/page.html"',
            "finally",
            "chrome.history.deleteUrl",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertNotIn("_error.message", worker)
        self.assertNotIn("String(_error)", worker)


if __name__ == "__main__":
    unittest.main()
