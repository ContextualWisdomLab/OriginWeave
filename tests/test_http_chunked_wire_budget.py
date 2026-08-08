#!/usr/bin/env python3
"""Govern the bounded in-memory wire budget for strict HTTP/1.1 chunked bodies."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CHUNKED = ROOT / "crates" / "originweave-http" / "src" / "chunked.rs"
POLICY = ROOT / "crates" / "originweave-http" / "src" / "policy.rs"


def rust_usize_constant(source: pathlib.Path, name: str) -> int:
    """Return one simple Rust usize constant after evaluating KiB/MiB products."""

    text = source.read_text(encoding="utf-8")
    match = re.search(
        rf"(?:pub\(crate\)\s+)?(?:pub\s+)?const\s+{re.escape(name)}\s*:\s*usize\s*=\s*([^;]+);",
        text,
    )
    if match is None:
        raise AssertionError(f"missing Rust usize constant {name}")
    expression = match.group(1).replace("_", "").strip()
    if not re.fullmatch(r"[0-9* ()]+", expression):
        raise AssertionError(f"unsupported constant expression for {name}: {expression}")
    return int(eval(expression, {"__builtins__": {}}, {}))  # noqa: S307 - numeric-only grammar above


class ChunkedWireBudgetTests(unittest.TestCase):
    """Keep the pre-parse chunked wire buffer near the encoded-content budget."""

    def test_default_chunked_wire_buffer_is_below_eighteen_mib(self) -> None:
        """A hostile chunk syntax stream must not regain the former ~80 MiB buffer bound."""

        max_chunk_line_bytes = rust_usize_constant(CHUNKED, "MAX_CHUNK_LINE_BYTES")
        max_chunk_count = rust_usize_constant(POLICY, "DEFAULT_MAX_CHUNK_COUNT")
        max_trailer_section_bytes = rust_usize_constant(
            POLICY, "DEFAULT_MAX_TRAILER_SECTION_BYTES"
        )
        max_encoded_content_bytes = rust_usize_constant(
            POLICY, "DEFAULT_MAX_ENCODED_CONTENT_BYTES"
        )

        per_chunk_overhead = max_chunk_line_bytes + 4
        wire_bound = (
            max_encoded_content_bytes
            + max_chunk_count * per_chunk_overhead
            + max_trailer_section_bytes
            + max_chunk_line_bytes
            + 4
        )

        self.assertLessEqual(max_chunk_line_bytes, 16)
        self.assertEqual(wire_bound, 18_104_340)
        self.assertLess(wire_bound, 18 * 1024 * 1024)


if __name__ == "__main__":
    unittest.main()
