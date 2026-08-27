#!/usr/bin/env python3
"""Repair the Web Audio browser evidence contract and delete this helper."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(relative: str, old: str, new: str) -> None:
    """Replace one exact repository anchor and fail closed on branch drift."""

    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one anchor in {relative}, found {count}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "scripts/ci/run_web_audio_privacy.py",
    'PATH_TOKEN_CHARACTERS = frozenset(string.ascii_letters + string.digits + "-_.")\n',
    'PATH_TOKEN_CHARACTERS = frozenset(string.ascii_letters + string.digits + "-_.")\n'
    'REQUIRED_BLOCKED_SURFACES = frozenset(\n'
    '    {\n'
    '        "audioContext",\n'
    '        "offlineAudioContext",\n'
    '        "audioWorkletNode",\n'
    '        "childAudioContext",\n'
    '        "childOfflineAudioContext",\n'
    '    }\n'
    ')\n'
    'OPTIONAL_PREFIXED_SURFACES = frozenset(\n'
    '    {"webkitAudioContext", "webkitOfflineAudioContext"}\n'
    ')\n'
    'EXPECTED_SURFACES = REQUIRED_BLOCKED_SURFACES | OPTIONAL_PREFIXED_SURFACES\n',
)

replace_once(
    "scripts/ci/run_web_audio_privacy.py",
    'class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):\n'
    '    """Serve only the controlled local fixture without request logging."""\n\n'
    '    def log_message(self, _format: str, *args: object) -> None:\n'
    '        """Suppress fixture request logs that contain no test evidence."""\n\n\n',
    'class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):\n'
    '    """Serve only the controlled local fixture without request logging."""\n\n'
    '    def log_message(self, _format: str, *args: object) -> None:\n'
    '        """Suppress fixture request logs that contain no test evidence."""\n\n\n'
    'class PrivacyProbeError(RuntimeError):\n'
    '    """A controlled fixture failed to reach its required privacy state."""\n\n'
    '    def __init__(self, surfaces: dict[str, str]) -> None:\n'
    '        """Preserve only bounded fixture surface states for diagnosis."""\n\n'
    '        self.surfaces = dict(surfaces)\n'
    '        super().__init__("Web Audio privacy fixture did not converge")\n\n\n'
    'def _privacy_evidence_satisfies(surfaces: dict[str, str]) -> bool:\n'
    '    """Accept blocked core surfaces and absent-or-blocked vendor aliases."""\n\n'
    '    if set(surfaces) != EXPECTED_SURFACES:\n'
    '        return False\n'
    '    if any(surfaces[name] != "blocked" for name in REQUIRED_BLOCKED_SURFACES):\n'
    '        return False\n'
    '    return all(\n'
    '        surfaces[name] in {"blocked", "unavailable"}\n'
    '        for name in OPTIONAL_PREFIXED_SURFACES\n'
    '    )\n\n\n',
)

replace_once(
    "scripts/ci/run_web_audio_privacy.py",
    '    expected_keys = {\n'
    '        "audioContext",\n'
    '        "webkitAudioContext",\n'
    '        "offlineAudioContext",\n'
    '        "webkitOfflineAudioContext",\n'
    '        "audioWorkletNode",\n'
    '        "childAudioContext",\n'
    '        "childOfflineAudioContext",\n'
    '    }\n'
    '    deadline = time.monotonic() + FIXTURE_TIMEOUT_SECONDS\n'
    '    latest: dict[str, str] = {}\n'
    '    while time.monotonic() < deadline:\n'
    '        value = _execute(driver_port, session_id, script)\n'
    '        if isinstance(value, dict):\n'
    '            latest = {str(key): str(item) for key, item in value.items()}\n'
    '            if set(latest) == expected_keys and all(\n'
    '                item == "blocked" for item in latest.values()\n'
    '            ):\n'
    '                return latest\n'
    '        time.sleep(0.1)\n'
    '    raise RuntimeError(f"Web Audio privacy fixture did not converge: {latest!r}")\n',
    '    deadline = time.monotonic() + FIXTURE_TIMEOUT_SECONDS\n'
    '    latest: dict[str, str] = {}\n'
    '    while time.monotonic() < deadline:\n'
    '        value = _execute(driver_port, session_id, script)\n'
    '        if isinstance(value, dict):\n'
    '            latest = {str(key): str(item) for key, item in value.items()}\n'
    '            if _privacy_evidence_satisfies(latest):\n'
    '                return latest\n'
    '        time.sleep(0.1)\n'
    '    raise PrivacyProbeError(latest)\n',
)

replace_once(
    "scripts/ci/run_web_audio_privacy.py",
    '            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:\n'
    '                trials.append(\n'
    '                    {\n'
    '                        "trial_number": trial_number,\n'
    '                        "passed": False,\n'
    '                        "error_type": type(exc).__name__,\n'
    '                    }\n'
    '                )\n',
    '            except PrivacyProbeError as exc:\n'
    '                trials.append(\n'
    '                    {\n'
    '                        "trial_number": trial_number,\n'
    '                        "passed": False,\n'
    '                        "error_type": type(exc).__name__,\n'
    '                        "surfaces": exc.surfaces,\n'
    '                    }\n'
    '                )\n'
    '            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:\n'
    '                trials.append(\n'
    '                    {\n'
    '                        "trial_number": trial_number,\n'
    '                        "passed": False,\n'
    '                        "error_type": type(exc).__name__,\n'
    '                    }\n'
    '                )\n',
)

replace_once(
    "tests/test_web_audio_privacy_contract.py",
    '        for invalid in (".", "..", "f/escape", "f%2Fescape", "f?query"):\n'
    '            with self.subTest(invalid=invalid):\n'
    '                with self.assertRaises(RuntimeError):\n'
    '                    validate(invalid, "session identifier")\n\n'
    '    def test_workflow_executes_and_publishes_the_real_browser_privacy_gate(self) -> None:\n',
    '        for invalid in (".", "..", "f/escape", "f%2Fescape", "f?query"):\n'
    '            with self.subTest(invalid=invalid):\n'
    '                with self.assertRaises(RuntimeError):\n'
    '                    validate(invalid, "session identifier")\n\n'
    '        satisfies = namespace["_privacy_evidence_satisfies"]\n'
    '        protected = {\n'
    '            "audioContext": "blocked",\n'
    '            "webkitAudioContext": "unavailable",\n'
    '            "offlineAudioContext": "blocked",\n'
    '            "webkitOfflineAudioContext": "unavailable",\n'
    '            "audioWorkletNode": "blocked",\n'
    '            "childAudioContext": "blocked",\n'
    '            "childOfflineAudioContext": "blocked",\n'
    '        }\n'
    '        self.assertIs(satisfies(protected), True)\n'
    '        prefixed_blocked = dict(protected)\n'
    '        prefixed_blocked["webkitAudioContext"] = "blocked"\n'
    '        prefixed_blocked["webkitOfflineAudioContext"] = "blocked"\n'
    '        self.assertIs(satisfies(prefixed_blocked), True)\n'
    '        leaked = dict(protected)\n'
    '        leaked["audioContext"] = "leaked"\n'
    '        self.assertIs(satisfies(leaked), False)\n'
    '        missing = dict(protected)\n'
    '        del missing["childAudioContext"]\n'
    '        self.assertIs(satisfies(missing), False)\n\n'
    '    def test_workflow_executes_and_publishes_the_real_browser_privacy_gate(self) -> None:\n',
)

(ROOT / "scripts/ci/repair_web_audio_privacy_probe.py").unlink()
(ROOT / ".github/workflows/one-shot-repair-web-audio-probe.yml").unlink()
