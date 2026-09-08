"""Contract tests for credential-safe ChromeDriver process-start diagnostics."""

from __future__ import annotations

import io
import os
import pathlib
import runpy
import threading
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER_PATH = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ChromeDriverProcessDiagnosticContractTests(unittest.TestCase):
    """Keep process diagnostics bounded to reviewed reason codes and drained continuously."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.runner = runpy.run_path(
            str(RUNNER_PATH),
            run_name="originweave_mv3_process_diagnostic_contract",
        )

    def test_split_sandbox_marker_is_classified_without_raw_retention(self) -> None:
        diagnostic_type = self.runner["_ChromeDriverStartupDiagnostic"]
        diagnostic = diagnostic_type()

        diagnostic.feed(b"prefix /tmp/private-profile token=do-not-retain No usable sand")
        self.assertEqual(diagnostic.startup_reason, "unknown")
        diagnostic.feed(b"box! suffix secret-shaped-value")

        self.assertEqual(diagnostic.startup_reason, "sandbox_unavailable")
        rendered_state = repr(diagnostic)
        self.assertNotIn("private-profile", rendered_state)
        self.assertNotIn("do-not-retain", rendered_state)
        self.assertNotIn("secret-shaped-value", rendered_state)
        self.assertFalse(hasattr(diagnostic, "__dict__"))

    def test_unknown_process_diagnostic_stays_unknown(self) -> None:
        diagnostic_type = self.runner["_ChromeDriverStartupDiagnostic"]
        diagnostic = diagnostic_type()

        diagnostic.feed(b"session not created: /tmp/private-profile bearer-secret")

        self.assertEqual(diagnostic.startup_reason, "unknown")
        self.assertNotIn("private-profile", repr(diagnostic))
        self.assertNotIn("bearer-secret", repr(diagnostic))

    def test_drain_consumes_large_pipe_without_retaining_payload(self) -> None:
        diagnostic_type = self.runner["_ChromeDriverStartupDiagnostic"]
        drain = self.runner["_drain_chromedriver_diagnostics"]
        diagnostic = diagnostic_type()
        read_fd, write_fd = os.pipe()
        reader = os.fdopen(read_fd, "rb", buffering=0)
        writer = os.fdopen(write_fd, "wb", buffering=0)
        thread = threading.Thread(target=drain, args=(reader, diagnostic), daemon=True)
        thread.start()
        try:
            writer.write(b"x" * 262_144)
            writer.write(b"No usable sand")
            writer.write(b"box!")
        finally:
            writer.close()
        thread.join(timeout=2.0)
        reader.close()

        self.assertFalse(thread.is_alive(), "ChromeDriver diagnostic pipe drain deadlocked")
        self.assertEqual(diagnostic.startup_reason, "sandbox_unavailable")
        self.assertNotIn("x" * 32, repr(diagnostic))

    def test_bytesio_drain_keeps_unreviewed_text_out_of_state(self) -> None:
        diagnostic_type = self.runner["_ChromeDriverStartupDiagnostic"]
        drain = self.runner["_drain_chromedriver_diagnostics"]
        diagnostic = diagnostic_type()
        sensitive = b"/home/runner/private-profile bearer-super-secret"

        drain(io.BytesIO(sensitive), diagnostic)

        self.assertEqual(diagnostic.startup_reason, "unknown")
        self.assertNotIn("private-profile", repr(diagnostic))
        self.assertNotIn("super-secret", repr(diagnostic))

    def test_all_chromedriver_launches_stream_instead_of_discarding_output(self) -> None:
        source = RUNNER_PATH.read_text(encoding="utf-8")

        self.assertNotIn("stdout=subprocess.DEVNULL", source)
        self.assertIn("stdout=subprocess.PIPE", source)
        self.assertEqual(source.count("_start_chromedriver("), 5)
        self.assertEqual(source.count("_create_chromedriver_session("), 5)

    def test_shared_launch_enables_verbose_diagnostics_without_log_file(self) -> None:
        source = RUNNER_PATH.read_text(encoding="utf-8")

        self.assertIn('"--verbose"', source)
        self.assertNotIn("--log-path", source)


if __name__ == "__main__":
    unittest.main()
