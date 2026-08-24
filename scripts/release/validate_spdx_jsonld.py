#!/usr/bin/env python3
"""Bounded, fail-closed SPDX 3.0.1 JSON-LD envelope verification.

This verifier deliberately checks only the serialization envelope needed before deeper
schema/ontology validation: the exact versioned global context identity required by the
official SPDX 3.0.1 JSON Schema, a bounded top-level JSON object, a bounded object-only
``@graph``, and exactly one ``SpdxDocument`` element. A release-facing helper additionally
binds candidate bytes to the canonical SHA-256 identity already declared by the release
manifest. It does not claim full SPDX structural or semantic conformance, artifact
authenticity, SBOM completeness, provenance, signing, publication, installation, update, or
rollback authority.
"""

from __future__ import annotations

import argparse
import errno
import hashlib
import hmac
import json
import math
import os
import pathlib
import stat
import sys
from collections.abc import Iterable
from typing import Any

SPDX_3_0_1_CONTEXT = "https://spdx.org/rdf/3.0.1/spdx-context.jsonld"
MAX_SPDX_JSONLD_BYTES = 16 * 1024 * 1024
MAX_SPDX_GRAPH_OBJECTS = 65_536


class SpdxJsonLdEnvelopeError(ValueError):
    """Typed, value-redacted failure for an SPDX JSON-LD envelope check."""

    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(f"SPDX JSON-LD envelope validation failed: {code}")


class _DuplicateJsonKey(ValueError):
    """Internal sentinel used to distinguish duplicate JSON object members."""


def _object_without_duplicates(pairs: Iterable[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise _DuplicateJsonKey
        result[key] = value
    return result


def _reject_nonfinite_constant(_value: str) -> None:
    raise ValueError("non-finite JSON number")


def _finite_json_float(value: str) -> float:
    """Parse one JSON float only when its binary representation remains finite."""

    parsed = float(value)
    if not math.isfinite(parsed):
        raise ValueError("non-finite JSON number")
    return parsed


def _has_required_spdx_context(context: Any) -> bool:
    """Require the exact SPDX 3.0.1 context identity used by the official JSON Schema.

    The SPDX 3.0.1 prose permits additional namespace mappings, but the same version's
    normative structural-validation resource constrains ``@context`` to this exact string.
    This preliminary gate therefore fails closed on context arrays rather than interpreting
    inline JSON-LD controls or term redefinitions before a reviewed schema-aware validator
    can establish their semantics.
    """

    return context == SPDX_3_0_1_CONTEXT


def validate_spdx_3_0_1_jsonld_bytes(payload: bytes) -> dict[str, int | str]:
    """Validate the bounded SPDX 3.0.1 JSON-LD serialization envelope.

    External document bytes are never included in raised errors. Successful validation
    proves only the narrow envelope contract documented by this module; callers must still
    perform the official SPDX JSON Schema and OWL/SHACL validation before claiming SPDX
    conformance. Excessive JSON nesting is treated as invalid external input instead of
    escaping the typed, value-redacted validation boundary.
    """

    if not isinstance(payload, bytes) or not payload or len(payload) > MAX_SPDX_JSONLD_BYTES:
        raise SpdxJsonLdEnvelopeError("invalid_size")

    try:
        text = payload.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise SpdxJsonLdEnvelopeError("invalid_utf8") from error

    try:
        decoded = json.loads(
            text,
            object_pairs_hook=_object_without_duplicates,
            parse_constant=_reject_nonfinite_constant,
            parse_float=_finite_json_float,
        )
    except _DuplicateJsonKey as error:
        raise SpdxJsonLdEnvelopeError("duplicate_key") from error
    except (json.JSONDecodeError, RecursionError, ValueError) as error:
        raise SpdxJsonLdEnvelopeError("invalid_json") from error

    if not isinstance(decoded, dict) or set(decoded) != {"@context", "@graph"}:
        raise SpdxJsonLdEnvelopeError("invalid_top_level")
    if not _has_required_spdx_context(decoded["@context"]):
        raise SpdxJsonLdEnvelopeError("invalid_context")

    graph = decoded["@graph"]
    if not isinstance(graph, list):
        raise SpdxJsonLdEnvelopeError("invalid_graph")
    if len(graph) > MAX_SPDX_GRAPH_OBJECTS:
        raise SpdxJsonLdEnvelopeError("too_many_graph_objects")

    document_count = 0
    for element in graph:
        if not isinstance(element, dict) or not isinstance(element.get("type"), str):
            raise SpdxJsonLdEnvelopeError("invalid_graph_object")
        if element["type"] == "SpdxDocument":
            document_count += 1

    if document_count != 1:
        raise SpdxJsonLdEnvelopeError("invalid_document_count")

    return {
        "context": SPDX_3_0_1_CONTEXT,
        "graph_object_count": len(graph),
        "spdx_document_count": document_count,
    }


def _valid_expected_sha256_digest(value: object) -> bool:
    """Return whether a release-artifact digest uses the canonical manifest spelling."""

    if not isinstance(value, str) or not value.startswith("sha256:") or len(value) != 71:
        return False
    return all(character in "0123456789abcdef" for character in value[7:])


def validate_release_spdx_3_0_1_jsonld_bytes(
    payload: bytes,
    expected_sha256_digest: str,
) -> dict[str, int | str]:
    """Validate exact release SBOM bytes against their manifest-backed digest and envelope.

    The expected digest is inert identity evidence supplied by the already-admitted release
    manifest. Candidate bytes must remain within the same bounded input contract, and a valid
    but substituted SPDX document fails before its contents can be promoted as release evidence.
    Diagnostics never reflect document-controlled values or the expected digest. Digest equality
    does not authenticate the manifest or grant signing, publication, installation, or update
    authority.
    """

    if not isinstance(payload, bytes) or not payload or len(payload) > MAX_SPDX_JSONLD_BYTES:
        raise SpdxJsonLdEnvelopeError("invalid_size")
    if not _valid_expected_sha256_digest(expected_sha256_digest):
        raise SpdxJsonLdEnvelopeError("invalid_expected_digest")

    actual_sha256_digest = "sha256:" + hashlib.sha256(payload).hexdigest()
    if not hmac.compare_digest(actual_sha256_digest, expected_sha256_digest):
        raise SpdxJsonLdEnvelopeError("digest_mismatch")

    summary = validate_spdx_3_0_1_jsonld_bytes(payload)
    summary["artifact_sha256"] = actual_sha256_digest
    return summary


def _nonblocking_read_opener(path: str, flags: int) -> int:
    """Open one candidate through a no-follow descriptor-relative component walk.

    Opening every ancestor relative to an already-open directory descriptor prevents a
    pathname swap from redirecting a later component through a transient symlink. Platforms
    that cannot provide both ``O_NOFOLLOW`` and descriptor-relative ``os.open`` fail closed
    instead of silently weakening this direct-path release boundary.
    """

    nofollow = getattr(os, "O_NOFOLLOW", 0)
    directory = getattr(os, "O_DIRECTORY", 0)
    cloexec = getattr(os, "O_CLOEXEC", 0)
    if not nofollow or not directory or os.open not in os.supports_dir_fd:
        raise OSError(errno.ENOTSUP, "secure descriptor-relative open is unavailable")

    candidate = pathlib.Path(path)
    components = list(candidate.parts)
    if candidate.is_absolute():
        anchor = candidate.anchor
        components = components[1:]
    else:
        anchor = "."
    if not components or any(component in {"", ".", ".."} for component in components):
        raise OSError(errno.ELOOP, "indirect path component is not permitted")

    directory_flags = os.O_RDONLY | directory | nofollow | cloexec
    parent_fd = os.open(anchor, directory_flags)
    try:
        for component in components[:-1]:
            next_fd = os.open(component, directory_flags, dir_fd=parent_fd)
            os.close(parent_fd)
            parent_fd = next_fd
        return os.open(
            components[-1],
            flags | getattr(os, "O_NONBLOCK", 0) | nofollow | cloexec,
            dir_fd=parent_fd,
        )
    finally:
        os.close(parent_fd)


def _require_direct_parent_chain(path: pathlib.Path) -> None:
    """Reject any existing ancestor that is not a direct directory entry."""

    parent = path.parent
    while parent != parent.parent:
        if not stat.S_ISDIR(parent.lstat().st_mode):
            raise SpdxJsonLdEnvelopeError("invalid_file_type")
        parent = parent.parent


def _read_bounded(path: pathlib.Path) -> bytes:
    """Read one bounded direct regular-file candidate without accepting indirect/streaming paths."""

    try:
        _require_direct_parent_chain(path)
        candidate_stat = path.lstat()
        if not stat.S_ISREG(candidate_stat.st_mode):
            raise SpdxJsonLdEnvelopeError("invalid_file_type")
        with open(path, "rb", opener=_nonblocking_read_opener) as source:
            opened_stat = os.fstat(source.fileno())
            if not stat.S_ISREG(opened_stat.st_mode):
                raise SpdxJsonLdEnvelopeError("invalid_file_type")
            if (candidate_stat.st_dev, candidate_stat.st_ino) != (
                opened_stat.st_dev,
                opened_stat.st_ino,
            ):
                raise SpdxJsonLdEnvelopeError("invalid_file_type")
            _require_direct_parent_chain(path)
            payload = source.read(MAX_SPDX_JSONLD_BYTES + 1)
    except OSError as error:
        if error.errno in {errno.ELOOP, errno.ENOTDIR}:
            raise SpdxJsonLdEnvelopeError("invalid_file_type") from error
        raise SpdxJsonLdEnvelopeError("read_failed") from error
    if not payload or len(payload) > MAX_SPDX_JSONLD_BYTES:
        raise SpdxJsonLdEnvelopeError("invalid_size")
    return payload


def main(argv: list[str] | None = None) -> int:
    """Validate one local SPDX JSON-LD envelope without echoing untrusted document bytes."""

    parser = argparse.ArgumentParser(
        description="Validate a bounded SPDX 3.0.1 JSON-LD serialization envelope."
    )
    parser.add_argument("document", type=pathlib.Path)
    arguments = parser.parse_args(argv)

    try:
        summary = validate_spdx_3_0_1_jsonld_bytes(_read_bounded(arguments.document))
    except SpdxJsonLdEnvelopeError as error:
        print(error, file=sys.stderr)
        return 2

    print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
