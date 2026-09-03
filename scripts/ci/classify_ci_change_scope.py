"""Classify exact-head pull-request changes for lightweight versus Rust-heavy CI."""

from __future__ import annotations

import sys
from pathlib import PurePosixPath
from typing import Iterable


def _decode_repository_path(raw_path: bytes) -> str:
    """Decode one Git pathname and enforce the repository-relative path boundary."""

    try:
        path = raw_path.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("changed path is not valid UTF-8") from error

    if not path or path.startswith("/"):
        raise ValueError("changed path must be a non-empty repository-relative path")

    parts = PurePosixPath(path).parts
    if ".." in parts:
        raise ValueError("changed path must not contain parent traversal")
    return path


def parse_nul_paths(data: bytes) -> tuple[str, ...]:
    """Decode a NUL-delimited Git path stream and reject ambiguous path input."""

    if not data:
        return ()

    raw_paths = data.split(b"\0")
    if raw_paths[-1] == b"":
        raw_paths.pop()

    return tuple(_decode_repository_path(raw_path) for raw_path in raw_paths)


def _similarity_status_is_valid(status: str, kind: str) -> bool:
    """Return whether a scored Git status has a canonical 0..100 similarity value."""

    if not status.startswith(kind) or len(status) == 1:
        return False
    score = status[1:]
    return score.isascii() and score.isdigit() and 0 <= int(score) <= 100


def parse_nul_name_status(data: bytes) -> tuple[str, ...]:
    """Decode ``git diff --name-status -z`` while preserving rename/copy preimages."""

    if not data:
        return ()

    fields = data.split(b"\0")
    if fields[-1] == b"":
        fields.pop()

    paths: list[str] = []
    index = 0
    while index < len(fields):
        raw_status = fields[index]
        index += 1
        try:
            status = raw_status.decode("ascii")
        except UnicodeDecodeError as error:
            raise ValueError("changed record has an invalid Git status") from error

        if status in {"A", "D", "M", "T", "U"} or _similarity_status_is_valid(
            status, "M"
        ):
            if index >= len(fields):
                raise ValueError("changed status record is missing its path")
            paths.append(_decode_repository_path(fields[index]))
            index += 1
            continue

        if _similarity_status_is_valid(status, "R") or _similarity_status_is_valid(
            status, "C"
        ):
            if index + 1 >= len(fields):
                raise ValueError("rename/copy status record must include both paths")
            paths.append(_decode_repository_path(fields[index]))
            paths.append(_decode_repository_path(fields[index + 1]))
            index += 2
            continue

        raise ValueError(f"changed record has unsupported Git status {status!r}")

    return tuple(paths)


def is_documentation_path(path: str) -> bool:
    """Return whether a repository path belongs to the prose-only contract surface."""

    return path.startswith("docs/") or ("/" not in path and path.endswith(".md"))


def classify_paths(paths: Iterable[str]) -> tuple[bool, bool]:
    """Return ``(documentation_only, rust_required)`` for exact changed paths."""

    materialized = tuple(paths)
    if not materialized:
        return False, True

    documentation_only = all(is_documentation_path(path) for path in materialized)
    return documentation_only, not documentation_only


def render_outputs(documentation_only: bool, rust_required: bool) -> str:
    """Render deterministic GitHub Actions outputs without accepting extra state."""

    return (
        f"documentation_only={'true' if documentation_only else 'false'}\n"
        f"rust_required={'true' if rust_required else 'false'}\n"
    )


def main() -> int:
    """Read status-aware NUL-delimited changes and emit fail-closed CI scope outputs."""

    try:
        paths = parse_nul_name_status(sys.stdin.buffer.read())
    except ValueError as error:
        print(f"CI scope classification failed: {error}", file=sys.stderr)
        return 2

    documentation_only, rust_required = classify_paths(paths)
    sys.stdout.write(render_outputs(documentation_only, rust_required))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
