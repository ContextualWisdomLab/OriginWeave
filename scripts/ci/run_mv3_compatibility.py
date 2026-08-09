#!/usr/bin/env python3
"""Run the bounded Manifest V3 compatibility fixture against pinned Chromium.

This is a release/CI evidence runner, not a product browser adapter. It uses the
W3C WebDriver HTTP protocol only to prove that a real Chrome for Testing build
can load the controlled MV3 fixture and exercise service-worker, content-script,
storage, declarative-net-request, and real browser-click behavior.
"""

from __future__ import annotations

import contextlib
import http.server
import json
import os
import pathlib
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
PINNED_CHROME_VERSION = "150.0.7871.129"
PINNED_CHROME_REVISION = "r1639810"
REQUEST_TIMEOUT_SECONDS = 5.0
STARTUP_TIMEOUT_SECONDS = 20.0
FIXTURE_TIMEOUT_SECONDS = 20.0
W3C_ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"


class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):
    """Serve only the controlled local fixture without noisy access logging."""

    def log_message(self, _format: str, *args: object) -> None:
        """Suppress request logs because the fixture contains no diagnostic value."""


def _free_loopback_port() -> int:
    """Reserve and release one loopback TCP port for a short-lived local service."""

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _json_request(
    method: str,
    url: str,
    payload: dict[str, Any] | None = None,
    *,
    timeout: float = REQUEST_TIMEOUT_SECONDS,
) -> dict[str, Any]:
    """Issue one bounded JSON WebDriver request and return its decoded object."""

    body = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        method=method,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read(1_048_576)
    except urllib.error.HTTPError as exc:
        detail = exc.read(65_536).decode("utf-8", errors="replace")
        raise RuntimeError(f"WebDriver HTTP {exc.code}: {detail}") from exc
    decoded = json.loads(raw.decode("utf-8"))
    if not isinstance(decoded, dict):
        raise RuntimeError("WebDriver returned a non-object JSON payload")
    value = decoded.get("value")
    if isinstance(value, dict) and value.get("error"):
        raise RuntimeError(f"WebDriver error: {value.get('error')}: {value.get('message')}")
    return decoded


def _wait_for_driver(base_url: str) -> None:
    """Wait for the exact local ChromeDriver process to become ready."""

    deadline = time.monotonic() + STARTUP_TIMEOUT_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            status = _json_request("GET", f"{base_url}/status", timeout=1.0)
            if status.get("value", {}).get("ready") is True:
                return
        except (OSError, ValueError, RuntimeError) as exc:
            last_error = exc
        time.sleep(0.1)
    raise RuntimeError(f"ChromeDriver did not become ready: {last_error}")


def _execute(base_url: str, session_id: str, script: str) -> Any:
    """Run fixture-only JavaScript through the test WebDriver session."""

    response = _json_request(
        "POST",
        f"{base_url}/session/{session_id}/execute/sync",
        {"script": script, "args": []},
    )
    return response.get("value")


def _wait_for_extension_evidence(base_url: str, session_id: str) -> dict[str, str]:
    """Wait until every controlled MV3 fixture surface reports its expected result."""

    script = """
return {
  content: document.documentElement.dataset.originweaveContentScript || "missing",
  storage: document.documentElement.dataset.originweaveStorage || "missing",
  workerReply: document.documentElement.dataset.originweaveWorkerReply || "missing",
  workerState: document.documentElement.dataset.originweaveWorkerState || "missing",
  dnr: document.documentElement.dataset.originweaveDnr || "missing"
};
"""
    expected = {
        "content": "ready",
        "storage": "ready",
        "workerReply": "pong",
        "workerState": "installed",
        "dnr": "blocked",
    }
    deadline = time.monotonic() + FIXTURE_TIMEOUT_SECONDS
    latest: dict[str, str] = {}
    while time.monotonic() < deadline:
        value = _execute(base_url, session_id, script)
        if isinstance(value, dict):
            latest = {str(key): str(item) for key, item in value.items()}
            if latest == expected:
                return latest
        time.sleep(0.1)
    raise RuntimeError(f"MV3 fixture did not converge: expected={expected!r}, observed={latest!r}")


def _exercise_real_click(base_url: str, session_id: str) -> str:
    """Use the WebDriver element-click command and verify the DOM post-condition."""

    found = _json_request(
        "POST",
        f"{base_url}/session/{session_id}/element",
        {"using": "css selector", "value": "#fixture-button"},
    )
    element = found.get("value", {})
    element_id = element.get(W3C_ELEMENT_KEY) if isinstance(element, dict) else None
    if not isinstance(element_id, str) or not element_id:
        raise RuntimeError("WebDriver did not return a W3C element identifier")
    _json_request(
        "POST",
        f"{base_url}/session/{session_id}/element/{element_id}/click",
        {},
    )
    output = _json_request(
        "POST",
        f"{base_url}/session/{session_id}/element",
        {"using": "css selector", "value": "#fixture-output"},
    ).get("value", {})
    output_id = output.get(W3C_ELEMENT_KEY) if isinstance(output, dict) else None
    if not isinstance(output_id, str) or not output_id:
        raise RuntimeError("WebDriver did not return the fixture output element")
    text = _json_request(
        "GET", f"{base_url}/session/{session_id}/element/{output_id}/text"
    ).get("value")
    if text != "clicked":
        raise RuntimeError(f"real click post-condition failed: {text!r}")
    return str(text)


def main() -> int:
    """Run the pinned local-only MV3 compatibility probe and print bounded evidence."""

    chrome_bin = pathlib.Path(os.environ.get("CHROME_BIN", ""))
    chromedriver_bin = pathlib.Path(os.environ.get("CHROMEDRIVER_BIN", ""))
    if not chrome_bin.is_file():
        raise SystemExit("CHROME_BIN must point to the pinned Chrome for Testing executable")
    if not chromedriver_bin.is_file():
        raise SystemExit("CHROMEDRIVER_BIN must point to the matching pinned ChromeDriver")
    if not (FIXTURE / "manifest.json").is_file():
        raise SystemExit("MV3 fixture manifest is missing")

    fixture_server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0),
        lambda *args, **kwargs: QuietFixtureHandler(
            *args, directory=str(FIXTURE), **kwargs
        ),
    )
    fixture_thread = threading.Thread(target=fixture_server.serve_forever, daemon=True)
    fixture_thread.start()

    driver_port = _free_loopback_port()
    driver_url = f"http://127.0.0.1:{driver_port}"
    session_id: str | None = None
    started = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="originweave-mv3-profile-") as profile_dir:
        driver = subprocess.Popen(
            [str(chromedriver_bin), f"--port={driver_port}", "--allowed-ips=127.0.0.1"],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        try:
            _wait_for_driver(driver_url)
            session = _json_request(
                "POST",
                f"{driver_url}/session",
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
                                    f"--disable-extensions-except={FIXTURE}",
                                    f"--load-extension={FIXTURE}",
                                ],
                            },
                        }
                    }
                },
            ).get("value", {})
            if not isinstance(session, dict):
                raise RuntimeError("ChromeDriver session response is malformed")
            session_id = session.get("sessionId")
            capabilities = session.get("capabilities", {})
            if not isinstance(session_id, str) or not session_id:
                raise RuntimeError("ChromeDriver did not return a session id")
            browser_version = capabilities.get("browserVersion") if isinstance(capabilities, dict) else None
            if browser_version != PINNED_CHROME_VERSION:
                raise RuntimeError(
                    f"unexpected Chrome version: expected {PINNED_CHROME_VERSION}, got {browser_version!r}"
                )

            fixture_url = f"http://127.0.0.1:{fixture_server.server_port}/page.html"
            _json_request(
                "POST",
                f"{driver_url}/session/{session_id}/url",
                {"url": fixture_url},
            )
            surfaces = _wait_for_extension_evidence(driver_url, session_id)
            click_result = _exercise_real_click(driver_url, session_id)
            evidence = {
                "chrome_version": browser_version,
                "chrome_revision": PINNED_CHROME_REVISION,
                "surfaces": {
                    "service-worker": surfaces["workerReply"] == "pong",
                    "content-script": surfaces["content"] == "ready",
                    "storage": surfaces["storage"] == "ready",
                    "declarative-net-request": surfaces["dnr"] == "blocked",
                    "real-browser-click": click_result == "clicked",
                },
                "duration_ms": round((time.monotonic() - started) * 1000),
            }
            if not all(evidence["surfaces"].values()):
                raise RuntimeError(f"compatibility surface failed: {evidence!r}")
            print(json.dumps(evidence, sort_keys=True))
            return 0
        finally:
            if session_id is not None:
                with contextlib.suppress(Exception):
                    _json_request("DELETE", f"{driver_url}/session/{session_id}", {})
            driver.terminate()
            try:
                driver.wait(timeout=5)
            except subprocess.TimeoutExpired:
                driver.kill()
                driver.wait(timeout=5)
            fixture_server.shutdown()
            fixture_server.server_close()
            fixture_thread.join(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
