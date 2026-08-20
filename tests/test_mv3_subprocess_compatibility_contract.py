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

    def test_startup_port_parser_treats_malformed_candidates_as_non_authoritative(self) -> None:
        """Malformed candidate records must be ignorable while a later valid record can win."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_startup_port_parser")
        parse_bound_port = namespace["_parse_chromedriver_bound_port"]
        prefix = namespace["CHROMEDRIVER_BOUND_PORT_PREFIX"].encode("ascii")
        maximum = namespace["MAX_CHROMEDRIVER_STARTUP_LINE_BYTES"]

        self.assertIsNone(parse_bound_port(prefix + b"not-a-port.\n", False))
        self.assertIsNone(parse_bound_port(prefix + b"9515\n", False))
        self.assertIsNone(parse_bound_port(prefix + b"9515.\n", True))
        self.assertIsNone(parse_bound_port(b"ordinary ChromeDriver diagnostic\n", False))
        self.assertIsNone(parse_bound_port(prefix + b"0.\n", False))
        self.assertIsNone(parse_bound_port(prefix + b"65536.\n", False))
        self.assertEqual(parse_bound_port(prefix + b"9515.\n", False), 9515)
        self.assertLess(len(prefix) + len(b"9515.\n"), maximum)

    def test_chromedriver_startup_recovers_after_malformed_candidate(self) -> None:
        """One malformed candidate must not prevent a later valid bound-port record."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_startup_recovery")
        start_chromedriver = namespace["_start_chromedriver"]
        prefix = namespace["CHROMEDRIVER_BOUND_PORT_PREFIX"].encode("ascii")
        subprocess_module = namespace["subprocess"]

        class FakeDriver:
            def __init__(self) -> None:
                self.stdout = io.BytesIO(
                    prefix + b"not-a-port.\n" + prefix + b"9515.\n"
                )
                self.terminate_calls = 0
                self.kill_calls = 0
                self.wait_calls = 0

            def terminate(self) -> None:
                self.terminate_calls += 1

            def kill(self) -> None:
                self.kill_calls += 1

            def wait(self, timeout: float | None = None) -> int:
                del timeout
                self.wait_calls += 1
                return 0

        fake_driver = FakeDriver()
        original_popen = subprocess_module.Popen
        subprocess_module.Popen = lambda *args, **kwargs: fake_driver
        try:
            returned_driver, bound_port = start_chromedriver(
                pathlib.Path("/reviewed/chromedriver")
            )
        finally:
            subprocess_module.Popen = original_popen

        self.assertIs(returned_driver, fake_driver)
        self.assertEqual(bound_port, 9515)
        self.assertEqual(fake_driver.terminate_calls, 0)
        self.assertEqual(fake_driver.kill_calls, 0)
        self.assertEqual(fake_driver.wait_calls, 0)


if __name__ == "__main__":
    unittest.main()
