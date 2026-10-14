"""Regression contract for Python 3.10-compatible teardown diagnostics."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class _Python310OSError(OSError):
    """Emulate Python 3.10, where BaseException.add_note is unavailable."""

    def __getattribute__(self, name: str):
        if name == "add_note":
            raise AttributeError(name)
        return super().__getattribute__(name)


class _FallbackFailureDriver:
    """Fail graceful termination and the bounded hard-kill fallback."""

    def __init__(self, terminate_error: OSError) -> None:
        self.terminate_error = terminate_error
        self.killed = False

    def terminate(self) -> None:
        """Raise the causal teardown error."""

        raise self.terminate_error

    def kill(self) -> None:
        """Model a reviewed secondary hard-kill failure."""

        self.killed = True
        raise PermissionError("controlled kill denial")

    def wait(self, timeout: float) -> int:
        """The hard-kill failure occurs before a reap can be attempted."""

        raise AssertionError(f"unexpected wait after kill failure: {timeout}")


class ManifestV3Python310CleanupNoteTests(unittest.TestCase):
    """Secondary cleanup diagnostics must not mask the causal teardown error."""

    def test_missing_add_note_preserves_primary_and_bounded_secondary_type(self) -> None:
        """Python 3.10 must retain the primary error without leaking fallback text."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_python310_cleanup_contract")
        teardown_driver_process = namespace["_teardown_driver_process"]
        primary_error = _Python310OSError("controlled terminate failure")
        driver = _FallbackFailureDriver(primary_error)

        error = teardown_driver_process(driver)

        self.assertIs(error, primary_error)
        self.assertTrue(driver.killed)
        self.assertIn(
            "bounded ChromeDriver kill fallback also failed: PermissionError",
            getattr(error, "_originweave_secondary_diagnostics", []),
        )
        self.assertNotIn("controlled kill denial", repr(error.__dict__))


if __name__ == "__main__":
    unittest.main()
