#!/usr/bin/env python3
"""Prove the Web Audio privacy guard in a pinned real Chromium build.

The runner serves a controlled loopback fixture, loads only the reviewed
OriginWeave privacy extension, and verifies that the first page scripts in both
the top document and a child frame receive `NotAllowedError` from every exposed
Web Audio construction entry point. It emits bounded credential-free evidence.
"""

from __future__ import annotations

import contextlib
import http.client
import http.server
import json
import os
import pathlib
import socket
import string
import subprocess
import tempfile
import threading
import time
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
EXTENSION = ROOT / "extensions" / "originweave-privacy-guard"
FIXTURE = ROOT / "tests" / "fixtures" / "web_audio_privacy"
PINNED_CHROME_VERSION = "150.0.7871.129"
PINNED_CHROME_REVISION = "r1639810"
REPEATABILITY_TRIALS = 3
REQUEST_TIMEOUT_SECONDS = 5.0
STARTUP_TIMEOUT_SECONDS = 20.0
FIXTURE_TIMEOUT_SECONDS = 20.0
MAX_WEBDRIVER_RESPONSE_BYTES = 1_048_576
PATH_TOKEN_CHARACTERS = frozenset(string.ascii_letters + string.digits + "-_.")


class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):
    """Serve only the controlled local fixture without request logging."""

    def log_message(self, _format: str, *args: object) -> None:
        """Suppress fixture request logs that contain no test evidence."""


def _free_loopback_port() -> int:
    """Reserve and release one loopback port for a short-lived local service."""

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _path_token(value: str, label: str) -> str:
    """Validate a ChromeDriver-issued token before using it in a request path."""

    if (
        not value
        or len(value) > 256
        or value in {".", ".."}
        or any(character not in PATH_TOKEN_CHARACTERS for character in value)
    ):
        raise RuntimeError(f"invalid WebDriver {label}")
    return value


def _webdriver_path(session_id: str, suffix: str) -> str:
    """Build one bounded ChromeDriver path from a validated session token."""

    safe_session = _path_token(session_id, "session identifier")
    if suffix and not suffix.startswith("/"):
        raise RuntimeError("invalid WebDriver path suffix")
    if "://" in suffix or any(character in suffix for character in "\r\n"):
        raise RuntimeError("invalid WebDriver path suffix")
    return f"/session/{safe_session}{suffix}"


def _json_request(
    driver_port: int,
    method: str,
    path: str,
    payload: dict[str, Any] | None = None,
    *,
    timeout: float = REQUEST_TIMEOUT_SECONDS,
) -> dict[str, Any]:
    """Issue one bounded JSON request to the fixed loopback ChromeDriver."""

    if not 1 <= driver_port <= 65_535:
        raise ValueError("invalid ChromeDriver port")
    if method not in {"GET", "POST", "DELETE"}:
        raise ValueError("unsupported ChromeDriver method")
    if not path.startswith("/") or "://" in path or any(
        character in path for character in "\r\n"
    ):
        raise ValueError("invalid ChromeDriver path")

    body = None if payload is None else json.dumps(payload).encode("utf-8")
    connection = http.client.HTTPConnection("127.0.0.1", driver_port, timeout=timeout)
    try:
        connection.request(
            method,
            path,
            body=body,
            headers={"Content-Type": "application/json"},
        )
        response = connection.getresponse()
        raw = response.read(MAX_WEBDRIVER_RESPONSE_BYTES + 1)
        if len(raw) > MAX_WEBDRIVER_RESPONSE_BYTES:
            raise RuntimeError("WebDriver response exceeded the bounded JSON limit")
        if response.status >= 400:
            detail = raw.decode("utf-8", errors="replace")
            raise RuntimeError(f"WebDriver HTTP {response.status}: {detail}")
    finally:
        connection.close()

    decoded = json.loads(raw.decode("utf-8"))
    if not isinstance(decoded, dict):
        raise RuntimeError("WebDriver returned a non-object JSON payload")
    value = decoded.get("value")
    if isinstance(value, dict) and value.get("error"):
        raise RuntimeError(f"WebDriver error: {value.get('error')}: {value.get('message')}")
    return decoded


def _wait_for_driver(driver_port: int) -> None:
    """Wait for the exact local ChromeDriver process to become ready."""

    deadline = time.monotonic() + STARTUP_TIMEOUT_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            status = _json_request(driver_port, "GET", "/status", timeout=1.0)
            if status.get("value", {}).get("ready") is True:
                return
        except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
            last_error = exc
        time.sleep(0.1)
    raise RuntimeError(f"ChromeDriver did not become ready: {last_error}")


def _execute(driver_port: int, session_id: str, script: str) -> Any:
    """Execute fixture-only JavaScript in the controlled WebDriver session."""

    response = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/execute/sync"),
        {"script": script, "args": []},
    )
    return response.get("value")


def _wait_for_privacy_evidence(driver_port: int, session_id: str) -> dict[str, str]:
    """Wait until top-document and child-frame privacy probes are complete."""

    script = """
const root = document.documentElement.dataset;
const frame = document.querySelector("#privacy-frame");
const child = frame?.contentDocument?.documentElement?.dataset;
return {
  audioContext: root.originweaveAudioContext || "missing",
  webkitAudioContext: root.originweaveWebkitAudioContext || "missing",
  offlineAudioContext: root.originweaveOfflineAudioContext || "missing",
  webkitOfflineAudioContext: root.originweaveWebkitOfflineAudioContext || "missing",
  audioWorkletNode: root.originweaveAudioWorkletNode || "missing",
  childAudioContext: child?.originweaveAudioContext || "missing",
  childOfflineAudioContext: child?.originweaveOfflineAudioContext || "missing"
};
"""
    expected_keys = {
        "audioContext",
        "webkitAudioContext",
        "offlineAudioContext",
        "webkitOfflineAudioContext",
        "audioWorkletNode",
        "childAudioContext",
        "childOfflineAudioContext",
    }
    deadline = time.monotonic() + FIXTURE_TIMEOUT_SECONDS
    latest: dict[str, str] = {}
    while time.monotonic() < deadline:
        value = _execute(driver_port, session_id, script)
        if isinstance(value, dict):
            latest = {str(key): str(item) for key, item in value.items()}
            if set(latest) == expected_keys and all(
                item == "blocked" for item in latest.values()
            ):
                return latest
        time.sleep(0.1)
    raise RuntimeError(f"Web Audio privacy fixture did not converge: {latest!r}")


def _run_trial(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    trial_number: int,
) -> dict[str, Any]:
    """Run one independent browser process and return credential-free evidence."""

    driver_port = _free_loopback_port()
    session_id: str | None = None
    started = time.monotonic()
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-web-audio-{trial_number}-"
    ) as profile_dir:
        driver = subprocess.Popen(
            [str(chromedriver_bin), f"--port={driver_port}", "--allowed-ips=127.0.0.1"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.STDOUT,
            text=True,
        )
        try:
            _wait_for_driver(driver_port)
            session = _json_request(
                driver_port,
                "POST",
                "/session",
                {
                    "capabilities": {
                        "alwaysMatch": {
                            "browserName": "chrome",
                            "goog:chromeOptions": {
                                "binary": str(chrome_bin),
                                "args": [
                                    "--headless=new",
                                    "--no-first-run",
                                    "--disable-default-apps",
                                    "--disable-component-update",
                                    "--disable-sync",
                                    "--disable-dev-shm-usage",
                                    "--no-sandbox",
                                    f"--user-data-dir={profile_dir}",
                                    f"--disable-extensions-except={EXTENSION}",
                                    f"--load-extension={EXTENSION}",
                                ],
                            },
                        }
                    }
                },
            ).get("value", {})
            if not isinstance(session, dict):
                raise RuntimeError("ChromeDriver session response is malformed")
            raw_session_id = session.get("sessionId")
            capabilities = session.get("capabilities", {})
            if not isinstance(raw_session_id, str):
                raise RuntimeError("ChromeDriver did not return a session id")
            session_id = _path_token(raw_session_id, "session identifier")
            browser_version = (
                capabilities.get("browserVersion")
                if isinstance(capabilities, dict)
                else None
            )
            if browser_version != PINNED_CHROME_VERSION:
                raise RuntimeError(
                    f"unexpected Chrome version: expected {PINNED_CHROME_VERSION}, "
                    f"got {browser_version!r}"
                )

            _json_request(
                driver_port,
                "POST",
                _webdriver_path(session_id, "/url"),
                {"url": fixture_url},
            )
            surfaces = _wait_for_privacy_evidence(driver_port, session_id)
            return {
                "trial_number": trial_number,
                "passed": True,
                "browser_version": browser_version,
                "surfaces": surfaces,
                "duration_ms": round((time.monotonic() - started) * 1000),
            }
        finally:
            if session_id is not None:
                with contextlib.suppress(Exception):
                    _json_request(
                        driver_port,
                        "DELETE",
                        _webdriver_path(session_id, ""),
                        {},
                    )
            driver.terminate()
            try:
                driver.wait(timeout=5)
            except subprocess.TimeoutExpired:
                driver.kill()
                driver.wait(timeout=5)


def main() -> int:
    """Run three isolated trials and emit bounded repeatability evidence."""

    chrome_bin = pathlib.Path(os.environ.get("CHROME_BIN", ""))
    chromedriver_bin = pathlib.Path(os.environ.get("CHROMEDRIVER_BIN", ""))
    if not chrome_bin.is_file():
        raise SystemExit("CHROME_BIN must point to the pinned Chrome executable")
    if not chromedriver_bin.is_file():
        raise SystemExit("CHROMEDRIVER_BIN must point to the matching ChromeDriver")
    for required in (
        EXTENSION / "manifest.json",
        EXTENSION / "web_audio_guard.js",
        FIXTURE / "page.html",
        FIXTURE / "frame.html",
    ):
        if not required.is_file():
            raise SystemExit(f"required privacy fixture file is missing: {required.name}")

    fixture_server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0),
        lambda *args, **kwargs: QuietFixtureHandler(
            *args, directory=str(FIXTURE), **kwargs
        ),
    )
    fixture_thread = threading.Thread(target=fixture_server.serve_forever, daemon=True)
    fixture_thread.start()
    started = time.monotonic()
    try:
        fixture_url = f"http://127.0.0.1:{fixture_server.server_port}/page.html"
        trials: list[dict[str, Any]] = []
        for trial_number in range(1, REPEATABILITY_TRIALS + 1):
            try:
                trials.append(
                    _run_trial(
                        chrome_bin,
                        chromedriver_bin,
                        fixture_url,
                        trial_number,
                    )
                )
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                trials.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                        "error_type": type(exc).__name__,
                    }
                )

        successful_trials = sum(trial.get("passed") is True for trial in trials)
        evidence = {
            "chrome_version": PINNED_CHROME_VERSION,
            "chrome_revision": PINNED_CHROME_REVISION,
            "repeatability_trials": REPEATABILITY_TRIALS,
            "successful_trials": successful_trials,
            "trial_pass_rate": successful_trials / REPEATABILITY_TRIALS,
            "guard_profile": "default_deny_exact_origin_grants",
            "top_document_and_child_frame": True,
            "trial_results": trials,
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
        print(json.dumps(evidence, sort_keys=True))
        if successful_trials != REPEATABILITY_TRIALS:
            raise RuntimeError(
                "Web Audio privacy gate failed: "
                f"{successful_trials}/{REPEATABILITY_TRIALS} trials passed"
            )
        return 0
    finally:
        fixture_server.shutdown()
        fixture_server.server_close()
        fixture_thread.join(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
