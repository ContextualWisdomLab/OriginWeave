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
import contextvars
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
_EXPECTED_PARENT_IDENTITIES: contextvars.ContextVar[tuple[tuple[int, int], ...]] = (
    contextvars.ContextVar("spdx_expected_parent_identities", default=())
)


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


def _count_spdx_documents(value: Any) -> int:
    """Count SPDX document elements at any depth without recursive interpreter use.

    SPDX element collections may inline other elements below the top-level graph. The
    serialization contract admits at most one ``SpdxDocument`` element, so a nested second
    document must not evade the preliminary envelope gate merely because it is not a direct
    ``@graph`` member. The walk is iterative and remains bounded by the already size-bounded
    parsed input.
    """

    count = 0
    pending = [value]
    while pending:
        current = pending.pop()
        if isinstance(current, dict):
            if current.get("type") == "SpdxDocument":
                count += 1
                if count > 1:
                    return count
            pending.extend(current.values())
        elif isinstance(current, list):
            pending.extend(current)
    return count


def validate_spdx_3_0_1_jsonld_bytes(payload: bytes) -> dict[str, int | str]:
    """Validate the bounded SPDX 3.0.1 JSON-LD serialization envelope.

    External document bytes are never included in raised errors or retained by their
    exception chain. Successful validation proves only the narrow envelope contract
    documented by this module; callers must still perform the official SPDX JSON Schema and
    OWL/SHACL validation before claiming SPDX conformance. Excessive JSON nesting is treated
    as invalid external input instead of escaping the typed, value-redacted validation
    boundary.
    """

    if not isinstance(payload, bytes) or not payload or len(payload) > MAX_SPDX_JSONLD_BYTES:
        raise SpdxJsonLdEnvelopeError("invalid_size")

    invalid_utf8 = False
    try:
        text = payload.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        invalid_utf8 = True
        text = ""
    if invalid_utf8:
        # UnicodeDecodeError retains the complete input bytes in its ``object`` attribute.
        # Raise only after leaving the handler so the public error keeps no payload-bearing
        # ``__cause__`` or ``__context__`` reference.
        raise SpdxJsonLdEnvelopeError("invalid_utf8")

    parse_error_code: str | None = None
    try:
        decoded = json.loads(
            text,
            object_pairs_hook=_object_without_duplicates,
            parse_constant=_reject_nonfinite_constant,
            parse_float=_finite_json_float,
        )
    except _DuplicateJsonKey:
        parse_error_code = "duplicate_key"
        decoded = None
    except (json.JSONDecodeError, RecursionError, ValueError):
        parse_error_code = "invalid_json"
        decoded = None
    if parse_error_code is not None:
        # JSONDecodeError retains the complete decoded document in its ``doc`` attribute.
        # Do not chain parser exceptions that could therefore turn private SBOM bytes into
        # logging, tracing, or support-bundle data downstream.
        raise SpdxJsonLdEnvelopeError(parse_error_code)

    if not isinstance(decoded, dict) or set(decoded) != {"@context", "@graph"}:
        raise SpdxJsonLdEnvelopeError("invalid_top_level")
    if not _has_required_spdx_context(decoded["@context"]):
        raise SpdxJsonLdEnvelopeError("invalid_context")

    graph = decoded["@graph"]
    if not isinstance(graph, list):
        raise SpdxJsonLdEnvelopeError("invalid_graph")
    if len(graph) > MAX_SPDX_GRAPH_OBJECTS:
        raise SpdxJsonLdEnvelopeError("too_many_graph_objects")

    for element in graph:
        if not isinstance(element, dict) or not isinstance(element.get("type"), str):
            raise SpdxJsonLdEnvelopeError("invalid_graph_object")

    document_count = _count_spdx_documents(graph)
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


def _path_components(path: pathlib.Path) -> tuple[str, list[str]]:
    """Return the direct lexical anchor and components used by the secure open walk."""

    components = list(path.parts)
    if path.is_absolute():
        anchor = path.anchor
        components = components[1:]
    else:
        anchor = "."
    if not components or any(component in {"", ".", ".."} for component in components):
        raise OSError(errno.ELOOP, "indirect path component is not permitted")
    return anchor, components


def _direct_parent_identities(path: pathlib.Path) -> tuple[tuple[int, int], ...]:
    """Capture the exact directory identities that own one direct candidate path."""

    anchor, components = _path_components(path)
    current = pathlib.Path(anchor)
    identities: list[tuple[int, int]] = []
    for component in [None, *components[:-1]]:
        if component is not None:
            current = current / component
        parent_stat = current.lstat()
        if not stat.S_ISDIR(parent_stat.st_mode):
            raise SpdxJsonLdEnvelopeError("invalid_file_type")
        identities.append((parent_stat.st_dev, parent_stat.st_ino))
    return tuple(identities)


def _require_expected_directory_identity(
    descriptor: int,
    expected_identity: tuple[int, int],
) -> None:
    """Fail the descriptor walk when an admitted directory identity has changed."""

    opened_stat = os.fstat(descriptor)
    if not stat.S_ISDIR(opened_stat.st_mode) or (
        opened_stat.st_dev,
        opened_stat.st_ino,
    ) != expected_identity:
        raise OSError(errno.ELOOP, "direct path parent identity changed")


def _nonblocking_read_opener(path: str, flags: int) -> int:
    """Open one candidate through an identity-bound descriptor-relative component walk.

    Opening every ancestor relative to an already-open directory descriptor prevents a
    pathname swap from redirecting a later component through a transient symlink. Every
    opened directory must also retain the identity captured by the caller before leaf
    admission, so replacing a parent with a different real directory cannot preserve trust
    merely by hard-linking the same leaf inode into the replacement. Platforms that cannot
    provide both ``O_NOFOLLOW`` and descriptor-relative ``os.open`` fail closed instead of
    silently weakening this direct-path release boundary.
    """

    nofollow = getattr(os, "O_NOFOLLOW", 0)
    directory = getattr(os, "O_DIRECTORY", 0)
    cloexec = getattr(os, "O_CLOEXEC", 0)
    if not nofollow or not directory or os.open not in os.supports_dir_fd:
        raise OSError(errno.ENOTSUP, "secure descriptor-relative open is unavailable")

    candidate = pathlib.Path(path)
    anchor, components = _path_components(candidate)
    expected_parent_identities = _EXPECTED_PARENT_IDENTITIES.get()
    if len(expected_parent_identities) != len(components):
        raise OSError(errno.ELOOP, "direct path parent identity is unavailable")

    directory_flags = os.O_RDONLY | directory | nofollow | cloexec
    parent_fd = os.open(anchor, directory_flags)
    try:
        _require_expected_directory_identity(parent_fd, expected_parent_identities[0])
        for index, component in enumerate(components[:-1], start=1):
            next_fd = os.open(component, directory_flags, dir_fd=parent_fd)
            os.close(parent_fd)
            parent_fd = next_fd
            _require_expected_directory_identity(parent_fd, expected_parent_identities[index])
        return os.open(
            components[-1],
            flags | getattr(os, "O_NONBLOCK", 0) | nofollow | cloexec,
            dir_fd=parent_fd,
        )
    finally:
        os.close(parent_fd)


def _read_bounded(path: pathlib.Path) -> bytes:
    """Read one bounded direct regular-file candidate with stable path-owner identities."""

    try:
        expected_parent_identities = _direct_parent_identities(path)
        candidate_stat = path.lstat()
        if not stat.S_ISREG(candidate_stat.st_mode):
            raise SpdxJsonLdEnvelopeError("invalid_file_type")
        identity_token = _EXPECTED_PARENT_IDENTITIES.set(expected_parent_identities)
        try:
            with open(path, "rb", opener=_nonblocking_read_opener) as source:
                opened_stat = os.fstat(source.fileno())
                if not stat.S_ISREG(opened_stat.st_mode):
                    raise SpdxJsonLdEnvelopeError("invalid_file_type")
                if (candidate_stat.st_dev, candidate_stat.st_ino) != (
                    opened_stat.st_dev,
                    opened_stat.st_ino,
                ):
                    raise SpdxJsonLdEnvelopeError("invalid_file_type")
                if _direct_parent_identities(path) != expected_parent_identities:
                    raise SpdxJsonLdEnvelopeError("invalid_file_type")
                payload = source.read(MAX_SPDX_JSONLD_BYTES + 1)
                if _direct_parent_identities(path) != expected_parent_identities:
                    raise SpdxJsonLdEnvelopeError("invalid_file_type")
                final_stat = path.lstat()
                if not stat.S_ISREG(final_stat.st_mode) or (
                    final_stat.st_dev,
                    final_stat.st_ino,
                ) != (opened_stat.st_dev, opened_stat.st_ino):
                    raise SpdxJsonLdEnvelopeError("invalid_file_type")
        finally:
            _EXPECTED_PARENT_IDENTITIES.reset(identity_token)
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
