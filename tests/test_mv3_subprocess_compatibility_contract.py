"""Regression contract for portable, bounded ChromeDriver output decoding."""

from __future__ import annotations

import ast
import io
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ChromeDriverSubprocessCompatibilityContractTests(unittest.TestCase):
    """Keep subprocess decoding explicit and bound retained startup-line memory."""

    def test_chromedriver_popen_uses_binary_pipe_with_explicit_utf8_decode(self) -> None:
        """Popen must avoid text-decoding kwargs and decode bounded output explicitly."""

        source = RUNNER.read_text(encoding="utf-8")
        tree = ast.parse(source)
        start = next(
            node
            for node in tree.body
            if isinstance(node, ast.FunctionDef) and node.name == "_start_chromedriver"
        )
        popen_calls = [
            node
            for node in ast.walk(start)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "subprocess"
            and node.func.attr == "Popen"
        ]
        self.assertEqual(len(popen_calls), 1)
        keyword_names = {keyword.arg for keyword in popen_calls[0].keywords}
        self.assertTrue({"text", "encoding", "errors"}.isdisjoint(keyword_names))

        self.assertIn('raw_line_bytes.decode("utf-8", errors="replace")', source)
        self.assertIn("len(raw_line_bytes) > MAX_CHROMEDRIVER_STARTUP_LINE_BYTES", source)

    def test_startup_output_reader_never_requests_an_unbounded_line(self) -> None:
        """A newline-free subprocess record must be drained in bounded reads."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_startup_output_bound")
        read_line = namespace["_read_chromedriver_startup_line"]
        maximum = namespace["MAX_CHROMEDRIVER_STARTUP_LINE_BYTES"]

        class RecordingStream(io.BytesIO):
            def __init__(self, initial_bytes: bytes) -> None:
                super().__init__(initial_bytes)
                self.requested_sizes: list[int] = []

            def readline(self, size: int = -1) -> bytes:
                self.requested_sizes.append(size)
                return super().readline(size)

        stream = RecordingStream(b"x" * (maximum * 4) + b"\nnext\n")
        raw_line, oversized = read_line(stream)

        self.assertTrue(oversized)
        self.assertLessEqual(len(raw_line), maximum + 1)
        self.assertTrue(stream.requested_sizes)
        self.assertNotIn(-1, stream.requested_sizes)
        self.assertLessEqual(max(stream.requested_sizes), maximum + 1)
        self.assertEqual(read_line(stream), (b"next\n", False))


if __name__ == "__main__":
    unittest.main()
