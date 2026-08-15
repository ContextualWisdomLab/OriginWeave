"""Fail-first contract for real Manifest V3 downloads compatibility."""

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

    def test_runner_preserves_only_reviewed_download_diagnostic_tokens(self) -> None:
        """Runner failure evidence must retain stage tokens while rejecting raw diagnostics."""

        runner = _load_runner_module()
        approved = {
            "download-source-rejected",
            "download-start-rejected",
            "download-search-missing",
            "download-interrupted",
            "download-url-mismatch",
            "download-byte-count-mismatch",
            "download-exists-false",
            "download-timeout",
            "download-complete-ready",
            "download-not-evaluated",
        }
        self.assertIn("downloadsDiagnostic", runner.SURFACE_EVIDENCE_KEYS)
        self.assertEqual(runner.DOWNLOAD_DIAGNOSTIC_VALUES, frozenset(approved))
        for token in approved:
            with self.subTest(token=token):
                self.assertEqual(
                    runner._safe_surface_value("downloadsDiagnostic", token), token
                )

        approved_error = runner.CompatibilitySurfaceError(
            {
                "downloads": "missing",
                "downloadsDiagnostic": "download-source-rejected",
            }
        )
        approved_evidence = runner._failure_evidence(approved_error)
        self.assertEqual(
            approved_evidence["observed"]["downloadsDiagnostic"],
            "download-source-rejected",
        )

        raw_download_path = str(ROOT / "private" / "download.txt")
        raw_browser_error = "Error: secret browser failure"
        for raw in (raw_download_path, raw_browser_error):
            with self.subTest(raw=raw):
                error = runner.CompatibilitySurfaceError(
                    {
                        "downloads": "missing",
                        "downloadsDiagnostic": raw,
                    }
                )
                evidence = runner._failure_evidence(error)
                self.assertEqual(
                    evidence["observed"]["downloadsDiagnostic"], "unexpected"
                )
                self.assertNotIn(raw, repr(evidence))

    def test_runner_collects_download_diagnostic_from_fixture_dataset(self) -> None:
        """The WebDriver evidence script must collect the bounded fixture diagnostic field."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("originweaveDownloadsDiagnostic", runner)
        self.assertIn('"downloadsDiagnostic": "download-complete-ready"', runner)


if __name__ == "__main__":
    unittest.main()
