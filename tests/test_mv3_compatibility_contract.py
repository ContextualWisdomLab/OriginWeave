"""Repository contract for the executable Manifest V3 compatibility lane."""

from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"


class ManifestV3CompatibilityContractTests(unittest.TestCase):
    """Keep the Chromium compatibility fixture explicit, bounded, and reproducible."""

    def test_fixture_exercises_required_mv3_surfaces(self) -> None:
        """The fixture must cover worker, content-script, storage, and DNR behavior."""

        manifest_path = FIXTURE / "manifest.json"
        self.assertTrue(manifest_path.is_file())
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        self.assertEqual(manifest["manifest_version"], 3)
        self.assertEqual(manifest["background"]["service_worker"], "service_worker.js")
        self.assertIn("storage", manifest["permissions"])
        self.assertIn("declarativeNetRequest", manifest["permissions"])
        self.assertIn("declarativeNetRequestWithHostAccess", manifest["permissions"])
        self.assertEqual(
            manifest["declarative_net_request"]["rule_resources"][0]["path"],
            "rules.json",
        )
        scripts = manifest["content_scripts"]
        self.assertEqual(len(scripts), 1)
        self.assertEqual(scripts[0]["js"], ["content_script.js"])
        self.assertEqual(scripts[0]["matches"], ["http://127.0.0.1/*"])
        self.assertEqual(manifest["host_permissions"], ["http://127.0.0.1/*"])
        for relative in (
            "service_worker.js",
            "content_script.js",
            "rules.json",
            "page.html",
        ):
            self.assertTrue((FIXTURE / relative).is_file(), relative)

    def test_runner_pins_one_chrome_for_testing_revision(self) -> None:
        """The compatibility lane must not silently float to a new Chromium build."""

        runner = (ROOT / "scripts" / "ci" / "run_mv3_compatibility.py").read_text(
            encoding="utf-8"
        )
        for expected in (
            "150.0.7871.129",
            "r1639810",
            "CHROME_BIN",
            "CHROMEDRIVER_BIN",
            "--load-extension",
            "--disable-extensions-except",
            "127.0.0.1",
            "service-worker",
            "content-script",
            "declarative-net-request",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)
        self.assertNotIn("latest", runner.lower())
        self.assertNotIn("google-chrome-stable", runner)

    def test_runner_transport_cannot_follow_dynamic_url_schemes(self) -> None:
        """WebDriver control transport must be hard-bound to loopback HTTP only."""

        runner = (ROOT / "scripts" / "ci" / "run_mv3_compatibility.py").read_text(
            encoding="utf-8"
        )
        self.assertIn("http.client.HTTPConnection", runner)
        self.assertIn('"127.0.0.1"', runner)
        self.assertNotIn("urllib.request", runner)
        self.assertNotIn("urllib.error", runner)

    def test_workflow_runs_the_real_browser_lane_without_model_credentials(self) -> None:
        """Compatibility evidence must execute Chromium and never require LLM secrets."""

        workflow = (ROOT / ".github" / "workflows" / "mv3-compatibility.yml").read_text(
            encoding="utf-8"
        )
        for expected in (
            "150.0.7871.129",
            "chrome-linux64.zip",
            "chromedriver-linux64.zip",
            "run_mv3_compatibility.py",
            "permissions:\n  contents: read",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, workflow)
        self.assertNotIn("NVIDIA_NIM_API_KEY", workflow)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", workflow)
        self.assertNotIn("contents: write", workflow)

    def test_doctoring_records_primary_chromium_evidence(self) -> None:
        """The exact browser baseline and non-compatibility claims must be documented."""

        doctoring = (ROOT / "docs" / "doctoring" / "mv3-compatibility.md").read_text(
            encoding="utf-8"
        )
        for expected in (
            "150.0.7871.129",
            "r1639810",
            "Manifest V3",
            "Chrome for Testing",
            "not claim 100% Chrome extension compatibility",
            "Chrome for Developers",
            "Google Chrome Labs",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, doctoring)


if __name__ == "__main__":
    unittest.main()
