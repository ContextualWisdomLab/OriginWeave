"""Repository contract for the executable Web Audio privacy boundary."""

from __future__ import annotations

import json
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
EXTENSION = ROOT / "extensions" / "originweave-privacy-guard"
FIXTURE = ROOT / "tests" / "fixtures" / "web_audio_privacy"
RUNNER = ROOT / "scripts" / "ci" / "run_web_audio_privacy.py"
WORKFLOW = ROOT / ".github" / "workflows" / "mv3-compatibility.yml"
RUST_POLICY = (
    ROOT
    / "crates"
    / "originweave-fingerprint"
    / "src"
    / "web_audio_guard.rs"
)


class WebAudioPrivacyContractTests(unittest.TestCase):
    """Keep the default-deny browser guard explicit and reproducible."""

    def test_manifest_runs_before_page_scripts_in_main_world_and_all_frames(self) -> None:
        """The guard must reach the page's own constructors before any page script."""

        manifest = json.loads((EXTENSION / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["manifest_version"], 3)
        self.assertNotIn("permissions", manifest)
        self.assertNotIn("host_permissions", manifest)
        scripts = manifest["content_scripts"]
        self.assertEqual(len(scripts), 1)
        script = scripts[0]
        self.assertEqual(script["matches"], ["http://*/*", "https://*/*"])
        self.assertEqual(script["js"], ["web_audio_guard.js"])
        self.assertEqual(script["run_at"], "document_start")
        self.assertEqual(script["world"], "MAIN")
        self.assertIs(script["all_frames"], True)
        self.assertIs(script["match_about_blank"], True)
        self.assertIs(script["match_origin_as_fallback"], True)

    def test_checked_in_guard_blocks_every_web_audio_entrypoint(self) -> None:
        """The reviewed guard must block online, offline, prefixed, and worklet APIs."""

        guard = (EXTENSION / "web_audio_guard.js").read_text(encoding="utf-8")
        self.assertEqual(guard.count("ORIGINWEAVE_ALLOWED_WEB_AUDIO_ORIGINS"), 1)
        for constructor in (
            "AudioContext",
            "webkitAudioContext",
            "OfflineAudioContext",
            "webkitOfflineAudioContext",
            "AudioWorkletNode",
        ):
            with self.subTest(constructor=constructor):
                self.assertIn(f'"{constructor}"', guard)
        self.assertIn('"NotAllowedError"', guard)
        self.assertIn("document_start", guard)
        for forbidden in ("eval(", "new Function", "fetch(", "XMLHttpRequest", "chrome."):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, guard)

    def test_rust_policy_includes_the_reviewed_guard_and_bounds_exact_grants(self) -> None:
        """Policy rendering must be source-bound, deterministic, and bounded."""

        policy = RUST_POLICY.read_text(encoding="utf-8")
        for expected in (
            "include_str!",
            "originweave-privacy-guard/web_audio_guard.js",
            "BTreeSet<Origin>",
            "MAX_ALLOWED_ORIGINS: usize = 128",
            "web_audio_fingerprinting_no_explicit_origin_grant",
            "replacen(ALLOWLIST_MARKER",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, policy)

    def test_fixture_probes_top_document_and_child_frame_before_dom_ready(self) -> None:
        """The browser fixture must test first-script and all-frame enforcement."""

        page = (FIXTURE / "page.html").read_text(encoding="utf-8")
        frame = (FIXTURE / "frame.html").read_text(encoding="utf-8")
        self.assertIn("originweaveAudioContext", page)
        self.assertIn("originweaveAudioWorkletNode", page)
        self.assertIn('src="frame.html"', page)
        self.assertIn("originweaveOfflineAudioContext", frame)
        self.assertNotIn("DOMContentLoaded", page)
        self.assertNotIn("DOMContentLoaded", frame)

    def test_runner_is_pinned_loopback_only_and_repeated(self) -> None:
        """Privacy evidence must use one pinned browser and three isolated trials."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            'PINNED_CHROME_VERSION = "150.0.7871.129"',
            'PINNED_CHROME_REVISION = "r1639810"',
            "REPEATABILITY_TRIALS = 3",
            "http.client.HTTPConnection",
            '"127.0.0.1"',
            "originweave-privacy-guard",
            "top_document_and_child_frame",
            "trial_pass_rate",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)
        self.assertNotIn("urllib.request", runner)
        self.assertNotIn("NVIDIA_NIM_API_KEY", runner)

        namespace = runpy.run_path(str(RUNNER), run_name="web_audio_contract")
        validate = namespace["_path_token"]
        token = "f.3A0B2C.d.9D8E7F.e.2"
        self.assertEqual(validate(token, "session identifier"), token)
        for invalid in (".", "..", "f/escape", "f%2Fescape", "f?query"):
            with self.subTest(invalid=invalid):
                with self.assertRaises(RuntimeError):
                    validate(invalid, "session identifier")

        satisfies = namespace["_privacy_evidence_satisfies"]
        protected = {
            "audioContext": "blocked",
            "webkitAudioContext": "unavailable",
            "offlineAudioContext": "blocked",
            "webkitOfflineAudioContext": "unavailable",
            "audioWorkletNode": "blocked",
            "childAudioContext": "blocked",
            "childOfflineAudioContext": "blocked",
        }
        self.assertIs(satisfies(protected), True)
        prefixed_blocked = dict(protected)
        prefixed_blocked["webkitAudioContext"] = "blocked"
        prefixed_blocked["webkitOfflineAudioContext"] = "blocked"
        self.assertIs(satisfies(prefixed_blocked), True)
        leaked = dict(protected)
        leaked["audioContext"] = "leaked"
        self.assertIs(satisfies(leaked), False)
        missing = dict(protected)
        del missing["childAudioContext"]
        self.assertIs(satisfies(missing), False)

    def test_workflow_executes_and_publishes_the_real_browser_privacy_gate(self) -> None:
        """The pinned Chromium workflow must execute and retain privacy evidence."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        for expected in (
            "run_web_audio_privacy.py",
            "web-audio-privacy.json",
            "extensions/originweave-privacy-guard/**",
            "tests/fixtures/web_audio_privacy/**",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, workflow)
        self.assertNotIn("contents: write", workflow)


if __name__ == "__main__":
    unittest.main()
