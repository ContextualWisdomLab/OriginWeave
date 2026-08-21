#!/usr/bin/env python3
"""Run bounded repeatable real-browser evidence against pinned Chromium.

This is a release/CI evidence runner, not a product browser adapter. It uses the
W3C WebDriver HTTP protocol only to prove that a real Chrome for Testing build
can load the controlled MV3 fixture and repeatedly exercise service-worker,
content-script, storage, declarative-net-request, tabs, windows, scripting,
commands, side-panel, bookmarks, history, real browser-click, and
restart-persistence behavior. It also executes the controlled Agent Task fixture
with extensions disabled in a fresh profile, performs real WebDriver input and
click operations, verifies the observable post-condition, and proves profile
cleanup without treating page content as instruction or authority.
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
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"
AGENT_TASK_FIXTURE = ROOT / "tests/fixtures/agent_task_basic"
PINNED_CHROME_VERSION = "150.0.7871.129"
PINNED_CHROME_REVISION = "r1639810"
REPEATABILITY_TRIALS = 3
AGENT_TASK_REPEATABILITY_TRIALS = 3
AGENT_TASK_INPUT_VALUE = "originweave controlled input"
REQUEST_TIMEOUT_SECONDS = 5.0
STARTUP_TIMEOUT_SECONDS = 20.0
FIXTURE_TIMEOUT_SECONDS = 20.0
MAX_WEBDRIVER_RESPONSE_BYTES = 1_048_576
W3C_ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"
PATH_TOKEN_CHARACTERS = frozenset(string.ascii_letters + string.digits + "-_.")


class QuietFixtureHandler(http.server.SimpleHTTPRequestHandler):
    """Serve only the controlled local fixture without noisy access logging."""

    def log_message(self, _format: str, *args: object) -> None:
        """Suppress request logs because the fixture contains no diagnostic value."""


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


def _find_element(driver_port: int, session_id: str, selector: str) -> str:
    """Find one fixture element and return its validated ChromeDriver identifier."""

    found = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/element"),
        {"using": "css selector", "value": selector},
    )
    element = found.get("value", {})
    element_id = element.get(W3C_ELEMENT_KEY) if isinstance(element, dict) else None
    if not isinstance(element_id, str):
        raise RuntimeError("WebDriver did not return a W3C element identifier")
    return _path_token(element_id, "element identifier")


def _element_command_path(session_id: str, element_id: str, suffix: str) -> str:
    """Build a bounded WebDriver element command path from validated identifiers."""

    safe_element = _path_token(element_id, "element identifier")
    return _webdriver_path(session_id, f"/element/{safe_element}{suffix}")


def _wait_for_extension_evidence(
    driver_port: int,
    session_id: str,
    expected_storage_persistence: str,
) -> dict[str, str]:
    """Wait until every controlled MV3 fixture surface reports its expected result."""

    if expected_storage_persistence not in {"initialized", "persisted"}:
        raise ValueError("invalid storage persistence expectation")
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
  dnr: document.documentElement.dataset.originweaveDnr || "missing",
  tabs: document.documentElement.dataset.originweaveTabs || "missing",
  windows: document.documentElement.dataset.originweaveWindows || "missing",
  scripting: document.documentElement.dataset.originweaveScripting || "missing",
  scriptingExecuted:
    document.documentElement.dataset.originweaveScriptingExecuted || "missing",
  commands: document.documentElement.dataset.originweaveCommands || "missing",
  sidePanel: document.documentElement.dataset.originweaveSidePanel || "missing",
  bookmarks: document.documentElement.dataset.originweaveBookmarks || "missing",
  history: document.documentElement.dataset.originweaveHistory || "missing"
};
"""
    expected = {
        "content": "ready",
        "storage": "ready",
        "storagePersistence": expected_storage_persistence,
        "workerReply": "pong",
        "workerState": "installed",
        "dnr": "blocked",
        "tabs": "ready",
        "windows": "ready",
        "scripting": "ready",
        "scriptingExecuted": "ready",
        "commands": "ready",
        "sidePanel": "ready",
        "bookmarks": "ready",
        "history": "ready",
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
    raise RuntimeError(
        f"MV3 fixture did not converge: expected={expected!r}, observed={latest!r}"
    )


def _exercise_real_click(driver_port: int, session_id: str) -> str:
    """Use the WebDriver element-click command and verify the DOM post-condition."""

    safe_element = _find_element(driver_port, session_id, "#fixture-button")
    _json_request(
        driver_port,
        "POST",
        _element_command_path(session_id, safe_element, "/click"),
        {},
    )
    safe_output = _find_element(driver_port, session_id, "#fixture-output")
    text = _json_request(
        driver_port,
        "GET",
        _element_command_path(session_id, safe_output, "/text"),
    ).get("value")
    if text != "clicked":
        raise RuntimeError(f"real click post-condition failed: {text!r}")
    return str(text)


def _run_browser_pass(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    profile_dir: str,
    expected_storage_persistence: str,
) -> dict[str, Any]:
    """Run one fresh browser process against a shared bounded compatibility profile."""

    driver_port = _free_loopback_port()
    session_id: str | None = None
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
        )
        click_result = _exercise_real_click(driver_port, session_id)
        worker_start_count = int(surfaces["workerStartCount"])
        return {
            "browser_version": browser_version,
            "worker_start_count": worker_start_count,
            "storage_persistence": surfaces["storagePersistence"],
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
    """Run one independent initial/restart pair and return credential-free evidence."""

    trial_started = time.monotonic()
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-mv3-trial-{trial_number}-"
    ) as profile_dir:
        initial = _run_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
            "initialized",
        )
        restarted = _run_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
            "persisted",
        )

    initial_count = int(initial["worker_start_count"])
    restarted_count = int(restarted["worker_start_count"])
    surfaces = {
        name: bool(initial["surfaces"][name]) and bool(restarted["surfaces"][name])
        for name in initial["surfaces"]
    }
    surfaces.update(
        {
            "restart-persistence": restarted["storage_persistence"] == "persisted",
            "worker-start-count": restarted_count > initial_count,
            "storage-persistence": restarted["storage_persistence"] == "persisted",
        }
    )
    if not all(surfaces.values()):
        raise RuntimeError(f"compatibility surface failed in trial {trial_number}")

    return {
        "trial_number": trial_number,
        "passed": True,
        "browser_version": restarted["browser_version"],
        "surfaces": surfaces,
        "browser_passes": [
            {
                "phase": "initial",
                "worker_start_count": initial_count,
                "storage_persistence": initial["storage_persistence"],
            },
            {
                "phase": "restart",
                "worker_start_count": restarted_count,
                "storage_persistence": restarted["storage_persistence"],
            },
        ],
        "duration_ms": round((time.monotonic() - trial_started) * 1000),
    }


def _validate_agent_task_submitted_state(state: object) -> None:
    """Require the controlled submitted marker without echoing page-controlled data."""

    if state != "submitted":
        raise RuntimeError("Agent Task state post-condition failed")


def _cleanup_agent_task_browser_session(driver_port: int, session_id: str) -> None:
    """Delete one Agent Task WebDriver session without suppressing cleanup failures."""

    _json_request(
        driver_port,
        "DELETE",
        _webdriver_path(session_id, ""),
        {},
    )


def _run_agent_task_browser_pass(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    profile_dir: str,
) -> dict[str, Any]:
    """Execute one synthetic Agent Task through real WebDriver input in pinned Chrome."""

    started = time.monotonic()
    driver_port = _free_loopback_port()
    session_id: str | None = None
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
                                "--disable-extensions",
                                f"--user-data-dir={profile_dir}",
                            ],
                        },
                    }
                }
            },
        ).get("value", {})
        if not isinstance(session, dict):
            raise RuntimeError("ChromeDriver Agent Task session response is malformed")
        raw_session_id = session.get("sessionId")
        capabilities = session.get("capabilities", {})
        if not isinstance(raw_session_id, str):
            raise RuntimeError("ChromeDriver did not return an Agent Task session id")
        session_id = _path_token(raw_session_id, "session identifier")
        browser_version = (
            capabilities.get("browserVersion") if isinstance(capabilities, dict) else None
        )
        if browser_version != PINNED_CHROME_VERSION:
            raise RuntimeError(
                f"unexpected Agent Task Chrome version: expected {PINNED_CHROME_VERSION}, "
                f"got {browser_version!r}"
            )

        _json_request(
            driver_port,
            "POST",
            _webdriver_path(session_id, "/url"),
            {"url": fixture_url},
        )
        initial_url = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        ).get("value")
        if initial_url != fixture_url:
            raise RuntimeError(
                f"Agent Task initial URL mismatch: expected {fixture_url!r}, got {initial_url!r}"
            )
        input_element = _find_element(driver_port, session_id, "#task-text")
        _json_request(
            driver_port,
            "POST",
            _element_command_path(session_id, input_element, "/clear"),
            {},
        )
        _json_request(
            driver_port,
            "POST",
            _element_command_path(session_id, input_element, "/value"),
            {"text": AGENT_TASK_INPUT_VALUE, "value": list(AGENT_TASK_INPUT_VALUE)},
        )
        submit_element = _find_element(
            driver_port,
            session_id,
            "#agent-task-form button[type=submit]",
        )
        _json_request(
            driver_port,
            "POST",
            _element_command_path(session_id, submit_element, "/click"),
            {},
        )
        post_submit_url = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        ).get("value")
        url_unchanged = post_submit_url == initial_url
        if not url_unchanged:
            raise RuntimeError("Agent Task URL changed during submission")
        result_element = _find_element(driver_port, session_id, "#task-result")
        state = _json_request(
            driver_port,
            "GET",
            _element_command_path(session_id, result_element, "/attribute/data-state"),
        ).get("value")
        text = _json_request(
            driver_port,
            "GET",
            _element_command_path(session_id, result_element, "/text"),
        ).get("value")
        _validate_agent_task_submitted_state(state)
        if text != AGENT_TASK_INPUT_VALUE:
            raise RuntimeError("Agent Task result did not match the synthetic typed value")
        return {
            "browser_version": browser_version,
            "post_condition": True,
            "input_echo_verified": True,
            "url_unchanged": url_unchanged,
            "extensions_disabled": True,
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
    finally:
        try:
            if session_id is not None:
                _cleanup_agent_task_browser_session(driver_port, session_id)
        finally:
            driver.terminate()
            try:
                driver.wait(timeout=5)
            except subprocess.TimeoutExpired:
                driver.kill()
                driver.wait(timeout=5)


def _run_agent_task_trial(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    trial_number: int,
) -> dict[str, Any]:
    """Run one isolated Agent Task browser trial and prove its profile is removed."""

    trial_started = time.monotonic()
    profile_path: pathlib.Path
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-agent-task-trial-{trial_number}-"
    ) as profile_dir:
        profile_path = pathlib.Path(profile_dir)
        result = _run_agent_task_browser_pass(
            chrome_bin,
            chromedriver_bin,
            fixture_url,
            profile_dir,
        )
    profile_cleaned = not profile_path.exists()
    if not profile_cleaned:
        raise RuntimeError(f"Agent Task profile cleanup failed in trial {trial_number}")

    return {
        "trial_number": trial_number,
        "passed": True,
        "browser_version": result["browser_version"],
        "post_condition": result["post_condition"],
        "input_echo_verified": result["input_echo_verified"],
        "url_unchanged": result["url_unchanged"],
        "extensions_disabled": result["extensions_disabled"],
        "profile_cleaned": profile_cleaned,
        "duration_ms": round((time.monotonic() - trial_started) * 1000),
    }


def _start_fixture_server(directory: pathlib.Path) -> tuple[http.server.ThreadingHTTPServer, threading.Thread]:
    """Start one loopback-only static fixture server for a bounded browser lane."""

    server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0),
        lambda *args, **kwargs: QuietFixtureHandler(
            *args,
            directory=str(directory),
            **kwargs,
        ),
    )
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, thread


def _stop_fixture_server(
    server: http.server.ThreadingHTTPServer,
    thread: threading.Thread,
) -> None:
    """Stop one bounded fixture server and join its helper thread."""

    server.shutdown()
    server.server_close()
    thread.join(timeout=5)


def main() -> int:
    """Run bounded MV3 and Agent Task trials and emit credential-free evidence."""

    chrome_bin = pathlib.Path(os.environ.get("CHROME_BIN", ""))
    chromedriver_bin = pathlib.Path(os.environ.get("CHROMEDRIVER_BIN", ""))
    if not chrome_bin.is_file():
        raise SystemExit("CHROME_BIN must point to the pinned Chrome for Testing executable")
    if not chromedriver_bin.is_file():
        raise SystemExit("CHROMEDRIVER_BIN must point to the matching pinned ChromeDriver")
    if not (FIXTURE / "manifest.json").is_file():
        raise SystemExit("MV3 fixture manifest is missing")
    if not (AGENT_TASK_FIXTURE / "index.html").is_file():
        raise SystemExit("Agent Task fixture is missing")

    fixture_server, fixture_thread = _start_fixture_server(FIXTURE)
    agent_task_server, agent_task_thread = _start_fixture_server(AGENT_TASK_FIXTURE)
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
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError):
                trial_results.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                    }
                )

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

        agent_task_url = (
            f"http://127.0.0.1:{agent_task_server.server_port}/index.html"
        )
        agent_task_trials: list[dict[str, Any]] = []
        for trial_number in range(1, AGENT_TASK_REPEATABILITY_TRIALS + 1):
            try:
                agent_task_trials.append(
                    _run_agent_task_trial(
                        chrome_bin,
                        chromedriver_bin,
                        agent_task_url,
                        trial_number,
                    )
                )
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError):
                agent_task_trials.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                    }
                )

        agent_task_successful_trials = sum(
            1 for trial in agent_task_trials if trial.get("passed") is True
        )
        agent_task_trial_pass_rate = (
            agent_task_successful_trials / AGENT_TASK_REPEATABILITY_TRIALS
        )
        agent_task_surfaces_complete = all(
            trial.get("post_condition") is True
            and trial.get("input_echo_verified") is True
            and trial.get("url_unchanged") is True
            and trial.get("extensions_disabled") is True
            and trial.get("profile_cleaned") is True
            for trial in agent_task_trials
            if trial.get("passed") is True
        )

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
            "agent_task": {
                "repeatability_trials": AGENT_TASK_REPEATABILITY_TRIALS,
                "successful_trials": agent_task_successful_trials,
                "trial_pass_rate": agent_task_trial_pass_rate,
                "trial_results": agent_task_trials,
            },
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
        if agent_task_successful_trials != AGENT_TASK_REPEATABILITY_TRIALS:
            raise RuntimeError(
                "Agent Task repeatability gate failed: "
                f"{agent_task_successful_trials}/{AGENT_TASK_REPEATABILITY_TRIALS} "
                "trials passed"
            )
        if not agent_task_surfaces_complete:
            raise RuntimeError("Agent Task repeatability surfaces were incomplete")
        return 0
    finally:
        _stop_fixture_server(agent_task_server, agent_task_thread)
        _stop_fixture_server(fixture_server, fixture_thread)


if __name__ == "__main__":
    raise SystemExit(main())
