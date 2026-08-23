#!/usr/bin/env python3
"""Run bounded repeatable real-browser evidence against pinned Chromium.

This is a release/CI evidence runner, not a product browser adapter. It uses the
W3C WebDriver HTTP protocol only to prove that a real Chrome for Testing build
can load the controlled MV3 fixture and repeatedly exercise service-worker,
content-script, storage, declarative-net-request, tabs, windows, scripting,
commands, side-panel, bookmarks, history, real browser-click, and
restart-persistence behavior. It also executes the controlled Agent Task fixture
with extensions disabled in a fresh profile, locates the controlled action
targets by exact browser-computed role/name evidence, performs real WebDriver
input and click operations, verifies the observable post-condition, proves the
controlled action preserves its loaded URL, and records bounded runtime resource
evidence without treating page content as instruction or authority.
"""

from __future__ import annotations

import contextlib
import hashlib
import http.client
import http.server
import json
import math
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
PROCESS_EXIT_TIMEOUT_SECONDS = 5.0
MAX_WEBDRIVER_RESPONSE_BYTES = 1_048_576
MAX_PROC_STATUS_CHARACTERS = 65_536
MAX_PROC_STAT_CHARACTERS = 65_536
MAX_BROWSER_PROCESS_TREE_SIZE = 256
MAX_PROC_PROCESS_SCAN_SIZE = 32_768
MAX_SEMANTIC_LOCATOR_CANDIDATES = 128
MAX_AGENT_TASK_STRUCTURED_VALUE_BYTES = 4_096
MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES = 4_096
MAX_U64 = (1 << 64) - 1
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
    """Build a bounded ChromeDriver path from a validated session identifier."""

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


def _get_element_semantics(
    driver_port: int,
    session_id: str,
    element_id: str,
) -> tuple[str, str]:
    """Read one controlled element's browser-computed role and accessible name."""

    role = _json_request(
        driver_port,
        "GET",
        _element_command_path(session_id, element_id, "/computedrole"),
    ).get("value")
    label = _json_request(
        driver_port,
        "GET",
        _element_command_path(session_id, element_id, "/computedlabel"),
    ).get("value")
    if not isinstance(role, str) or not isinstance(label, str):
        raise RuntimeError("WebDriver returned malformed element semantics")
    return role, label


def _find_element_by_accessible_role_name(
    driver_port: int,
    session_id: str,
    role: str,
    accessible_name: str,
) -> str:
    """Find exactly one controlled element by browser-computed role and name."""

    found = _json_request(
        driver_port,
        "POST",
        _webdriver_path(session_id, "/elements"),
        {"using": "css selector", "value": "*"},
    )
    elements = found.get("value")
    if not isinstance(elements, list):
        raise RuntimeError("WebDriver did not return a semantic locator candidate list")
    if len(elements) > MAX_SEMANTIC_LOCATOR_CANDIDATES:
        raise RuntimeError("semantic locator exceeded bounded candidate limit")

    matches: list[str] = []
    for element in elements:
        element_id = element.get(W3C_ELEMENT_KEY) if isinstance(element, dict) else None
        if not isinstance(element_id, str):
            raise RuntimeError("WebDriver returned malformed semantic locator candidate")
        safe_element = _path_token(element_id, "element identifier")
        candidate_role, candidate_name = _get_element_semantics(
            driver_port,
            session_id,
            safe_element,
        )
        if candidate_role == role and candidate_name == accessible_name:
            matches.append(safe_element)
            if len(matches) > 1:
                raise RuntimeError("semantic locator returned multiple exact matches")

    if not matches:
        raise RuntimeError("semantic locator returned no exact match")
    return matches[0]


def _hash_agent_task_structured_value(value: str) -> str:
    """Hash one bounded extracted text value without retaining the raw value in evidence."""

    if not isinstance(value, str):
        raise TypeError("Agent Task structured value must be text")
    encoded = value.encode("utf-8")
    if not encoded:
        raise ValueError("Agent Task structured value must not be empty")
    if len(encoded) > MAX_AGENT_TASK_STRUCTURED_VALUE_BYTES:
        raise ValueError("Agent Task structured value exceeded the bounded text contract")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def _measure_agent_task_semantic_observation_bytes(observation: dict[str, Any]) -> int:
    """Measure one non-empty semantic observation under the canonical evidence bound."""

    if not isinstance(observation, dict):
        raise TypeError("Agent Task semantic observation must be an object")
    if not observation:
        raise ValueError("Agent Task semantic observation must not be empty")
    encoded = json.dumps(
        observation,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    if len(encoded) > MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES:
        raise ValueError("Agent Task semantic observation exceeded the bounded evidence contract")
    return len(encoded)


def _require_pristine_agent_task_profile(profile_dir: str) -> None:
    """Fail closed unless the controlled Agent Task profile directory is empty."""

    profile_path = pathlib.Path(profile_dir)
    if not profile_path.is_dir() or any(profile_path.iterdir()):
        raise RuntimeError("Agent Task profile is not pristine before launch")


def _probe_agent_task_ambient_state(driver_port: int, session_id: str) -> dict[str, bool]:
    """Require no browser-visible cookies or Web Storage before the controlled action."""

    cookies = _json_request(
        driver_port,
        "GET",
        _webdriver_path(session_id, "/cookie"),
    ).get("value")
    if not isinstance(cookies, list):
        raise RuntimeError("Agent Task cookie inspection returned malformed evidence")
    if cookies:
        raise RuntimeError("Agent Task profile exposed ambient cookies")

    storage = _execute(
        driver_port,
        session_id,
        """
return {
  localStorageLength: window.localStorage.length,
  sessionStorageLength: window.sessionStorage.length
};
""",
    )
    if not isinstance(storage, dict):
        raise RuntimeError("Agent Task Web Storage inspection returned malformed evidence")
    local_storage_length = storage.get("localStorageLength")
    session_storage_length = storage.get("sessionStorageLength")
    for value in (local_storage_length, session_storage_length):
        if isinstance(value, bool) or not isinstance(value, int) or value < 0:
            raise RuntimeError("Agent Task Web Storage inspection returned malformed evidence")
    if local_storage_length or session_storage_length:
        raise RuntimeError("Agent Task profile exposed ambient Web Storage")
    return {
        "ambient_cookies_absent": True,
        "ambient_web_storage_absent": True,
    }


def _parse_linux_proc_status_rss_bytes(status_text: str) -> int:
    """Parse exactly one positive Linux ``VmRSS`` kB field into bounded bytes."""

    rss_values: list[int] = []
    for line in status_text.splitlines():
        if not line.startswith("VmRSS:"):
            continue
        fields = line.split()
        if len(fields) != 3 or fields[0] != "VmRSS:" or fields[2] != "kB":
            raise ValueError("malformed Linux VmRSS field")
        raw_kibibytes = fields[1]
        if not raw_kibibytes.isascii() or not raw_kibibytes.isdigit():
            raise ValueError("malformed Linux VmRSS value")
        kibibytes = int(raw_kibibytes, 10)
        if kibibytes <= 0:
            raise ValueError("Linux VmRSS must be positive")
        if kibibytes > MAX_U64 // 1024:
            raise OverflowError("Linux VmRSS exceeds u64 byte range")
        rss_values.append(kibibytes * 1024)
    if len(rss_values) != 1:
        raise ValueError("Linux proc status must contain exactly one VmRSS field")
    return rss_values[0]


def _parse_linux_proc_status_optional_rss_bytes(status_text: str) -> int | None:
    """Parse optional Linux ``VmRSS`` without normalizing malformed evidence."""

    rss_lines = [line for line in status_text.splitlines() if line.startswith("VmRSS:")]
    if not rss_lines:
        return None
    if len(rss_lines) != 1:
        raise ValueError("Linux proc status must contain at most one VmRSS field")

    fields = rss_lines[0].split()
    if len(fields) != 3 or fields[0] != "VmRSS:" or fields[2] != "kB":
        raise ValueError("malformed Linux VmRSS field")
    raw_kibibytes = fields[1]
    if not raw_kibibytes.isascii() or not raw_kibibytes.isdigit():
        raise ValueError("malformed Linux VmRSS value")
    kibibytes = int(raw_kibibytes, 10)
    if kibibytes == 0:
        return None
    if kibibytes > MAX_U64 // 1024:
        raise OverflowError("Linux VmRSS exceeds u64 byte range")
    return kibibytes * 1024


def _parse_linux_proc_status_process_identity(status_text: str) -> tuple[int, int]:
    """Parse exactly one positive ``Pid`` and one non-negative ``PPid`` from status."""

    parsed: dict[str, int] = {}
    for line in status_text.splitlines():
        if not (line.startswith("Pid:") or line.startswith("PPid:")):
            continue
        fields = line.split()
        if len(fields) != 2 or fields[0] not in {"Pid:", "PPid:"}:
            raise ValueError("malformed Linux process identity field")
        label = fields[0]
        if label in parsed:
            raise ValueError("duplicate Linux process identity field")
        raw_process_id = fields[1]
        if not raw_process_id.isascii() or not raw_process_id.isdigit():
            raise ValueError("malformed Linux process identity value")
        parsed[label] = int(raw_process_id, 10)

    if set(parsed) != {"Pid:", "PPid:"}:
        raise ValueError("Linux proc status must contain exactly one Pid and PPid")
    process_id = parsed["Pid:"]
    parent_process_id = parsed["PPid:"]
    if process_id <= 0:
        raise ValueError("Linux process identifier must be positive")
    if parent_process_id < 0:
        raise ValueError("Linux parent process identifier must be non-negative")
    return process_id, parent_process_id


def _parse_linux_proc_stat_process_identity(stat_text: str) -> tuple[int, int]:
    """Parse one Linux proc-stat PID/start-time identity without trusting ``comm`` text."""

    if not isinstance(stat_text, str) or not stat_text:
        raise ValueError("Linux proc stat must be non-empty text")
    command_open = stat_text.find(" (")
    command_close = stat_text.rfind(") ")
    if command_open <= 0 or command_close <= command_open + 2:
        raise ValueError("malformed Linux proc stat process identity")

    raw_process_id = stat_text[:command_open]
    if not raw_process_id.isascii() or not raw_process_id.isdigit():
        raise ValueError("malformed Linux proc stat process identifier")
    process_id = int(raw_process_id, 10)
    if process_id <= 0:
        raise ValueError("Linux proc stat process identifier must be positive")

    command_text = stat_text[command_open + 2 : command_close]
    if not command_text:
        raise ValueError("Linux proc stat command must not be empty")
    suffix_fields = stat_text[command_close + 2 :].split()
    if len(suffix_fields) < 20 or len(suffix_fields[0]) != 1:
        raise ValueError("Linux proc stat does not contain field 22 start time")
    for raw_field in suffix_fields[1:]:
        unsigned_field = raw_field[1:] if raw_field[:1] in {"+", "-"} else raw_field
        if not unsigned_field or not unsigned_field.isascii() or not unsigned_field.isdigit():
            raise ValueError("malformed Linux proc stat numeric field")

    raw_start_time_ticks = suffix_fields[19]
    if not raw_start_time_ticks.isascii() or not raw_start_time_ticks.isdigit():
        raise ValueError("malformed Linux proc stat start time")
    start_time_ticks = int(raw_start_time_ticks, 10)
    if start_time_ticks <= 0:
        raise ValueError("Linux proc stat start time must be positive")
    if start_time_ticks > MAX_U64:
        raise OverflowError("Linux proc stat start time exceeds u64 range")
    return process_id, start_time_ticks


def _read_linux_proc_stat_process_identity(process_id: int) -> tuple[int, int] | None:
    """Read one bounded Linux PID/start-time identity, returning absence after exit."""

    if isinstance(process_id, bool) or not isinstance(process_id, int) or process_id <= 0:
        raise ValueError("invalid Linux process identifier")
    stat_path = pathlib.Path("/proc") / str(process_id) / "stat"
    try:
        with stat_path.open("r", encoding="utf-8", errors="strict") as stat_file:
            stat_text = stat_file.read(MAX_PROC_STAT_CHARACTERS + 1)
    except FileNotFoundError:
        return None
    if len(stat_text) > MAX_PROC_STAT_CHARACTERS:
        raise RuntimeError("Linux proc stat exceeded the bounded text limit")
    identity = _parse_linux_proc_stat_process_identity(stat_text)
    if identity[0] != process_id:
        raise RuntimeError("Linux proc stat identity did not match its directory")
    return identity


def _wait_for_linux_process_identity_exit(
    process_id: int,
    start_time_ticks: int,
    *,
    timeout_seconds: float = PROCESS_EXIT_TIMEOUT_SECONDS,
) -> bool:
    """Wait boundedly until the exact PID/start-time identity exits or is reused."""

    if isinstance(process_id, bool) or not isinstance(process_id, int) or process_id <= 0:
        raise ValueError("invalid Linux process identifier")
    if (
        isinstance(start_time_ticks, bool)
        or not isinstance(start_time_ticks, int)
        or start_time_ticks <= 0
    ):
        raise ValueError("invalid Linux process start time")
    if (
        isinstance(timeout_seconds, bool)
        or not isinstance(timeout_seconds, (int, float))
        or timeout_seconds < 0
        or not math.isfinite(timeout_seconds)
    ):
        raise ValueError("invalid Linux process-exit timeout")

    deadline = time.monotonic() + float(timeout_seconds)
    expected_identity = (process_id, start_time_ticks)
    while True:
        current_identity = _read_linux_proc_stat_process_identity(process_id)
        if current_identity is None or current_identity != expected_identity:
            return True
        remaining_seconds = deadline - time.monotonic()
        if remaining_seconds <= 0:
            return False
        time.sleep(min(0.05, remaining_seconds))


def _sample_linux_process_rss_bytes(process_id: int) -> int:
    """Read one attributed Linux process RSS through a bounded ``/proc`` status file."""

    if isinstance(process_id, bool) or not isinstance(process_id, int) or process_id <= 0:
        raise ValueError("invalid Linux process identifier")
    status_path = pathlib.Path("/proc") / str(process_id) / "status"
    with status_path.open("r", encoding="utf-8", errors="strict") as status_file:
        status_text = status_file.read(MAX_PROC_STATUS_CHARACTERS + 1)
    if len(status_text) > MAX_PROC_STATUS_CHARACTERS:
        raise RuntimeError("Linux proc status exceeded the bounded text limit")
    return _parse_linux_proc_status_rss_bytes(status_text)


def _snapshot_linux_process_evidence() -> dict[int, tuple[int, int | None]]:
    """Capture one bounded best-effort PID/PPID/RSS sweep from Linux proc status."""

    proc_root = pathlib.Path("/proc")
    process_entries: list[tuple[int, pathlib.Path]] = []
    for entry in proc_root.iterdir():
        raw_process_id = entry.name
        if not raw_process_id.isascii() or not raw_process_id.isdigit():
            continue
        process_id = int(raw_process_id, 10)
        if process_id <= 0:
            continue
        process_entries.append((process_id, entry))
        if len(process_entries) > MAX_PROC_PROCESS_SCAN_SIZE:
            raise RuntimeError("Linux proc process scan exceeded the bounded entry limit")

    process_evidence: dict[int, tuple[int, int | None]] = {}
    for expected_process_id, entry in sorted(process_entries):
        status_path = entry / "status"
        try:
            with status_path.open("r", encoding="utf-8", errors="strict") as status_file:
                status_text = status_file.read(MAX_PROC_STATUS_CHARACTERS + 1)
        except FileNotFoundError:
            continue
        if len(status_text) > MAX_PROC_STATUS_CHARACTERS:
            raise RuntimeError("Linux proc status exceeded the bounded text limit")
        process_id, parent_process_id = _parse_linux_proc_status_process_identity(
            status_text
        )
        if process_id != expected_process_id:
            raise RuntimeError("Linux proc status identity did not match its directory")
        if process_id in process_evidence:
            raise RuntimeError("Linux proc process snapshot contained a duplicate PID")
        rss_bytes = _parse_linux_proc_status_optional_rss_bytes(status_text)
        process_evidence[process_id] = (parent_process_id, rss_bytes)
    return process_evidence


def _discover_linux_process_tree_ids(
    root_process_id: int,
    process_evidence: dict[int, tuple[int, int | None]],
) -> tuple[int, ...]:
    """Discover one bounded root-plus-descendant set from sampled process evidence."""

    if (
        isinstance(root_process_id, bool)
        or not isinstance(root_process_id, int)
        or root_process_id <= 0
    ):
        raise ValueError("invalid Linux root process identifier")
    if root_process_id not in process_evidence:
        raise RuntimeError("Linux process snapshot did not contain the browser root PID")

    discovered = [root_process_id]
    known = {root_process_id}
    while True:
        children = sorted(
            process_id
            for process_id, (parent_process_id, _rss_bytes) in process_evidence.items()
            if parent_process_id in known and process_id not in known
        )
        if not children:
            break
        for process_id in children:
            if len(known) >= MAX_BROWSER_PROCESS_TREE_SIZE:
                raise ValueError("Linux process tree exceeded the bounded process-tree size")
            known.add(process_id)
            discovered.append(process_id)
    return tuple(discovered)


def _sample_linux_process_set_rss_bytes(
    process_ids: tuple[int, ...],
    process_evidence: dict[int, tuple[int, int | None]],
) -> int:
    """Sum resident RSS for one exact bounded process set without overflow."""

    if not process_ids or len(process_ids) > MAX_BROWSER_PROCESS_TREE_SIZE:
        raise ValueError("invalid Linux process set size")
    if len(set(process_ids)) != len(process_ids):
        raise ValueError("Linux process set identifiers must be unique")

    total_rss_bytes = 0
    for process_id in process_ids:
        if isinstance(process_id, bool) or not isinstance(process_id, int) or process_id <= 0:
            raise ValueError("invalid Linux process identifier")
        if process_id not in process_evidence:
            raise ValueError("Linux process set was not present in the sampled evidence")
        rss_bytes = process_evidence[process_id][1]
        if rss_bytes is None:
            continue
        if isinstance(rss_bytes, bool) or not isinstance(rss_bytes, int) or rss_bytes <= 0:
            raise ValueError("Linux process set contained invalid sampled RSS")
        if rss_bytes > MAX_U64 - total_rss_bytes:
            raise OverflowError("Linux process-set RSS exceeds u64 byte range")
        total_rss_bytes += rss_bytes
    return total_rss_bytes


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
    """Run one independent initial/restart pair and retain cleanup evidence on failure."""

    trial_started = time.monotonic()
    profile_path: pathlib.Path
    initial: dict[str, Any] | None = None
    restarted: dict[str, Any] | None = None
    failure_type: str | None = None
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-mv3-trial-{trial_number}-"
    ) as profile_dir:
        profile_path = pathlib.Path(profile_dir)
        try:
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
        except (
            OSError,
            ValueError,
            RuntimeError,
            json.JSONDecodeError,
            subprocess.TimeoutExpired,
        ) as exc:
            failure_type = type(exc).__name__
    profile_cleaned = not profile_path.exists()
    if not profile_cleaned:
        raise RuntimeError(f"Manifest V3 profile cleanup failed in trial {trial_number}")

    duration_ms = round((time.monotonic() - trial_started) * 1000)
    if failure_type is not None:
        return {
            "trial_number": trial_number,
            "passed": False,
            "failure_type": failure_type,
            "profile_cleaned": True,
            "duration_ms": duration_ms,
        }
    if initial is None or restarted is None:
        raise RuntimeError("Manifest V3 restart trial returned incomplete browser evidence")

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
        "profile_cleaned": True,
        "duration_ms": duration_ms,
    }


def _run_agent_task_browser_pass(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    profile_dir: str,
) -> dict[str, Any]:
    """Execute one synthetic Agent Task and measure bounded real-browser evidence."""

    _require_pristine_agent_task_profile(profile_dir)
    profile_pristine_before_launch = True
    started = time.monotonic()
    driver_port = _free_loopback_port()
    session_id: str | None = None
    browser_process_id: int | None = None
    browser_process_start_time_ticks: int | None = None
    result: dict[str, Any] | None = None
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
                            "prefs": {
                                "credentials_enable_service": False,
                                "profile.password_manager_enabled": False,
                            },
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
        if not isinstance(capabilities, dict):
            raise RuntimeError("ChromeDriver Agent Task capabilities are malformed")
        session_id = _path_token(raw_session_id, "session identifier")
        browser_version = capabilities.get("browserVersion")
        browser_process_id = capabilities.get("goog:processID")
        if browser_version != PINNED_CHROME_VERSION:
            raise RuntimeError(
                f"unexpected Agent Task Chrome version: expected {PINNED_CHROME_VERSION}, "
                f"got {browser_version!r}"
            )
        if (
            isinstance(browser_process_id, bool)
            or not isinstance(browser_process_id, int)
            or browser_process_id <= 0
        ):
            raise RuntimeError("ChromeDriver did not return a valid browser process id")
        browser_process_identity = _read_linux_proc_stat_process_identity(browser_process_id)
        if browser_process_identity is None:
            raise RuntimeError("Agent Task browser process identity disappeared after launch")
        browser_process_start_time_ticks = browser_process_identity[1]

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
            raise RuntimeError("Agent Task did not load the requested fixture URL")
        ambient_state = _probe_agent_task_ambient_state(driver_port, session_id)

        input_element = _find_element_by_accessible_role_name(
            driver_port,
            session_id,
            "textbox",
            "Task text",
        )
        input_role, input_name = _get_element_semantics(
            driver_port,
            session_id,
            input_element,
        )
        if input_role != "textbox" or input_name != "Task text":
            raise RuntimeError("Agent Task input semantic evidence mismatch")
        submit_element = _find_element_by_accessible_role_name(
            driver_port,
            session_id,
            "button",
            "Submit task",
        )
        submit_role, submit_name = _get_element_semantics(
            driver_port,
            session_id,
            submit_element,
        )
        if submit_role != "button" or submit_name != "Submit task":
            raise RuntimeError("Agent Task submit semantic evidence mismatch")
        semantic_observation = {
            "input": {"role": input_role, "name": input_name},
            "submit": {"role": submit_role, "name": submit_name},
        }
        semantic_observation_bytes = _measure_agent_task_semantic_observation_bytes(
            semantic_observation
        )

        action_started = time.monotonic()
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
        _json_request(
            driver_port,
            "POST",
            _element_command_path(session_id, submit_element, "/click"),
            {},
        )
        action_latency_ms = round((time.monotonic() - action_started) * 1000, 3)
        if action_latency_ms <= 0:
            raise RuntimeError("Agent Task measured a non-positive action latency")

        post_submit_url = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        ).get("value")
        url_unchanged = post_submit_url == initial_url
        if not url_unchanged:
            raise RuntimeError("Agent Task URL changed during submission")

        result_element = _find_element_by_accessible_role_name(
            driver_port,
            session_id,
            "status",
            "Task result",
        )
        result_role, result_name = _get_element_semantics(
            driver_port,
            session_id,
            result_element,
        )
        if result_role != "status" or result_name != "Task result":
            raise RuntimeError("Agent Task result semantic evidence mismatch")
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
        if state != "submitted":
            raise RuntimeError(f"Agent Task state post-condition failed: {state!r}")
        if text != AGENT_TASK_INPUT_VALUE:
            raise RuntimeError("Agent Task result did not match the synthetic typed value")
        structured_value_sha256 = _hash_agent_task_structured_value(text)

        process_evidence = _snapshot_linux_process_evidence()
        chromium_process_ids = _discover_linux_process_tree_ids(
            browser_process_id,
            process_evidence,
        )
        browser_process_rss_bytes = _sample_linux_process_rss_bytes(browser_process_id)
        chromium_process_set_rss_bytes = _sample_linux_process_set_rss_bytes(
            chromium_process_ids,
            process_evidence,
        )
        chromium_process_count = len(chromium_process_ids)
        task_duration_ms = round((time.monotonic() - started) * 1000, 3)
        if task_duration_ms <= 0:
            raise RuntimeError("Agent Task measured a non-positive task duration")
        result = {
            "browser_version": browser_version,
            "post_condition": True,
            "input_echo_verified": True,
            "url_unchanged": url_unchanged,
            "input_semantics_verified": True,
            "submit_semantics_verified": True,
            "result_semantics_verified": True,
            "structured_value_field": "task_result",
            "structured_value_sha256": structured_value_sha256,
            "extensions_disabled": True,
            "profile_pristine_before_launch": profile_pristine_before_launch,
            "ambient_cookies_absent": ambient_state["ambient_cookies_absent"],
            "ambient_web_storage_absent": ambient_state["ambient_web_storage_absent"],
            "saved_credential_services_disabled": True,
            "browser_process_rss_bytes": browser_process_rss_bytes,
            "chromium_process_count": chromium_process_count,
            "chromium_process_set_rss_bytes": chromium_process_set_rss_bytes,
            "semantic_observation_bytes": semantic_observation_bytes,
            "action_latency_ms": action_latency_ms,
            "task_duration_ms": task_duration_ms,
            "duration_ms": round(task_duration_ms),
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

    if result is None:
        raise RuntimeError("Agent Task browser pass returned no result after shutdown")
    if browser_process_id is None or browser_process_start_time_ticks is None:
        raise RuntimeError("Agent Task browser process identity was not captured")
    if not _wait_for_linux_process_identity_exit(
        browser_process_id,
        browser_process_start_time_ticks,
    ):
        raise RuntimeError("Agent Task browser process did not terminate")
    result["browser_process_terminated"] = True
    return result


def _run_agent_task_trial(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    trial_number: int,
) -> dict[str, Any]:
    """Run one isolated Agent Task trial and retain cleanup evidence on failure."""

    trial_started = time.monotonic()
    profile_path: pathlib.Path
    result: dict[str, Any] | None = None
    failure_type: str | None = None
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-agent-task-trial-{trial_number}-"
    ) as profile_dir:
        profile_path = pathlib.Path(profile_dir)
        try:
            result = _run_agent_task_browser_pass(
                chrome_bin,
                chromedriver_bin,
                fixture_url,
                profile_dir,
            )
        except (
            OSError,
            ValueError,
            RuntimeError,
            json.JSONDecodeError,
            subprocess.TimeoutExpired,
        ) as exc:
            failure_type = type(exc).__name__
    profile_cleaned = not profile_path.exists()
    if not profile_cleaned:
        raise RuntimeError(f"Agent Task profile cleanup failed in trial {trial_number}")

    duration_ms = round((time.monotonic() - trial_started) * 1000)
    if failure_type is not None:
        return {
            "trial_number": trial_number,
            "passed": False,
            "failure_type": failure_type,
            "profile_cleaned": True,
            "duration_ms": duration_ms,
        }
    if result is None:
        raise RuntimeError("Agent Task browser pass returned no result")

    return {
        "trial_number": trial_number,
        "passed": True,
        "browser_version": result["browser_version"],
        "post_condition": result["post_condition"],
        "input_echo_verified": result["input_echo_verified"],
        "url_unchanged": result["url_unchanged"],
        "input_semantics_verified": result["input_semantics_verified"],
        "submit_semantics_verified": result["submit_semantics_verified"],
        "result_semantics_verified": result["result_semantics_verified"],
        "structured_value_field": result["structured_value_field"],
        "structured_value_sha256": result["structured_value_sha256"],
        "extensions_disabled": result["extensions_disabled"],
        "profile_pristine_before_launch": result["profile_pristine_before_launch"],
        "ambient_cookies_absent": result["ambient_cookies_absent"],
        "ambient_web_storage_absent": result["ambient_web_storage_absent"],
        "saved_credential_services_disabled": result[
            "saved_credential_services_disabled"
        ],
        "browser_process_rss_bytes": result["browser_process_rss_bytes"],
        "browser_process_terminated": result["browser_process_terminated"],
        "chromium_process_count": result["chromium_process_count"],
        "chromium_process_set_rss_bytes": result["chromium_process_set_rss_bytes"],
        "semantic_observation_bytes": result["semantic_observation_bytes"],
        "action_latency_ms": result["action_latency_ms"],
        "task_duration_ms": result["task_duration_ms"],
        "profile_cleaned": True,
        "duration_ms": duration_ms,
    }


def _is_no_such_window_runtime_error(error: RuntimeError) -> bool:
    """Recognize only structured ChromeDriver no-such-window failure evidence."""

    message = str(error)
    direct_prefix = "WebDriver error: "
    if message.startswith(direct_prefix):
        code, separator, _detail = message[len(direct_prefix) :].partition(":")
        return bool(separator) and code.strip().casefold() == "no such window"

    http_prefix = "WebDriver HTTP 404: "
    if not message.startswith(http_prefix):
        return False
    try:
        payload = json.loads(message[len(http_prefix) :])
    except json.JSONDecodeError:
        return False
    if not isinstance(payload, dict):
        return False
    value = payload.get("value")
    return isinstance(value, dict) and value.get("error") == "no such window"


def _force_close_agent_task_context(driver_port: int, session_id: str) -> bool:
    """Close only the current browsing context and require no-such-window evidence."""

    closed = _json_request(
        driver_port,
        "DELETE",
        _webdriver_path(session_id, "/window"),
    )
    surviving_contexts = closed.get("value")
    if not isinstance(surviving_contexts, list):
        raise RuntimeError("Agent Task forced-close returned malformed surviving contexts")
    if not surviving_contexts:
        raise RuntimeError("Agent Task forced-close left no surviving browsing context")
    if any(not isinstance(handle, str) or not handle for handle in surviving_contexts):
        raise RuntimeError("Agent Task forced-close returned invalid surviving context")

    try:
        _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        )
    except RuntimeError as exc:
        if _is_no_such_window_runtime_error(exc):
            return True
        raise
    raise RuntimeError("Agent Task context remained usable after forced close")


def _run_agent_task_forced_close_browser_pass(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    profile_dir: str,
) -> dict[str, Any]:
    """Force-close a disposable real context while preserving the WebDriver session."""

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
            raise RuntimeError("ChromeDriver forced-close session response is malformed")
        raw_session_id = session.get("sessionId")
        capabilities = session.get("capabilities", {})
        if not isinstance(raw_session_id, str):
            raise RuntimeError("ChromeDriver did not return a forced-close session id")
        if not isinstance(capabilities, dict):
            raise RuntimeError("ChromeDriver forced-close capabilities are malformed")
        session_id = _path_token(raw_session_id, "session identifier")
        browser_version = capabilities.get("browserVersion")
        if browser_version != PINNED_CHROME_VERSION:
            raise RuntimeError(
                f"unexpected forced-close Chrome version: expected {PINNED_CHROME_VERSION}, "
                f"got {browser_version!r}"
            )

        survivor_context = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/window"),
        ).get("value")
        if not isinstance(survivor_context, str):
            raise RuntimeError("ChromeDriver did not return the survivor context handle")
        survivor_context = _path_token(survivor_context, "window handle")

        created = _json_request(
            driver_port,
            "POST",
            _webdriver_path(session_id, "/window/new"),
            {"type": "tab"},
        ).get("value", {})
        if not isinstance(created, dict):
            raise RuntimeError("ChromeDriver returned malformed disposable context evidence")
        disposable_context = created.get("handle")
        if not isinstance(disposable_context, str):
            raise RuntimeError("ChromeDriver did not return a disposable context handle")
        disposable_context = _path_token(disposable_context, "window handle")
        if disposable_context == survivor_context:
            raise RuntimeError("ChromeDriver reused the survivor context as disposable context")

        _json_request(
            driver_port,
            "POST",
            _webdriver_path(session_id, "/window"),
            {"handle": disposable_context},
        )
        _json_request(
            driver_port,
            "POST",
            _webdriver_path(session_id, "/url"),
            {"url": fixture_url},
        )
        loaded_url = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        ).get("value")
        if loaded_url != fixture_url:
            raise RuntimeError("Agent Task forced-close probe did not load its fixture URL")

        forced_close_detected = _force_close_agent_task_context(driver_port, session_id)
        if not forced_close_detected:
            raise RuntimeError("Agent Task forced-close probe did not detect the close")

        _json_request(
            driver_port,
            "POST",
            _webdriver_path(session_id, "/window"),
            {"handle": survivor_context},
        )
        surviving_url = _json_request(
            driver_port,
            "GET",
            _webdriver_path(session_id, "/url"),
        ).get("value")
        if not isinstance(surviving_url, str):
            raise RuntimeError("Agent Task survivor context was not usable after forced close")

        return {
            "browser_version": browser_version,
            "forced_close_detected": forced_close_detected,
            "session_survived": True,
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


def _run_agent_task_forced_close_trial(
    chrome_bin: pathlib.Path,
    chromedriver_bin: pathlib.Path,
    fixture_url: str,
    trial_number: int,
) -> dict[str, Any]:
    """Run one forced-close trial and retain cleanup evidence on failure."""

    trial_started = time.monotonic()
    profile_path: pathlib.Path
    result: dict[str, Any] | None = None
    failure_type: str | None = None
    with tempfile.TemporaryDirectory(
        prefix=f"originweave-agent-task-forced-close-{trial_number}-"
    ) as profile_dir:
        profile_path = pathlib.Path(profile_dir)
        try:
            result = _run_agent_task_forced_close_browser_pass(
                chrome_bin,
                chromedriver_bin,
                fixture_url,
                profile_dir,
            )
        except (
            OSError,
            ValueError,
            RuntimeError,
            json.JSONDecodeError,
            subprocess.TimeoutExpired,
        ) as exc:
            failure_type = type(exc).__name__
    profile_cleaned = not profile_path.exists()
    if not profile_cleaned:
        raise RuntimeError(
            f"Agent Task forced-close profile cleanup failed in trial {trial_number}"
        )

    duration_ms = round((time.monotonic() - trial_started) * 1000)
    if failure_type is not None:
        return {
            "trial_number": trial_number,
            "passed": False,
            "failure_type": failure_type,
            "profile_cleaned": True,
            "duration_ms": duration_ms,
        }
    if result is None:
        raise RuntimeError("Agent Task forced-close browser pass returned no result")

    return {
        "trial_number": trial_number,
        "passed": True,
        "browser_version": result["browser_version"],
        "forced_close_detected": result["forced_close_detected"],
        "session_survived": result["session_survived"],
        "profile_cleaned": True,
        "duration_ms": duration_ms,
    }


def _start_fixture_server(
    directory: pathlib.Path,
) -> tuple[http.server.ThreadingHTTPServer, threading.Thread]:
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
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                trial_results.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                        "failure_type": type(exc).__name__,
                    }
                )

        successful_trials = sum(
            1 for trial in trial_results if trial.get("passed") is True
        )
        trial_pass_rate = successful_trials / REPEATABILITY_TRIALS
        mv3_profiles_cleaned = all(
            trial.get("profile_cleaned") is True for trial in trial_results
        )
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
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                agent_task_trials.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                        "failure_type": type(exc).__name__,
                    }
                )

        forced_close_trials: list[dict[str, Any]] = []
        for trial_number in range(1, AGENT_TASK_REPEATABILITY_TRIALS + 1):
            try:
                forced_close_trials.append(
                    _run_agent_task_forced_close_trial(
                        chrome_bin,
                        chromedriver_bin,
                        agent_task_url,
                        trial_number,
                    )
                )
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                forced_close_trials.append(
                    {
                        "trial_number": trial_number,
                        "passed": False,
                        "failure_type": type(exc).__name__,
                    }
                )

        agent_task_successful_trials = sum(
            1 for trial in agent_task_trials if trial.get("passed") is True
        )
        agent_task_trial_pass_rate = (
            agent_task_successful_trials / AGENT_TASK_REPEATABILITY_TRIALS
        )
        agent_task_profiles_cleaned = all(
            trial.get("profile_cleaned") is True for trial in agent_task_trials
        )
        agent_task_isolation_complete = all(
            trial.get("profile_pristine_before_launch") is True
            and trial.get("ambient_cookies_absent") is True
            and trial.get("ambient_web_storage_absent") is True
            and trial.get("saved_credential_services_disabled") is True
            and trial.get("extensions_disabled") is True
            and trial.get("profile_cleaned") is True
            for trial in agent_task_trials
            if trial.get("passed") is True
        )
        agent_task_surfaces_complete = all(
            trial.get("post_condition") is True
            and trial.get("input_echo_verified") is True
            and trial.get("url_unchanged") is True
            and trial.get("input_semantics_verified") is True
            and trial.get("submit_semantics_verified") is True
            and trial.get("result_semantics_verified") is True
            and trial.get("structured_value_field") == "task_result"
            and isinstance(trial.get("structured_value_sha256"), str)
            and len(trial["structured_value_sha256"]) == len("sha256:") + 64
            and trial["structured_value_sha256"].startswith("sha256:")
            and trial.get("extensions_disabled") is True
            and trial.get("profile_cleaned") is True
            and trial.get("browser_process_terminated") is True
            and isinstance(trial.get("browser_process_rss_bytes"), int)
            and trial["browser_process_rss_bytes"] > 0
            and isinstance(trial.get("chromium_process_count"), int)
            and 0 < trial["chromium_process_count"] <= MAX_BROWSER_PROCESS_TREE_SIZE
            and isinstance(trial.get("chromium_process_set_rss_bytes"), int)
            and trial["chromium_process_set_rss_bytes"] > 0
            and isinstance(trial.get("semantic_observation_bytes"), int)
            and 0
            < trial["semantic_observation_bytes"]
            <= MAX_AGENT_TASK_SEMANTIC_OBSERVATION_BYTES
            and isinstance(trial.get("action_latency_ms"), (int, float))
            and trial["action_latency_ms"] > 0
            and isinstance(trial.get("task_duration_ms"), (int, float))
            and trial["task_duration_ms"] >= trial["action_latency_ms"]
            for trial in agent_task_trials
            if trial.get("passed") is True
        )
        forced_close_successful_trials = sum(
            1 for trial in forced_close_trials if trial.get("passed") is True
        )
        forced_close_profiles_cleaned = all(
            trial.get("profile_cleaned") is True for trial in forced_close_trials
        )
        forced_close_surfaces_complete = all(
            trial.get("forced_close_detected") is True
            and trial.get("session_survived") is True
            and trial.get("profile_cleaned") is True
            for trial in forced_close_trials
            if trial.get("passed") is True
        )

        evidence = {
            "chrome_version": PINNED_CHROME_VERSION,
            "chrome_revision": PINNED_CHROME_REVISION,
            "repeatability_trials": REPEATABILITY_TRIALS,
            "successful_trials": successful_trials,
            "trial_pass_rate": trial_pass_rate,
            "profiles_cleaned": mv3_profiles_cleaned,
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
                "profiles_cleaned": agent_task_profiles_cleaned,
                "isolation_complete": agent_task_isolation_complete,
                "trial_results": agent_task_trials,
                "forced_close": {
                    "repeatability_trials": AGENT_TASK_REPEATABILITY_TRIALS,
                    "successful_trials": forced_close_successful_trials,
                    "profiles_cleaned": forced_close_profiles_cleaned,
                    "trial_results": forced_close_trials,
                },
            },
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
        print(json.dumps(evidence, sort_keys=True))
        if not mv3_profiles_cleaned:
            raise RuntimeError("Manifest V3 profile cleanup gate failed")
        if successful_trials != REPEATABILITY_TRIALS:
            raise RuntimeError(
                "Manifest V3 repeatability gate failed: "
                f"{successful_trials}/{REPEATABILITY_TRIALS} trials passed"
            )
        if not common_surfaces or not all(common_surfaces.values()):
            raise RuntimeError("Manifest V3 repeatability surfaces were incomplete")
        if not agent_task_profiles_cleaned:
            raise RuntimeError("Agent Task profile cleanup gate failed")
        if agent_task_successful_trials != AGENT_TASK_REPEATABILITY_TRIALS:
            raise RuntimeError(
                "Agent Task repeatability gate failed: "
                f"{agent_task_successful_trials}/{AGENT_TASK_REPEATABILITY_TRIALS} "
                "trials passed"
            )
        if not agent_task_isolation_complete:
            raise RuntimeError("Agent Task isolation gate failed")
        if not agent_task_surfaces_complete:
            raise RuntimeError("Agent Task repeatability surfaces were incomplete")
        if not forced_close_profiles_cleaned:
            raise RuntimeError("Agent Task forced-close profile cleanup gate failed")
        if (
            forced_close_successful_trials != AGENT_TASK_REPEATABILITY_TRIALS
            or not forced_close_surfaces_complete
        ):
            raise RuntimeError(
                "Agent Task forced-close recovery gate failed: "
                f"{forced_close_successful_trials}/{AGENT_TASK_REPEATABILITY_TRIALS} "
                "trials passed"
            )
        return 0
    finally:
        _stop_fixture_server(agent_task_server, agent_task_thread)
        _stop_fixture_server(fixture_server, fixture_thread)


if __name__ == "__main__":
    raise SystemExit(main())
