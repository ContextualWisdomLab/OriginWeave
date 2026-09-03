"""Classify exact-head pull-request paths for lightweight versus Rust-heavy CI."""

from __future__ import annotations

import sys
from pathlib import PurePosixPath
from typing import Iterable


def parse_nul_paths(data: bytes) -> tuple[str, ...]:
    """Decode a NUL-delimited Git path stream and reject ambiguous path input."""

    if not data:
        return ()

    raw_paths = data.split(b"\0")
    if raw_paths[-1] == b"":
        raw_paths.pop()

    paths: list[str] = []
    for raw_path in raw_paths:
        try:
            path = raw_path.decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError("changed path is not valid UTF-8") from error

        if not path or path.startswith("/"):
            raise ValueError("changed path must be a non-empty repository-relative path")

        parts = PurePosixPath(path).parts
        if ".." in parts:
            raise ValueError("changed path must not contain parent traversal")
        paths.append(path)

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
    """Read NUL-delimited paths from stdin and emit fail-closed CI scope outputs."""

    try:
        paths = parse_nul_paths(sys.stdin.buffer.read())
    except ValueError as error:
        print(f"CI scope classification failed: {error}", file=sys.stderr)
        return 2

    documentation_only, rust_required = classify_paths(paths)
    sys.stdout.write(render_outputs(documentation_only, rust_required))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
