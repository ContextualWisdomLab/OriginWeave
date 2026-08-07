#!/usr/bin/env python3
"""Fail unless an LLVM coverage summary reports exact production coverage."""

from __future__ import annotations

import json
import pathlib
import sys
from collections.abc import Mapping
from typing import Any

REQUIRED_METRICS = ("functions", "lines", "regions", "branches")
MAX_REGION_DIAGNOSTICS = 100


def _single_data_summary(payload: Mapping[str, Any]) -> Mapping[str, Any]:
    """Return the single LLVM data summary or reject an ambiguous payload."""

    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1 or not isinstance(data[0], Mapping):
        raise ValueError("coverage JSON must contain exactly one data summary")
    return data[0]


def uncovered_metrics(payload: Mapping[str, Any]) -> dict[str, tuple[int, int]]:
    """Return metrics whose covered count differs from their total count."""

    totals = _single_data_summary(payload).get("totals")
    if not isinstance(totals, Mapping):
        raise ValueError("coverage JSON is missing totals")

    uncovered: dict[str, tuple[int, int]] = {}
    for metric_name in REQUIRED_METRICS:
        metric = totals.get(metric_name)
        if not isinstance(metric, Mapping):
            raise ValueError(f"coverage JSON is missing {metric_name}")
        count = metric.get("count")
        covered = metric.get("covered")
        if not isinstance(count, int) or isinstance(count, bool):
            raise ValueError(f"{metric_name}.count must be an integer")
        if not isinstance(covered, int) or isinstance(covered, bool):
            raise ValueError(f"{metric_name}.covered must be an integer")
        if count < 0 or covered < 0 or covered > count:
            raise ValueError(f"{metric_name} contains impossible counts")
        if covered != count:
            uncovered[metric_name] = (covered, count)
    return uncovered


def uncovered_region_locations(payload: Mapping[str, Any]) -> list[str]:
    """Return best-effort source coordinates for uncovered LLVM region entries.

    LLVM coverage JSON segments use ``line, column, count, has_count,
    is_region_entry, is_gap_region`` as their first six fields. Diagnostics are
    intentionally best-effort: malformed or summary-only file detail never
    replaces the aggregate fail-closed coverage decision.
    """

    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    files = summary.get("files")
    if not isinstance(files, list):
        return []

    locations: set[str] = set()
    for file_entry in files:
        if not isinstance(file_entry, Mapping):
            continue
        filename = file_entry.get("filename")
        segments = file_entry.get("segments")
        if not isinstance(filename, str) or not isinstance(segments, list):
            continue
        for segment in segments:
            if not isinstance(segment, list) or len(segment) < 6:
                continue
            line, column, count, has_count, is_region_entry, is_gap_region = segment[:6]
            if (
                isinstance(line, int)
                and not isinstance(line, bool)
                and isinstance(column, int)
                and not isinstance(column, bool)
                and isinstance(count, int)
                and not isinstance(count, bool)
                and has_count is True
                and is_region_entry is True
                and is_gap_region is False
                and count == 0
            ):
                locations.add(f"{filename}:{line}:{column}")
    return sorted(locations)[:MAX_REGION_DIAGNOSTICS]


def verify_file(path: pathlib.Path) -> None:
    """Load one coverage JSON artifact and enforce exact coverage."""

    with path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    if not isinstance(payload, Mapping):
        raise ValueError("coverage JSON root must be an object")
    uncovered = uncovered_metrics(payload)
    if uncovered:
        details = ", ".join(
            f"{metric}={covered}/{count}"
            for metric, (covered, count) in sorted(uncovered.items())
        )
        region_locations = uncovered_region_locations(payload)
        if region_locations:
            details = f"{details}; uncovered regions: {', '.join(region_locations)}"
        raise RuntimeError(f"production coverage is below 100%: {details}")


def main(arguments: list[str]) -> int:
    """Run the command-line coverage verifier."""

    if len(arguments) != 1:
        print("usage: verify_coverage.py <coverage.json>", file=sys.stderr)
        return 2
    try:
        verify_file(pathlib.Path(arguments[0]))
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print("production functions, lines, regions, and branches are 100% covered")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
