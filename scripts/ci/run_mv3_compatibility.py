#!/usr/bin/env python3
"""Run bounded repeatable Manifest V3 compatibility evidence against pinned Chromium.

This is a release/CI evidence runner, not a product browser adapter. It uses the
W3C WebDriver HTTP protocol only to prove that a real Chrome for Testing build
can load the controlled MV3 fixture and repeatedly exercise service-worker,
content-script, storage, declarative-net-request, tabs, windows, scripting,
commands, side-panel, bookmarks, history, downloads, real browser-click,
restart-persistence, and controlled extension update-migration behavior.
"""

from __future__ import annotations

import contextlib
import http.client
import http.server
import json
import os
import pathlib
import shutil
import socket
import string
import subprocess
import tempfile
import threading
import time
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
PINNED_CHROME_VERSION = "150.0.7871.129"
PINNED_CHROME_REVISION = "r1639810"
INITIAL_EXTENSION_VERSION = "1.0.0"
UPDATED_EXTENSION_VERSION = "1.0.1"
REPEATABILITY_TRIALS = 3
REQUEST_TIMEOUT_SECONDS = 5.0
STARTUP_TIMEOUT_SECONDS = 20.0
FIXTURE_TIMEOUT_SECONDS = 20.0
MAX_WEBDRIVER_RESPONSE_BYTES = 1_048_576
W3C_ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"
PATH_TOKEN_CHARACTERS = frozenset(string.ascii_letters + string.digits + "-_.")
SURFACE_EVIDENCE_KEYS = (
    "content",
    "storage",
    "storagePersistence",
    "workerReply",
    "workerState",
    "workerStartCount",
    "extensionVersion",
    "storageMigration",
    "dnr",
    "tabs",
    "windows",
    "scripting",
    "scriptingExecuted",
    "commands",
    "sidePanel",
    "bookmarks",
    "history",
    "downloads",
    "downloadsDiagnostic",
)
SURFACE_EVIDENCE_VALUES = frozenset(
    {
        "ready",
        "missing",
        "initialized",
        "persisted",
        "current",
        "migrated",
        "invalid",
        "pong",
        "installed",
        "blocked",
    }
)
DOWNLOAD_DIAGNOSTIC_VALUES = frozenset(
    {
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
)
EXTENSION_VERSION_EVIDENCE_VALUES = frozenset(
    {INITIAL_EXTENSION_VERSION, UPDATED_EXTENSION_VERSION, "missing"}
)


class CompatibilitySurfaceError(RuntimeError):
    """Report only bounded fixture-surface state when real-browser evidence does not converge."""

    def __init__(self, observed: dict[str, str]) -> None:
        self.observed = {
            key: _safe_surface_value(key, observed[key])
            for key in SURFACE_EVIDENCE_KEYS
            if key in observed
        }
        super().__init__("Manifest V3 fixture surfaces did not converge")


class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):
    """Serve only the controlled local fixture without noisy access logging."""

    def log_message(self, _format: str, *args: object) -> None:
        """Suppress request logs because the fixture contains no diagnostic value."""


def _safe_surface_value(key: str, value: str) -> str:
    """Reduce one controlled DOM evidence value to a non-sensitive diagnostic token."""

    if key == "workerStartCount":
        return value if value.isdecimal() and len(value) <= 20 else "invalid"
    if key == "downloadsDiagnostic":
        return value if value in DOWNLOAD_DIAGNOSTIC_VALUES else "unexpected"
    if key == "extensionVersion":
        return value if value in EXTENSION_VERSION_EVIDENCE_VALUES else "unexpected"
    return value if value in SURFACE_EVIDENCE_VALUES else "unexpected"


def _failure_evidence(error: BaseException) -> dict[str, Any]:
    """Classify one browser-trial failure without retaining raw exception text."""

    if isinstance(error, CompatibilitySurfaceError):
        return {"failure_kind": "surface_mismatch", "observed": error.observed}
    if isinstance(error, json.JSONDecodeError):
        return {"failure_kind": "json_decode_error"}
    if isinstance(error, OSError):
        return {"failure_kind": "io_error"}
    if isinstance(error, ValueError):
        return {"failure_kind": "value_error"}
    return {"failure_kind": "runtime_error"}


def _free_loopback_port() -> int:
    """Reserve and release one loopback TCP port for a short-lived local service."""

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _path_token(value: str, label: str) -> str:
    """Validate one ChromeDriver-issued identifier before interpolating a path."""

    if (
        not value
        or len(value) > 256
        or value in {".", ".."}
        or any(char not in PATH_TOKEN_CHARACTERS for char in value)
    ):
        raise RuntimeError(f"invalid WebDriver {label}")
    return value


def _webdriver_path(session_id: str, suffix: str) -> str:
    """Build one bounded ChromeDriver path from a validated session identifier."""

    safe_session = _path_token(session_id, "session identifier")
    if suffix and not suffix.startswith("/"):
        raise RuntimeError("invalid WebDriver path suffix")
    if "://" in suffix or any(char in suffix for char in "\r\n"):
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
    """Issue one bounded JSON request to the fixed loopback ChromeDriver authority."""

    if not 1 <= driver_port <= 65_535:
        raise ValueError("invalid ChromeDriver port")
    if method not in {"GET", "POST", "DELETE"}:
        raise ValueError("unsupported ChromeDriver method")
    if not path.startswith("/") or "://" in path or any(char in path for char in "\r\n"):
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
    """Run fixture-only JavaScript through the test WebDriver session."""

    response = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/execute/sync"),
        {"script": script, "args": []},
    )
    return response.get("value")


def _wait_for_extension_evidence(
    driver_port: int,
    session_id: str,
    expected_storage_persistence: str,
    expected_extension_version: str,
    expected_storage_migration: str,
) -> dict[str, str]:
    """Wait until every controlled MV3 fixture surface reports its expected result."""

    if expected_storage_persistence not in {"initialized", "persisted"}:
        raise ValueError("invalid storage persistence expectation")
    if expected_extension_version not in {
        INITIAL_EXTENSION_VERSION,
        UPDATED_EXTENSION_VERSION,
    }:
        raise ValueError("invalid extension version expectation")
    if expected_storage_migration not in {"initialized", "current", "migrated"}:
        raise ValueError("invalid storage migration expectation")
    script = """
return {
  content: document.documentElement.dataset.originweaveContentScript || "missing",
  storage: document.documentElement.dataset.originweaveStorage || "missing",
  storagePersistence:
    document.documentElement.dataset.originweaveStoragePersistence || "missing",
  workerReply: document.documentElement.dataset.originweaveWorkerReply || "missing",
  workerState: document.documentElement.dataset.originweaveWorkerState || "missing",
  workerStartCount:
    document.documentElement.dataset.originweaveWorkerStartCount || "missing",
  extensionVersion:
    document.documentElement.dataset.originweaveExtensionVersion || "missing",
  storageMigration:
    document.documentElement.dataset.originweaveStorageMigration || "missing",
  dnr: document.documentElement.dataset.originweaveDnr || "missing",
  tabs: document.documentElement.dataset.originweaveTabs || "missing",
  windows: document.documentElement.dataset.originweaveWindows || "missing",
  scripting: document.documentElement.dataset.originweaveScripting || "missing",
  scriptingExecuted:
    document.documentElement.dataset.originweaveScriptingExecuted || "missing",
  commands: document.documentElement.dataset.originweaveCommands || "missing",
  sidePanel: document.documentElement.dataset.originweaveSidePanel || "missing",
  bookmarks: document.documentElement.dataset.originweaveBookmarks || "missing",
  history: document.documentElement.dataset.originweaveHistory || "missing",
  downloads: document.documentElement.dataset.originweaveDownloads || "missing",
  downloadsDiagnostic:
    document.documentElement.dataset.originweaveDownloadsDiagnostic || "download-not-evaluated"
};
"""
    expected = {
        "content": "ready",
        "storage": "ready",
        "storagePersistence": expected_storage_persistence,
        "workerReply": "pong",
        "workerState": "installed",
        "extensionVersion": expected_extension_version,
        "storageMigration": expected_storage_migration,
        "dnr": "blocked",
        "tabs": "ready",
        "windows": "ready",
        "scripting": "ready",
        "scriptingExecuted": "ready",
        "commands": "ready",
        "sidePanel": "ready",
        "bookmarks": "ready",
        "history": "ready",
        "downloads": "ready",
        "downloadsDiagnostic": "download-complete-ready",
    }
    deadline = time.monotonic() + FIXTURE_TIMEOUT_SECONDS
    latest: dict[str, str] = {}
    while time.monotonic() < deadline:
        value = _execute(driver_port, session_id, script)
        if isinstance(value, dict):
            latest = {str(key): str(item) for key, item in value.items()}
            try:
                worker_start_count = int(latest.get("workerStartCount", "0"))
            except ValueError:
                worker_start_count = 0
            if worker_start_count > 0 and all(
                latest.get(key) == item for key, item in expected.items()
            ):
                return latest
        time.sleep(0.1)
    raise CompatibilitySurfaceError(latest)


def _exercise_real_click(driver_port: int, session_id: str) -> str:
    """Use the WebDriver element-click command and verify the DOM post-condition."""

    found = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/element"),
        {"using": "css selector", "value": "#fixture-button"},
    )
    element = found.get("value", {})
    element_id = element.get(W3C_ELEMENT_KEY) if isinstance(element, dict) else None
    if not isinstance(element_id, str):
        raise RuntimeError("WebDriver did not return a W3C element identifier")
    safe_element = _path_token(element_id, "element identifier")
    _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, f"/element/{safe_element}/click"),
        {},
    )
    output = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/element"),
        {"using": "css selector", "value": "#fixture-output"},
    ).get("value", {})
    output_id = output.get(W3C_ELEMENT_KEY) if isinstance(output, dict) else None
    if not isinstance(output_id, str):
        raise RuntimeError("WebDriver did not return the fixture output element")
    safe_output = _path_token(output_id, "element identifier")
    text = _json_request(
        driver_port,
        "GET",
        _webdriver_path(session_id, f"/element/{safe_output}/text"),
    ).get("value")
    if text != "clicked":
        raise RuntimeError(f"real click post-condition failed: {text!r}")
    return str(text)


def _set_fixture_version(extension_dir: pathlib.Path, version: str) -> None:
    """Set one controlled version only inside a trial-local extension copy."""

    if version not in {INITIAL_EXTENSION_VERSION, UPDATED_EXTENSION_VERSION}:
        raise ValueError("unsupported fixture extension version")
    resolved_extension_dir = extension_dir.resolve()
    if resolved_extension_dir == FIXTURE.resolve():
        raise RuntimeError("refusing to mutate the checked-in MV3 fixture")
    manifest_path = resolved_extension_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict):
        raise RuntimeError("MV3 fixture manifest must be a JSON object")
    current_version = manifest.get("version")
    if current_version not in {INITIAL_EXTENSION_VERSION, UPDATED_EXTENSION_VERSION}:
        raise RuntimeError("MV3 fixture manifest has an unexpected version")
    if version == INITIAL_EXTENSION_VERSION and current_version != INITIAL_EXTENSION_VERSION:
        raise RuntimeError("cannot rewind the trial-local extension version")
    if version == UPDATED_EXTENSION_VERSION and current_version != INITIAL_EXTENSION_VERSION:
        raise RuntimeError("extension update must start from the initial version")
    manifest["version"] = version
    manifest_path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def _run_browser_pass(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    profile_dir: pathlib.Path,
    extension_dir: pathlib.Path,
    expected_storage_persistence: str,
    expected_extension_version: str,
    expected_storage_migration: str,
) -> dict[str, Any]:
    """Run one fresh browser process against a shared bounded compatibility profile."""

    driver_port = _free_loopback_port()
    session_id: str | None = None
    download_dir = profile_dir / "downloads"
    download_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
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
                                f"--disable-extensions-except={extension_dir}",
                                f"--load-extension={extension_dir}",
                            ],
                            "prefs": {
                                "download.default_directory": str(download_dir),
                                "download.prompt_for_download": False,
                                "download.directory_upgrade": True,
                            },
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
            capabilities.get("browserVersion") if isinstance(capabilities, dict) else None
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
        surfaces = _wait_for_extension_evidence(
            driver_port,
            session_id,
            expected_storage_persistence,
            expected_extension_version,
            expected_storage_migration,
        )
        click_result = _exercise_real_click(driver_port, session_id)
        worker_start_count = int(surfaces["workerStartCount"])
        return {
            "browser_version": browser_version,
            "worker_start_count": worker_start_count,
            "storage_persistence": surfaces["storagePersistence"],
            "extension_version": surfaces["extensionVersion"],
            "storage_migration": surfaces["storageMigration"],
            "surfaces": {
                "service-worker": surfaces["workerReply"] == "pong",
                "content-script": surfaces["content"] == "ready",
                "storage": surfaces["storage"] == "ready",
                "declarative-net-request": surfaces["dnr"] == "blocked",
                "tabs": surfaces["tabs"] == "ready",
                "windows": surfaces["windows"] == "ready",
                "scripting": surfaces["scripting"] == "ready"
                and surfaces["scriptingExecuted"] == "ready",
                "commands": surfaces["commands"] == "ready",
                "side-panel": surfaces["sidePanel"] == "ready",
                "bookmarks": surfaces["bookmarks"] == "ready",
                "history": surfaces["history"] == "ready",
                "downloads": surfaces["downloads"] == "ready",
                "real-browser-click": click_result == "clicked",
            },
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


def _run_restart_trial(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    trial_number: int,
) -> dict[str, Any]:
    """Run one independent initial/restart/update-migration trial."""

    trial_started = time.monotonic()
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-mv3-trial-{trial_number}-"
    ) as trial_root:
        trial_dir = pathlib.Path(trial_root)
        profile_dir = trial_dir / "profile"
        extension_dir = trial_dir / "extension"
        shutil.copytree(FIXTURE, extension_dir)
        _set_fixture_version(extension_dir, INITIAL_EXTENSION_VERSION)

        initial = _run_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
            extension_dir,
            "initialized",
            INITIAL_EXTENSION_VERSION,
            "initialized",
        )
        restarted = _run_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
            extension_dir,
            "persisted",
            INITIAL_EXTENSION_VERSION,
            "current",
        )

        _set_fixture_version(extension_dir, UPDATED_EXTENSION_VERSION)
        updated = _run_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
            extension_dir,
            "persisted",
            UPDATED_EXTENSION_VERSION,
            "migrated",
        )

    initial_count = int(initial["worker_start_count"])
    restarted_count = int(restarted["worker_start_count"])
    updated_count = int(updated["worker_start_count"])
    surfaces = {
        name: bool(initial["surfaces"][name])
        and bool(restarted["surfaces"][name])
        and bool(updated["surfaces"][name])
        for name in initial["surfaces"]
    }
    surfaces.update(
        {
            "restart-persistence": restarted["storage_persistence"] == "persisted",
            "worker-start-count": restarted_count > initial_count
            and updated_count > restarted_count,
            "storage-persistence": updated["storage_persistence"] == "persisted",
            "update-migration": updated["extension_version"] == UPDATED_EXTENSION_VERSION
            and updated["storage_migration"] == "migrated",
        }
    )
    if not all(surfaces.values()):
        raise RuntimeError(f"compatibility surface failed in trial {trial_number}")

    return {
        "trial_number": trial_number,
        "passed": True,
        "browser_version": updated["browser_version"],
        "surfaces": surfaces,
        "browser_passes": [
            {
                "phase": "initial",
                "worker_start_count": initial_count,
                "storage_persistence": initial["storage_persistence"],
                "extension_version": initial["extension_version"],
                "storage_migration": initial["storage_migration"],
            },
            {
                "phase": "restart",
                "worker_start_count": restarted_count,
                "storage_persistence": restarted["storage_persistence"],
                "extension_version": restarted["extension_version"],
                "storage_migration": restarted["storage_migration"],
            },
            {
                "phase": "update-migration",
                "worker_start_count": updated_count,
                "storage_persistence": updated["storage_persistence"],
                "extension_version": updated["extension_version"],
                "storage_migration": updated["storage_migration"],
            },
        ],
        "duration_ms": round((time.monotonic() - trial_started) * 1000),
    }


def main() -> int:
    """Run three independent restart/update trials and emit bounded repeatability evidence."""

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
    started = time.monotonic()

    try:
        fixture_url = f"http://127.0.0.1:{fixture_server.server_port}/page.html"
        trial_results: list[dict[str, Any]] = []
        for trial_number in range(1, REPEATABILITY_TRIALS + 1):
            try:
                trial_results.append(
                    _run_restart_trial(
                        chrome_bin,
                        chromedriver_bin,
                        fixture_url,
                        trial_number,
                    )
                )
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                failed_trial: dict[str, Any] = {
                    "trial_number": trial_number,
                    "passed": False,
                }
                failed_trial.update(_failure_evidence(exc))
                trial_results.append(failed_trial)

        successful_trials = sum(
            1 for trial in trial_results if trial.get("passed") is True
        )
        trial_pass_rate = successful_trials / REPEATABILITY_TRIALS
        successful_results = [
            trial for trial in trial_results if trial.get("passed") is True
        ]
        common_surfaces: dict[str, bool] = {}
        if successful_results:
            first_surfaces = successful_results[0].get("surfaces", {})
            if isinstance(first_surfaces, dict):
                common_surfaces = {
                    str(name): all(
                        isinstance(trial.get("surfaces"), dict)
                        and trial["surfaces"].get(name) is True
                        for trial in successful_results
                    )
                    for name in first_surfaces
                }

        evidence = {
            "chrome_version": PINNED_CHROME_VERSION,
            "chrome_revision": PINNED_CHROME_REVISION,
            "repeatability_trials": REPEATABILITY_TRIALS,
            "successful_trials": successful_trials,
            "trial_pass_rate": trial_pass_rate,
            "surfaces": common_surfaces,
            "trial_results": trial_results,
            "browser_passes": (
                successful_results[-1].get("browser_passes", [])
                if successful_results
                else []
            ),
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
        print(json.dumps(evidence, sort_keys=True))
        if successful_trials != REPEATABILITY_TRIALS:
            raise RuntimeError(
                "Manifest V3 repeatability gate failed: "
                f"{successful_trials}/{REPEATABILITY_TRIALS} trials passed"
            )
        if not common_surfaces or not all(common_surfaces.values()):
            raise RuntimeError("Manifest V3 repeatability surfaces were incomplete")
        return 0
    finally:
        fixture_server.shutdown()
        fixture_server.server_close()
        fixture_thread.join(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
