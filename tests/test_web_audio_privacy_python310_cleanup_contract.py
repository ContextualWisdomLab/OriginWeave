"""Regression contract for Python 3.10-safe Web Audio cleanup failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_web_audio_privacy.py"


class WebAudioPrivacyPython310CleanupContractTests(unittest.TestCase):
    """Keep a causal browser failure primary when exception notes are unavailable."""

    def test_cleanup_failure_does_not_require_exception_add_note(self) -> None:
        """Cleanup recovery must remain correct on Python 3.10."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="web_audio_python310_cleanup_contract"
        )
        cleanup = namespace["_cleanup_browser_session_preserving_primary"]
        globals_dict = cleanup.__globals__
        original_cleanup = globals_dict["_cleanup_browser_session"]

        class Python310StylePrimaryError(RuntimeError):
            """Model a BaseException implementation without Python 3.11 add_note."""

            add_note = None

        def fail_cleanup(_driver_port: int, _session_id: str) -> None:
            raise OSError("simulated bounded cleanup failure")

        globals_dict["_cleanup_browser_session"] = fail_cleanup
        try:
            primary = Python310StylePrimaryError("primary browser failure")
            with self.assertRaises(Python310StylePrimaryError) as raised:
                cleanup(9515, "safe-session", primary)
            self.assertIs(raised.exception, primary)
            self.assertIsInstance(raised.exception.__context__, OSError)
        finally:
            globals_dict["_cleanup_browser_session"] = original_cleanup


if __name__ == "__main__":
    unittest.main()
