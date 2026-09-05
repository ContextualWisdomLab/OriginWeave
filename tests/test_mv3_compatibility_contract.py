"""Repository contract for the executable Manifest V3 compatibility lane."""

from __future__ import annotations

import http.client
import json
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
WORKFLOW = ROOT / ".github" / "workflows" / "mv3-compatibility.yml"


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

    def test_fixture_expands_the_declared_core_mv3_api_matrix(self) -> None:
        """The real-browser lane must exercise core Chrome APIs beyond the initial smoke set."""

        manifest = json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))
        for permission in ("tabs", "scripting", "sidePanel"):
            with self.subTest(permission=permission):
                self.assertIn(permission, manifest["permissions"])
        self.assertEqual(manifest["side_panel"]["default_path"], "side_panel.html")
        self.assertIn("originweave-fixture-command", manifest["commands"])
        self.assertTrue((FIXTURE / "side_panel.html").is_file())

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "chrome.tabs.query",
            "chrome.windows.getCurrent",
            "chrome.scripting.executeScript",
            "chrome.commands.getAll",
            "chrome.sidePanel.getOptions",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        for expected in (
            "originweaveTabs",
            "originweaveWindows",
            "originweaveScripting",
            "originweaveCommands",
            "originweaveSidePanel",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, content)
        for surface in ("tabs", "windows", "scripting", "commands", "side-panel"):
            with self.subTest(surface=surface):
                self.assertIn(f'"{surface}"', runner)

    def test_runner_pins_one_chrome_for_testing_revision(self) -> None:
        """The compatibility lane must not silently float to a new Chromium build."""

        runner = RUNNER.read_text(encoding="utf-8")
        workflow = WORKFLOW.read_text(encoding="utf-8")
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
        self.assertIn('CHROME_VERSION: "150.0.7871.129"', workflow)
        combined = f"{runner}\n{workflow}".lower()
        for forbidden in (
            "last-known-good-versions",
            "known-good-versions-with-downloads",
            "latest-versions-per-milestone",
            "latest-patch-versions-per-build",
            "google-chrome-stable",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, combined)

    def test_runner_transport_cannot_follow_dynamic_url_schemes(self) -> None:
        """WebDriver control transport must be hard-bound to loopback HTTP only."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("http.client.HTTPConnection", runner)
        self.assertIn('"127.0.0.1"', runner)
        self.assertNotIn("urllib.request", runner)
        self.assertNotIn("urllib.error", runner)

    def test_runner_cleanup_cannot_suppress_untyped_failures(self) -> None:
        """Cleanup failures must stay typed evidence instead of becoming false-green runs."""

        runner = RUNNER.read_text(encoding="utf-8")
        self.assertNotIn("contextlib.suppress(Exception)", runner)
        self.assertIn("_delete_webdriver_session_bounded", runner)
        self.assertIn("_terminate_owned_process_bounded", runner)

    def test_runner_session_cleanup_classifies_http_protocol_failure(self) -> None:
        """Malformed ChromeDriver HTTP during cleanup must remain typed evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_cleanup_http_failure")
        delete_session = namespace["_delete_webdriver_session_bounded"]

        def fail_with_bad_status(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise http.client.BadStatusLine("malformed status line")

        delete_session.__globals__["_json_request"] = fail_with_bad_status
        self.assertEqual(
            delete_session(9515, "controlled-session"),
            "BadStatusLine",
        )

    def test_runner_startup_retries_http_protocol_failure(self) -> None:
        """A transient malformed ChromeDriver startup response must be retried boundedly."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_startup_http_failure")
        wait_for_driver = namespace["_wait_for_driver"]
        attempts = [0]

        def transient_bad_status(*_args: object, **_kwargs: object) -> dict[str, object]:
            attempts[0] += 1
            if attempts[0] == 1:
                raise http.client.BadStatusLine("malformed status line")
            return {"value": {"ready": True}}

        wait_for_driver.__globals__["_json_request"] = transient_bad_status
        wait_for_driver(9515)
        self.assertEqual(attempts[0], 2)

    def test_runner_accepts_real_chromedriver_element_ids_without_path_injection(self) -> None:
        """ChromeDriver dotted element IDs must work while path syntax stays fail-closed."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_contract")
        validate = namespace["_path_token"]
        element_id = "f.3A0B2C.d.9D8E7F.e.2"
        self.assertEqual(validate(element_id, "element identifier"), element_id)
        for invalid in (".", "..", "f/escape", "f%2Fescape", "f?query", "f#fragment"):
            with self.subTest(invalid=invalid):
                with self.assertRaises(RuntimeError):
                    validate(invalid, "element identifier")

    def test_runner_proves_restart_and_storage_persistence(self) -> None:
        """A second browser session must prove MV3 state survives a real browser restart."""

        runner = RUNNER.read_text(encoding="utf-8")
        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        for expected in (
            "_run_browser_pass",
            "restart-persistence",
            "worker-start-count",
            "storage-persistence",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)
        self.assertIn("originweave_worker_start_count", worker)
        self.assertIn("originweaveWorkerStartCount", content)

    def test_runner_reports_repeated_trial_pass_rate(self) -> None:
        """Release evidence must quantify repeatability rather than one lucky browser run."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "REPEATABILITY_TRIALS = 3",
            "trial_results",
            "trial_pass_rate",
            "successful_trials",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_workflow_runs_the_real_browser_lane_without_model_credentials(self) -> None:
        """Compatibility evidence must execute Chromium and never require LLM secrets."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
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
