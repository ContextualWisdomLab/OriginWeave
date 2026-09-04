"""Classify exact-head pull-request changes for lightweight versus Rust-heavy CI."""

from __future__ import annotations

import sys
from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import Iterable

_ABSENT_MODE = "000000"
_PLAIN_DOCUMENT_MODE = "100644"


@dataclass(frozen=True, slots=True)
class RawChange:
    """One non-combined Git raw-diff record with mode and path authority intact."""

    source_mode: str
    destination_mode: str
    status: str
    paths: tuple[str, ...]


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
    """Decode a NUL-delimited Git path stream without granting scope authority."""

    if not data:
        return ()
    if not data.endswith(b"\0"):
        raise ValueError("changed path stream must be NUL-terminated")

    raw_paths = data.split(b"\0")
    raw_paths.pop()
    return tuple(_decode_repository_path(raw_path) for raw_path in raw_paths)


def _similarity_status_is_valid(status: str, kind: str) -> bool:
    """Return whether a scored Git status has a canonical 0..100 similarity value."""

    if not status.startswith(kind) or len(status) == 1:
        return False
    score = status[1:]
    return score.isascii() and score.isdigit() and 0 <= int(score) <= 100


def _status_path_count(status: str) -> int:
    """Return the raw-diff pathname cardinality for a supported Git status."""

    if status in {"A", "D", "M", "T", "U"} or _similarity_status_is_valid(
        status, "M"
    ):
        return 1
    if _similarity_status_is_valid(status, "R") or _similarity_status_is_valid(
        status, "C"
    ):
        return 2
    raise ValueError(f"changed record has unsupported Git status {status!r}")


def parse_nul_name_status(data: bytes) -> tuple[str, ...]:
    """Decode legacy name/status evidence while preserving rename/copy preimages.

    This representation intentionally carries no lightweight-classification authority
    because Git file modes are absent. It remains useful for diagnostics and for
    failing closed when a producer has not yet migrated to raw-diff evidence.
    """

    if not data:
        return ()
    if not data.endswith(b"\0"):
        raise ValueError("changed status stream must be NUL-terminated")

    fields = data.split(b"\0")
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

        path_count = _status_path_count(status)
        if index + path_count > len(fields):
            if path_count == 2:
                raise ValueError("rename/copy status record must include both paths")
            raise ValueError("changed status record is missing its path")

        paths.extend(
            _decode_repository_path(raw_path)
            for raw_path in fields[index : index + path_count]
        )
        index += path_count

    return tuple(paths)


def _validate_raw_mode(mode: str) -> None:
    """Reject malformed Git modes before they can influence scope classification."""

    if len(mode) != 6 or any(character not in "01234567" for character in mode):
        raise ValueError(f"changed raw record has invalid mode {mode!r}")


def _validate_raw_object_id(object_id: str) -> None:
    """Reject malformed abbreviated object identifiers in raw-diff metadata."""

    if not 4 <= len(object_id) <= 64 or any(
        character not in "0123456789abcdef" for character in object_id
    ):
        raise ValueError("changed raw record has an invalid object id")


def _validate_raw_object_identity_semantics(
    source_mode: str,
    destination_mode: str,
    source_oid: str,
    destination_oid: str,
) -> None:
    """Require raw object identities to agree with two-tree presence metadata."""

    if len(source_oid) != len(destination_oid):
        raise ValueError("changed raw record object ids have inconsistent widths")

    for mode, object_id in (
        (source_mode, source_oid),
        (destination_mode, destination_oid),
    ):
        side_absent = mode == _ABSENT_MODE
        identity_absent = not object_id.strip("0")
        if side_absent != identity_absent:
            raise ValueError("changed raw record has inconsistent mode/object id metadata")


def _validate_raw_mode_semantics(source_mode: str, destination_mode: str, status: str) -> None:
    """Reject status/mode combinations that cannot describe a normal two-tree diff."""

    kind = status[0]
    source_exists = source_mode != _ABSENT_MODE
    destination_exists = destination_mode != _ABSENT_MODE

    valid = False
    if kind == "A":
        valid = not source_exists and destination_exists
    elif kind == "D":
        valid = source_exists and not destination_exists
    elif kind == "U":
        valid = not source_exists and not destination_exists
    elif kind == "T":
        valid = source_exists and destination_exists and source_mode != destination_mode
    elif kind in {"M", "R", "C"}:
        valid = source_exists and destination_exists

    if not valid:
        raise ValueError("changed raw record has inconsistent status/mode metadata")


def _parse_raw_metadata(raw_metadata: bytes) -> tuple[str, str, str]:
    """Decode one canonical non-combined ``git diff --raw -z`` metadata field."""

    try:
        metadata = raw_metadata.decode("ascii")
    except UnicodeDecodeError as error:
        raise ValueError("changed raw record metadata is not ASCII") from error

    fields = metadata.split(" ")
    if len(fields) != 5 or any(not field for field in fields):
        raise ValueError("changed raw record metadata has an invalid field layout")

    source_mode_field, destination_mode, source_oid, destination_oid, status = fields
    if not source_mode_field.startswith(":") or source_mode_field.startswith("::"):
        raise ValueError("combined or malformed raw diff metadata is unsupported")

    source_mode = source_mode_field[1:]
    _validate_raw_mode(source_mode)
    _validate_raw_mode(destination_mode)
    _validate_raw_object_id(source_oid)
    _validate_raw_object_id(destination_oid)
    _status_path_count(status)
    _validate_raw_mode_semantics(source_mode, destination_mode, status)
    _validate_raw_object_identity_semantics(
        source_mode,
        destination_mode,
        source_oid,
        destination_oid,
    )
    return source_mode, destination_mode, status


def parse_nul_raw_changes(data: bytes) -> tuple[RawChange, ...]:
    """Decode ``git diff --raw -z`` without discarding modes or rename preimages."""

    if not data:
        return ()
    if not data.endswith(b"\0"):
        raise ValueError("changed raw stream must be NUL-terminated")

    fields = data.split(b"\0")
    fields.pop()

    changes: list[RawChange] = []
    index = 0
    while index < len(fields):
        source_mode, destination_mode, status = _parse_raw_metadata(fields[index])
        index += 1
        path_count = _status_path_count(status)
        if index + path_count > len(fields):
            if path_count == 2:
                raise ValueError("raw rename/copy record must include both paths")
            raise ValueError("changed raw record is missing its path")

        paths = tuple(
            _decode_repository_path(raw_path)
            for raw_path in fields[index : index + path_count]
        )
        index += path_count
        changes.append(
            RawChange(
                source_mode=source_mode,
                destination_mode=destination_mode,
                status=status,
                paths=paths,
            )
        )

    return tuple(changes)


def is_documentation_path(path: str) -> bool:
    """Return whether a repository path belongs to the reviewed Markdown prose surface."""

    return path.endswith(".md") and (path.startswith("docs/") or "/" not in path)


def classify_paths(paths: Iterable[str]) -> tuple[bool, bool]:
    """Fail closed for path-only evidence because file modes are not represented."""

    tuple(paths)
    return False, True


def _change_is_plain_documentation(change: RawChange) -> bool:
    """Return whether one raw change is proven to affect ordinary documentation blobs."""

    if change.status.startswith(("T", "U")):
        return False
    if any(not is_documentation_path(path) for path in change.paths):
        return False

    return all(
        mode in {_ABSENT_MODE, _PLAIN_DOCUMENT_MODE}
        for mode in (change.source_mode, change.destination_mode)
    )


def classify_changes(changes: Iterable[RawChange]) -> tuple[bool, bool]:
    """Return ``(documentation_only, rust_required)`` from mode-aware raw evidence."""

    materialized = tuple(changes)
    if not materialized:
        return False, True

    documentation_only = all(
        _change_is_plain_documentation(change) for change in materialized
    )
    return documentation_only, not documentation_only


def render_outputs(documentation_only: bool, rust_required: bool) -> str:
    """Render deterministic GitHub Actions outputs without accepting extra state."""

    return (
        f"documentation_only={'true' if documentation_only else 'false'}\n"
        f"rust_required={'true' if rust_required else 'false'}\n"
    )


def main() -> int:
    """Read exact Git diff evidence and emit fail-closed CI scope outputs."""

    data = sys.stdin.buffer.read()
    try:
        if data.startswith(b":"):
            documentation_only, rust_required = classify_changes(
                parse_nul_raw_changes(data)
            )
        else:
            documentation_only, rust_required = classify_paths(
                parse_nul_name_status(data)
            )
    except ValueError as error:
        print(f"CI scope classification failed: {error}", file=sys.stderr)
        return 2

    sys.stdout.write(render_outputs(documentation_only, rust_required))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
