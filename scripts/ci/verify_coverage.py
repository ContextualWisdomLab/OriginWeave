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
    """Return the one LLVM data summary or reject an ambiguous payload."""
    data = payload.get("data")
    if not isinstance(data, list) or len(data) != 1 or not isinstance(data[0], Mapping):
        raise ValueError("coverage JSON must contain exactly one data summary")
    return data[0]


def uncovered_metrics(payload: Mapping[str, Any]) -> dict[str, tuple[int, int]]:
    """Return exact metrics whose covered count differs from their total count."""
    totals = _single_data_summary(payload).get("totals")
    if not isinstance(totals, Mapping):
        raise ValueError("coverage JSON is missing totals")
    uncovered: dict[str, tuple[int, int]] = {}
    for name in REQUIRED_METRICS:
        metric = totals.get(name)
        if not isinstance(metric, Mapping):
            raise ValueError(f"coverage JSON is missing {name}")
        count = metric.get("count")
        covered = metric.get("covered")
        if not isinstance(count, int) or isinstance(count, bool):
            raise ValueError(f"{name}.count must be an integer")
        if not isinstance(covered, int) or isinstance(covered, bool):
            raise ValueError(f"{name}.covered must be an integer")
        if count < 0 or covered < 0 or covered > count:
            raise ValueError(f"{name} contains impossible counts")
        if covered != count:
            uncovered[name] = (covered, count)
    return uncovered


def _deficient_region_filenames(summary: Mapping[str, Any]) -> set[str]:
    """Return files whose aggregate LLVM region summary is incomplete."""
    files = summary.get("files")
    if not isinstance(files, list):
        return set()
    deficits: set[str] = set()
    for entry in files:
        if not isinstance(entry, Mapping):
            continue
        filename = entry.get("filename")
        file_summary = entry.get("summary")
        if not isinstance(filename, str) or not isinstance(file_summary, Mapping):
            continue
        regions = file_summary.get("regions")
        if not isinstance(regions, Mapping):
            continue
        count, covered = regions.get("count"), regions.get("covered")
        if (
            isinstance(count, int)
            and not isinstance(count, bool)
            and isinstance(covered, int)
            and not isinstance(covered, bool)
            and 0 <= covered < count
        ):
            deficits.add(filename)
    return deficits


def uncovered_file_region_summaries(payload: Mapping[str, Any]) -> list[str]:
    """Return deficient per-file aggregate region summaries."""
    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    deficient = _deficient_region_filenames(summary)
    files = summary.get("files")
    if not deficient or not isinstance(files, list):
        return []
    output: list[str] = []
    for entry in files:
        if not isinstance(entry, Mapping) or entry.get("filename") not in deficient:
            continue
        file_summary = entry.get("summary")
        if not isinstance(file_summary, Mapping):
            continue
        regions = file_summary.get("regions")
        if not isinstance(regions, Mapping):
            continue
        filename, count, covered = entry.get("filename"), regions.get("count"), regions.get("covered")
        if isinstance(filename, str) and isinstance(count, int) and isinstance(covered, int):
            output.append(f"{filename}={covered}/{count}")
    return sorted(output)[:MAX_REGION_DIAGNOSTICS]


def _function_region_locations(
    summary: Mapping[str, Any], allowed_filenames: set[str] | None = None
) -> set[str]:
    """Merge equivalent LLVM function regions and return genuinely uncovered code regions."""
    functions = summary.get("functions")
    if not isinstance(functions, list):
        return set()
    counts: dict[tuple[str, int, int, int, int, int], int] = {}
    for function in functions:
        if not isinstance(function, Mapping):
            continue
        filenames, regions = function.get("filenames"), function.get("regions")
        if not isinstance(filenames, list) or not isinstance(regions, list):
            continue
        for region in regions:
            if not isinstance(region, list) or len(region) < 8:
                continue
            line, column, end_line, end_column, count, file_id, _expanded_id, kind = region[:8]
            numbers = (line, column, end_line, end_column, count, file_id, kind)
            if any(not isinstance(value, int) or isinstance(value, bool) for value in numbers):
                continue
            if kind != 0 or not 0 <= file_id < len(filenames):
                continue
            filename = filenames[file_id]
            if not isinstance(filename, str):
                continue
            if allowed_filenames is not None and filename not in allowed_filenames:
                continue
            key = (filename, line, column, end_line, end_column, kind)
            counts[key] = counts.get(key, 0) + count
    return {
        f"{filename}:{line}:{column}"
        for (filename, line, column, _end_line, _end_column, _kind), count in counts.items()
        if count == 0
    }


def uncovered_raw_region_diagnostics(payload: Mapping[str, Any]) -> list[str]:
    """Expose raw zero-count function-region metadata only for deficient files."""
    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    deficient = _deficient_region_filenames(summary)
    functions = summary.get("functions")
    if not deficient or not isinstance(functions, list):
        return []
    output: set[str] = set()
    for function in functions:
        if not isinstance(function, Mapping):
            continue
        name, filenames, regions = function.get("name"), function.get("filenames"), function.get("regions")
        if not isinstance(name, str) or not isinstance(filenames, list) or not isinstance(regions, list):
            continue
        for region in regions:
            if not isinstance(region, list) or len(region) < 8:
                continue
            line, column, end_line, end_column, count, file_id, expanded_id, kind = region[:8]
            numbers = (line, column, end_line, end_column, count, file_id, expanded_id, kind)
            if any(not isinstance(value, int) or isinstance(value, bool) for value in numbers):
                continue
            if count != 0 or not 0 <= file_id < len(filenames):
                continue
            filename = filenames[file_id]
            if not isinstance(filename, str) or filename not in deficient:
                continue
            output.add(
                f"{filename}:{line}:{column}-{end_line}:{end_column} count=0 "
                f"kind={kind} expanded_file_id={expanded_id} function={name}"
            )
    return sorted(output)[:MAX_REGION_DIAGNOSTICS]


def uncovered_raw_segment_diagnostics(payload: Mapping[str, Any]) -> list[str]:
    """Expose zero-count LLVM file-segment flags only for deficient files."""
    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    deficient = _deficient_region_filenames(summary)
    files = summary.get("files")
    if not deficient or not isinstance(files, list):
        return []
    output: set[str] = set()
    for entry in files:
        if not isinstance(entry, Mapping):
            continue
        filename, segments = entry.get("filename"), entry.get("segments")
        if not isinstance(filename, str) or filename not in deficient or not isinstance(segments, list):
            continue
        for segment in segments:
            if not isinstance(segment, list) or len(segment) < 6:
                continue
            line, column, count, has_count, region_entry, gap = segment[:6]
            if (
                isinstance(line, int)
                and not isinstance(line, bool)
                and isinstance(column, int)
                and not isinstance(column, bool)
                and isinstance(count, int)
                and not isinstance(count, bool)
                and isinstance(has_count, bool)
                and isinstance(region_entry, bool)
                and isinstance(gap, bool)
                and count == 0
                and has_count
            ):
                output.add(
                    f"{filename}:{line}:{column} count=0 has_count=True "
                    f"region_entry={region_entry} gap={gap}"
                )
    return sorted(output)[:MAX_REGION_DIAGNOSTICS]


def uncovered_expansion_locations(payload: Mapping[str, Any]) -> list[str]:
    """Return zero-count macro-expansion source coordinates in deficient files."""
    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    deficient = _deficient_region_filenames(summary)
    files = summary.get("files")
    if not isinstance(files, list):
        return []
    output: set[str] = set()
    for entry in files:
        if not isinstance(entry, Mapping):
            continue
        filename, expansions = entry.get("filename"), entry.get("expansions")
        if not isinstance(filename, str) or not isinstance(expansions, list):
            continue
        if deficient and filename not in deficient:
            continue
        for expansion in expansions:
            if not isinstance(expansion, Mapping):
                continue
            region = expansion.get("source_region")
            if not isinstance(region, list) or len(region) < 8:
                continue
            line, column, _end_line, _end_column, count = region[:5]
            if (
                isinstance(line, int)
                and not isinstance(line, bool)
                and isinstance(column, int)
                and not isinstance(column, bool)
                and isinstance(count, int)
                and not isinstance(count, bool)
                and count == 0
            ):
                output.add(f"{filename}:{line}:{column}")
    return sorted(output)[:MAX_REGION_DIAGNOSTICS]


def uncovered_region_locations(payload: Mapping[str, Any]) -> list[str]:
    """Return best-effort zero-count code-region coordinates."""
    try:
        summary = _single_data_summary(payload)
    except ValueError:
        return []
    files = summary.get("files")
    if not isinstance(files, list):
        return []
    deficient = _deficient_region_filenames(summary)
    output: set[str] = set()
    for entry in files:
        if not isinstance(entry, Mapping):
            continue
        filename, segments = entry.get("filename"), entry.get("segments")
        if not isinstance(filename, str) or not isinstance(segments, list):
            continue
        if deficient and filename not in deficient:
            continue
        for segment in segments:
            if not isinstance(segment, list) or len(segment) < 6:
                continue
            line, column, count, has_count, region_entry, gap = segment[:6]
            if (
                isinstance(line, int)
                and not isinstance(line, bool)
                and isinstance(column, int)
                and not isinstance(column, bool)
                and isinstance(count, int)
                and not isinstance(count, bool)
                and has_count is True
                and region_entry is True
                and gap is False
                and count == 0
            ):
                output.add(f"{filename}:{line}:{column}")
    if not output:
        output.update(_function_region_locations(summary, deficient or None))
    return sorted(output)[:MAX_REGION_DIAGNOSTICS]


def verify_file(path: pathlib.Path) -> None:
    """Load one coverage report and enforce all exact production metrics."""
    with path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    if not isinstance(payload, Mapping):
        raise ValueError("coverage JSON root must be an object")
    uncovered = uncovered_metrics(payload)
    if not uncovered:
        return
    details = ", ".join(
        f"{metric}={covered}/{count}" for metric, (covered, count) in sorted(uncovered.items())
    )
    file_summaries = uncovered_file_region_summaries(payload)
    if file_summaries:
        details += f"; uncovered files: {', '.join(file_summaries)}"
    regions = uncovered_region_locations(payload)
    if regions:
        details += f"; uncovered regions: {', '.join(regions)}"
    expansions = uncovered_expansion_locations(payload)
    if expansions:
        details += f"; uncovered expansions: {', '.join(expansions)}"
    if file_summaries and not regions and not expansions:
        segments = uncovered_raw_segment_diagnostics(payload)
        if segments:
            details += f"; raw zero-count segments: {', '.join(segments)}"
        raw_regions = uncovered_raw_region_diagnostics(payload)
        if raw_regions:
            details += f"; raw zero-count regions: {', '.join(raw_regions)}"
    raise RuntimeError(f"production coverage is below 100%: {details}")


def main(arguments: list[str]) -> int:
    """Run the exact coverage gate as a command-line program."""
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
