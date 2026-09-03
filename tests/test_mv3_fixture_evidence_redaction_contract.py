"""Regression contracts for MV3 evidence diagnostic redaction."""

from __future__ import annotations

import pathlib
import runpy
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
SENSITIVE_MARKERS = (
    "secret-token",
    "/home/runner/private",
    "example.invalid",
)


class ManifestV3FixtureEvidenceRedactionTests(unittest.TestCase):
    """Prevent page-controlled fixture values from entering CI diagnostics."""

    def assert_redacted(self, rendered: str) -> None:
        """Require representative token, path, and URL material to be absent."""

        for marker in SENSITIVE_MARKERS:
            with self.subTest(marker=marker):
                self.assertNotIn(marker, rendered)

    def test_fixture_timeout_does_not_echo_page_controlled_values(self) -> None:
        """A non-converging fixture must report local classifications, not DOM values."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_fixture_redaction_contract")
        wait_for_extension_evidence = namespace["_wait_for_extension_evidence"]
        sensitive = "secret-token /home/runner/private https://example.invalid"
        hostile = {
            "content": sensitive,
            "storage": sensitive,
            "storagePersistence": sensitive,
            "workerReply": sensitive,
            "workerState": sensitive,
            "workerStartCount": "0",
            "dnr": sensitive,
            "tabs": sensitive,
            "windows": sensitive,
            "scripting": sensitive,
            "scriptingExecuted": sensitive,
            "commands": sensitive,
            "sidePanel": sensitive,
            "bookmarks": sensitive,
            "history": sensitive,
        }

        with unittest.mock.patch.dict(
            wait_for_extension_evidence.__globals__,
            {
                "_execute": unittest.mock.Mock(return_value=hostile),
                "FIXTURE_TIMEOUT_SECONDS": 1.0,
            },
        ), unittest.mock.patch.object(
            namespace["time"],
            "monotonic",
            side_effect=(0.0, 0.0, 2.0),
        ), unittest.mock.patch.object(namespace["time"], "sleep", return_value=None):
            with self.assertRaises(RuntimeError) as raised:
                wait_for_extension_evidence(9515, "session", "initialized")

        self.assert_redacted(str(raised.exception))

    def test_click_mismatch_does_not_echo_page_controlled_text(self) -> None:
        """A click post-condition mismatch must not retain returned DOM text."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_click_redaction_contract")
        exercise_real_click = namespace["_exercise_real_click"]
        element_key = namespace["W3C_ELEMENT_KEY"]
        sensitive = "secret-token /home/runner/private https://example.invalid"
        responses = (
            {"value": {element_key: "element.one"}},
            {},
            {"value": {element_key: "element.two"}},
            {"value": sensitive},
        )

        with unittest.mock.patch.dict(
            exercise_real_click.__globals__,
            {"_json_request": unittest.mock.Mock(side_effect=responses)},
        ):
            with self.assertRaises(RuntimeError) as raised:
                exercise_real_click(9515, "session")

        self.assert_redacted(str(raised.exception))


if __name__ == "__main__":
    unittest.main()
