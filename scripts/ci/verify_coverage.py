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


def _deficient_region_filenames(summary: Mapping[str, Any]) -> set[str]:
    """Return filenames whose aggregate LLVM region summary is incomplete."""

    files = summary.get("files")
    if not isinstance(files, list):
        return set()
    deficits: set[str] = set()
    for file_entry in files:
        if not isinstance(file_entry, Mapping):
            continue
        filename = file_entry.get("filename")
        file_summary = file_entry.get("summary")
        if not isinstance(filename, str) or not isinstance(file_summary, Mapping):
            continue
        regions = file_summary.get("regions")
        if not isinstance(regions, Mapping):
            continue
        count = regions.get("count")
        covered = regions.get("covered")
        if (
            isinstance(count, int)
            and not isinstance(count, bool)
            and isinstance(covered, int)
            and not isinstance(covered, bool)
            and 0 <= covered <= count
            and covered != count
        ):
            deficits.add(filename)
    return deficits


def uncovered_file_region_summaries(payload: Mapping[str, Any]) -> list[str]:
    """Return source files whose aggregate LLVM region coverage is incomplete.

    This diagnostic uses the per-file aggregate summary rather than individual
    function instantiations. The latter can contain zero-count template or
    monomorphization regions even when the source file is fully covered after
    LLVM merges all instantiations. Malformed detail is ignored because the
    aggregate fail-closed coverage decision is enforced separately.
    """

    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    deficient = _deficient_region_filenames(summary)
    if not deficient:
        return []

    files = summary.get("files")
    if not isinstance(files, list):
        return []
    deficits: list[str] = []
    for file_entry in files:
        if not isinstance(file_entry, Mapping):
            continue
        filename = file_entry.get("filename")
        if not isinstance(filename, str) or filename not in deficient:
            continue
        file_summary = file_entry.get("summary")
        if not isinstance(file_summary, Mapping):
            continue
        regions = file_summary.get("regions")
        if not isinstance(regions, Mapping):
            continue
        count = regions.get("count")
        covered = regions.get("covered")
        if isinstance(count, int) and isinstance(covered, int):
            deficits.append(f"{filename}={covered}/{count}")
    return sorted(deficits)[:MAX_REGION_DIAGNOSTICS]


def _function_region_locations(
    summary: Mapping[str, Any],
    allowed_filenames: set[str] | None = None,
) -> set[str]:
    """Return uncovered source regions after merging LLVM function instantiations.

    LLVM can export the same source region once per monomorphization or other
    function instantiation. Aggregate file coverage treats that source region as
    covered when any equivalent instantiation executes, so diagnostics must sum
    execution counts by exact source coordinates before deciding it is missed.
    """

    functions = summary.get("functions")
    if not isinstance(functions, list):
        return set()
    region_counts: dict[tuple[str, int, int, int, int, int], int] = {}
    for function in functions:
        if not isinstance(function, Mapping):
            continue
        filenames = function.get("filenames")
        regions = function.get("regions")
        if not isinstance(filenames, list) or not isinstance(regions, list):
            continue
        for region in regions:
            if not isinstance(region, list) or len(region) < 8:
                continue
            line, column, end_line, end_column, count, file_id, _expanded_file_id, kind = region[:8]
            if (
                not isinstance(line, int)
                or isinstance(line, bool)
                or not isinstance(column, int)
                or isinstance(column, bool)
                or not isinstance(end_line, int)
                or isinstance(end_line, bool)
                or not isinstance(end_column, int)
                or isinstance(end_column, bool)
                or not isinstance(count, int)
                or isinstance(count, bool)
                or not isinstance(file_id, int)
                or isinstance(file_id, bool)
                or not isinstance(kind, int)
                or isinstance(kind, bool)
                or kind != 0
                or not 0 <= file_id < len(filenames)
                or not isinstance(filenames[file_id], str)
            ):
                continue
            filename = filenames[file_id]
            if allowed_filenames is not None and filename not in allowed_filenames:
                continue
            key = (filename, line, column, end_line, end_column, kind)
            region_counts[key] = region_counts.get(key, 0) + count
    return {
        f"{filename}:{line}:{column}"
        for (filename, line, column, _end_line, _end_column, _kind), count in region_counts.items()
        if count == 0
    }


def uncovered_region_locations(payload: Mapping[str, Any]) -> list[str]:
    """Return best-effort source coordinates for uncovered LLVM code regions.

    LLVM coverage JSON segments use ``line, column, count, has_count,
    is_region_entry, is_gap_region`` as their first six fields. Some valid LLVM
    exports can retain an uncovered code region only in the per-function
    ``regions`` array, so diagnostics fall back to that authoritative detail
    when segment entries do not identify the miss. When per-file aggregate
    summaries identify deficient files, the function fallback is scoped to
    those files so fully covered monomorphizations do not flood diagnostics.
    """

    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    files = summary.get("files")
    if not isinstance(files, list):
        return []

    deficient = _deficient_region_filenames(summary)
    locations: set[str] = set()
    for file_entry in files:
        if not isinstance(file_entry, Mapping):
            continue
        filename = file_entry.get("filename")
        segments = file_entry.get("segments")
        if not isinstance(filename, str) or not isinstance(segments, list):
            continue
        if deficient and filename not in deficient:
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
    if not locations:
        locations.update(_function_region_locations(summary, deficient or None))
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
        file_summaries = uncovered_file_region_summaries(payload)
        if file_summaries:
            details = f"{details}; uncovered files: {', '.join(file_summaries)}"
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
