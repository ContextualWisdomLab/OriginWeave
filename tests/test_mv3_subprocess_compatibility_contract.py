"""Regression contract for portable, explicit ChromeDriver output decoding."""

from __future__ import annotations

import ast
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ChromeDriverSubprocessCompatibilityContractTests(unittest.TestCase):
    """Keep subprocess decoding explicit instead of version-gated Popen kwargs."""

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


if __name__ == "__main__":
    unittest.main()