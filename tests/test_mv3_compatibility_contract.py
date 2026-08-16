"""Repository contract for the executable Manifest V3 compatibility lane."""

from __future__ import annotations

import json
import pathlib
import runpy
import unittest
import unittest.mock

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

    def test_runner_preserves_safe_surface_failure_evidence(self) -> None:
        """A failed trial must identify the bounded fixture surface without leaking raw errors."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_contract")
        surface_error = namespace["CompatibilitySurfaceError"]
        failure_evidence = namespace["_failure_evidence"]

        observed = {"downloads": "missing", "storage": "ready"}
        diagnostic = failure_evidence(surface_error(observed))
        self.assertEqual(diagnostic["failure_kind"], "surface_mismatch")
        self.assertEqual(diagnostic["observed"], observed)

        generic = failure_evidence(
            RuntimeError("secret-token https://example.invalid /home/runner/private")
        )
        self.assertEqual(generic, {"failure_kind": "runtime_error"})

    def test_webdriver_errors_do_not_retain_raw_response_payloads(self) -> None:
        """WebDriver protocol failures must stay useful without copying raw browser text."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_contract")
        json_request = namespace["_json_request"]
        http_module = namespace["http"]

        class FakeResponse:
            def __init__(self, status: int, body: bytes) -> None:
                self.status = status
                self.body = body

            def read(self, _limit: int) -> bytes:
                return self.body

        class FakeConnection:
            def __init__(self, response: FakeResponse) -> None:
                self.response = response

            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> FakeResponse:
                return self.response

            def close(self) -> None:
                return None

        raw_secret = "secret-token /home/runner/private https://example.invalid"
        cases = (
            FakeResponse(500, raw_secret.encode("utf-8")),
            FakeResponse(
                200,
                json.dumps(
                    {
                        "value": {
                            "error": "unknown error",
                            "message": raw_secret,
                        }
                    }
                ).encode("utf-8"),
            ),
        )
        for response in cases:
            with self.subTest(status=response.status):
                with unittest.mock.patch.object(
                    http_module.client,
                    "HTTPConnection",
                    return_value=FakeConnection(response),
                ):
                    with self.assertRaises(RuntimeError) as raised:
                        json_request(9515, "GET", "/status")
                rendered = str(raised.exception)
                self.assertNotIn("secret-token", rendered)
                self.assertNotIn("/home/runner/private", rendered)
                self.assertNotIn("example.invalid", rendered)

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
