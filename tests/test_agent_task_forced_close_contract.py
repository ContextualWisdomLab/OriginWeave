"""Contract for deterministic forced-close failure evidence in the Agent Task lane."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskForcedCloseContractTests(unittest.TestCase):
    """Require one real-browser interruption probe without normalizing failures."""

    def test_runner_exposes_forced_close_probe(self) -> None:
        """The pinned-browser lane must have an executable forced-close boundary."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_forced_close_contract")
        for expected in (
            "_force_close_agent_task_context",
            "_run_agent_task_forced_close_browser_pass",
            "_run_agent_task_forced_close_trial",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)

    def test_forced_close_requires_the_context_to_become_unusable(self) -> None:
        """A closed top-level context must fail the next command as no-such-window."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_forced_close_behavior")
        force_close = namespace["_force_close_agent_task_context"]
        requests: list[tuple[str, str]] = []

        def closed_context_request(
            _driver_port: int,
            method: str,
            path: str,
            _payload: object | None = None,
            **_kwargs: object,
        ) -> dict[str, object]:
            requests.append((method, path))
            if method == "DELETE" and path.endswith("/window"):
                return {"value": []}
            if method == "GET" and path.endswith("/url"):
                raise RuntimeError("WebDriver error: no such window: controlled close")
            raise AssertionError(f"unexpected request: {method} {path}")

        force_close.__globals__["_json_request"] = closed_context_request
        self.assertTrue(force_close(4444, "session-a"))
        self.assertEqual(
            requests,
            [
                ("DELETE", "/session/session-a/window"),
                ("GET", "/session/session-a/url"),
            ],
        )

    def test_forced_close_rejects_surviving_or_ambiguous_context_state(self) -> None:
        """The probe must fail closed if the close did not produce exact evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_forced_close_fail_closed")
        force_close = namespace["_force_close_agent_task_context"]

        def surviving_context_request(
            _driver_port: int,
            method: str,
            path: str,
            _payload: object | None = None,
            **_kwargs: object,
        ) -> dict[str, object]:
            if method == "DELETE" and path.endswith("/window"):
                return {"value": []}
            if method == "GET" and path.endswith("/url"):
                return {"value": "http://127.0.0.1/fixture"}
            raise AssertionError(f"unexpected request: {method} {path}")

        force_close.__globals__["_json_request"] = surviving_context_request
        with self.assertRaisesRegex(RuntimeError, "remained usable after forced close"):
            force_close(4444, "session-a")

        force_close = runpy.run_path(
            str(RUNNER), run_name="agent_task_forced_close_wrong_error"
        )["_force_close_agent_task_context"]

        def wrong_failure_request(
            _driver_port: int,
            method: str,
            path: str,
            _payload: object | None = None,
            **_kwargs: object,
        ) -> dict[str, object]:
            if method == "DELETE" and path.endswith("/window"):
                return {"value": []}
            if method == "GET" and path.endswith("/url"):
                raise RuntimeError("WebDriver error: timeout: unrelated failure")
            raise AssertionError(f"unexpected request: {method} {path}")

        force_close.__globals__["_json_request"] = wrong_failure_request
        with self.assertRaisesRegex(RuntimeError, "timeout"):
            force_close(4444, "session-a")

    def test_main_evidence_requires_forced_close_and_profile_cleanup(self) -> None:
        """Compatibility success must include the interruption probe and cleanup proof."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            '"forced_close"',
            '"forced_close_detected"',
            '"profile_cleaned"',
            "Agent Task forced-close recovery gate failed",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
