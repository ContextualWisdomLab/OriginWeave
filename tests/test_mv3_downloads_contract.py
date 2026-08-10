"""Fail-first contract for real Manifest V3 downloads compatibility."""

from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3DownloadsContractTests(unittest.TestCase):
    """Require the real Chrome downloads API in every pinned-browser trial."""

    def test_fixture_declares_downloads_permission_and_local_resource(self) -> None:
        """The controlled extension must request downloads and serve its test payload locally."""

        manifest = json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))
        self.assertIn("downloads", manifest["permissions"])
        payload = (FIXTURE / "download.txt").read_bytes()
        self.assertEqual(payload, b"OriginWeave deterministic MV3 download fixture.\n")

    def test_service_worker_executes_and_verifies_a_real_loopback_download(self) -> None:
        """Evidence must originate from the controlled fixture origin and bounded inspection."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        for expected in (
            "chrome.downloads.download",
            "chrome.downloads.search",
            'new URL("download.txt", sourceUrl).href',
            'parsed.hostname !== "127.0.0.1"',
            'parsed.protocol !== "http:"',
            "downloadsReady",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertNotIn('chrome.runtime.getURL("download.txt")', worker)

    def test_download_failures_emit_only_bounded_stage_diagnostics(self) -> None:
        """Fixture diagnostics must name a reviewed stage without retaining raw browser errors."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        for expected in (
            "download-source-rejected",
            "download-start-rejected",
            "download-search-missing",
            "download-interrupted",
            "download-url-mismatch",
            "download-byte-count-mismatch",
            "download-exists-false",
            "download-timeout",
            "download-complete-ready",
            "downloadsDiagnostic",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertIn("originweaveDownloadsDiagnostic", content)
        self.assertNotIn("download.default_directory", worker)
        self.assertNotIn("item.filename", worker)
        self.assertNotIn("_error.message", worker)
        self.assertNotIn("String(_error)", worker)

    def test_content_script_and_runner_require_downloads_on_every_pass(self) -> None:
        """The compatibility report must fail closed when downloads evidence is missing."""

        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("originweaveDownloads", content)
        self.assertIn('"downloads": surfaces["downloads"] == "ready"', runner)
        self.assertIn('"downloads": "ready"', runner)


if __name__ == "__main__":
    unittest.main()
