"""Fail-closed contract for real-click diagnostic handling in the MV3 runner."""

from __future__ import annotations

import pathlib
import runpy
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3ClickDiagnosticContractTests(unittest.TestCase):
    """Keep browser-controlled click postconditions out of exception text."""

    def test_click_mismatch_does_not_retain_raw_browser_text(self) -> None:
        """A failed click must classify the mismatch without copying page-controlled text."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_click_contract")
        exercise = namespace["_exercise_real_click"]
        element_key = namespace["W3C_ELEMENT_KEY"]
        raw_text = "secret-token /home/runner/private https://example.invalid"
        responses = iter(
            (
                {"value": {element_key: "f.1.d.2.e.3"}},
                {"value": {}},
                {"value": {element_key: "f.4.d.5.e.6"}},
                {"value": raw_text},
            )
        )

        with unittest.mock.patch.dict(
            exercise.__globals__,
            {"_json_request": unittest.mock.Mock(side_effect=lambda *_a, **_k: next(responses))},
        ):
            with self.assertRaises(RuntimeError) as raised:
                exercise(9515, "session.1")

        rendered = str(raised.exception)
        self.assertEqual(rendered, "real click post-condition mismatch")
        self.assertNotIn("secret-token", rendered)
        self.assertNotIn("/home/runner/private", rendered)
        self.assertNotIn("example.invalid", rendered)


if __name__ == "__main__":
    unittest.main()
