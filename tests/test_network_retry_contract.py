"""Repository contract for bounded direct-TCP retry classification."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONNECTION = ROOT / "crates/originweave-network/src/connection.rs"


class NetworkRetryContractTests(unittest.TestCase):
    """Prevent deterministic operating-system failures from wasting retry budget."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.source = CONNECTION.read_text(encoding="utf-8")

    def test_connect_retries_only_explicit_transient_error_kinds(self) -> None:
        """Retries must be allow-listed instead of applying to every I/O error."""

        self.assertIn("fn is_retryable_connect_error", self.source)
        self.assertIn(
            "Err(source) if is_retryable_connect_error(source.kind())",
            self.source,
        )
        for kind in (
            "TimedOut",
            "ConnectionRefused",
            "ConnectionReset",
            "ConnectionAborted",
            "Interrupted",
        ):
            with self.subTest(kind=kind):
                self.assertIn(f"io::ErrorKind::{kind}", self.source)

    def test_nonretryable_errors_have_behavioral_rust_regressions(self) -> None:
        """Permission and caller-input failures must stop after one attempt."""

        self.assertIn("non_retryable_connection_errors_stop_immediately", self.source)
        self.assertIn("io::ErrorKind::PermissionDenied", self.source)
        self.assertIn("io::ErrorKind::InvalidInput", self.source)
        self.assertIn("assert_eq!(connector.connect_calls.get(), 1)", self.source)


if __name__ == "__main__":
    unittest.main()
