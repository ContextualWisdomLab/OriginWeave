#!/usr/bin/env python3
"""Bounded, fail-closed SPDX 3.0.1 JSON-LD envelope verification.

This verifier deliberately checks only the serialization envelope needed before deeper
schema/ontology validation: the exact versioned global context identity required by the
official SPDX 3.0.1 JSON Schema, a bounded top-level JSON object, a bounded object-only
``@graph``, and exactly one ``SpdxDocument`` element. It does not claim full SPDX structural
or semantic conformance, artifact authenticity, SBOM completeness, provenance, signing,
publication, installation, update, or rollback authority.
"""

from __future__ import annotations

import argparse
import json
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


def _nonblocking_read_opener(path: str, flags: int) -> int:
    """Open one local candidate without allowing a FIFO/device open to wait indefinitely."""

    return os.open(path, flags | getattr(os, "O_NONBLOCK", 0))


def _read_bounded(path: pathlib.Path) -> bytes:
    """Read one bounded regular-file candidate without accepting streaming file types."""

    try:
        with open(path, "rb", opener=_nonblocking_read_opener) as source:
            if not stat.S_ISREG(os.fstat(source.fileno()).st_mode):
                raise SpdxJsonLdEnvelopeError("invalid_file_type")
            payload = source.read(MAX_SPDX_JSONLD_BYTES + 1)
    except OSError as error:
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
