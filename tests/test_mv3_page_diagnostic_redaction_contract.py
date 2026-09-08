"""Regression tests for credential-safe page-derived browser evidence failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
HOSTILE_PAGE_VALUE = "buyer-secret-marker-must-not-reach-ci"


class Mv3PageDiagnosticRedactionContractTests(unittest.TestCase):
    """Page-controlled observations may decide failure but must not become CI text."""

    def test_real_click_failure_does_not_echo_page_text(self) -> None:
        """A mismatched DOM post-condition must keep page text out of diagnostics."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_click_diagnostic_contract")
        exercise = namespace["_exercise_real_click"]
        element_ids = iter(("button-element", "output-element"))
        request_count = 0

        def find_element(*_args: object, **_kwargs: object) -> str:
            return next(element_ids)

        def json_request(*_args: object, **_kwargs: object) -> dict[str, object]:
            nonlocal request_count
            request_count += 1
            if request_count == 1:
                return {"value": None}
            return {"value": HOSTILE_PAGE_VALUE}

        exercise.__globals__["_find_element"] = find_element
        exercise.__globals__["_json_request"] = json_request

        with self.assertRaises(RuntimeError) as captured:
            exercise(9515, "session-1")

        self.assertEqual(str(captured.exception), "real click post-condition failed")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))

    def test_mv3_convergence_failure_does_not_echo_page_dataset(self) -> None:
        """Untrusted extension/page dataset values must not be serialized into CI errors."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_surface_diagnostic_contract")
        wait_for_evidence = namespace["_wait_for_extension_evidence"]
        wait_for_evidence.__globals__["FIXTURE_TIMEOUT_SECONDS"] = 0.5
        wait_for_evidence.__globals__["_execute"] = lambda *_args, **_kwargs: {
            "content": HOSTILE_PAGE_VALUE,
            "workerStartCount": "1",
        }

        time_module = wait_for_evidence.__globals__["time"]
        with patch.object(time_module, "monotonic", side_effect=(0.0, 0.0, 1.0)), patch.object(
            time_module,
            "sleep",
            return_value=None,
        ), self.assertRaises(RuntimeError) as captured:
            wait_for_evidence(9515, "session-1", "initialized")

        self.assertEqual(str(captured.exception), "MV3 fixture did not converge")
        self.assertNotIn(HOSTILE_PAGE_VALUE, str(captured.exception))


if __name__ == "__main__":
    unittest.main()
